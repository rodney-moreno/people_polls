use actix_cors::Cors;
use actix_session::{storage::CookieSessionStore, Session, SessionExt, SessionMiddleware};
use actix_web::{
    body::BoxBody,
    cookie::Key,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http,
    middleware::Logger,
    web, App, Error, HttpMessage, HttpResponse, HttpServer, Responder,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use edgedb_tokio::Client;
use env_logger::Env;
use futures::future::LocalBoxFuture;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::sync::Mutex;

struct AppState {
    pub client: Client,
}

#[derive(Debug, Clone)]
struct SessionUser {
    email: String,
}

pub struct Authenticate;

impl<S, B> Transform<S, ServiceRequest> for Authenticate
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static + actix_web::body::MessageBody,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticateMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticateMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthenticateMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthenticateMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static + actix_web::body::MessageBody,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = Rc::clone(&self.service);
        Box::pin(async move {
            let session = req.get_session();
            println!("{:?}", session.entries());

            if session.entries().is_empty() || session.get::<String>("email")?.is_none() {
                let resp = HttpResponse::Unauthorized().body("{}");
                Ok(req.into_response(resp).map_into_boxed_body())
            } else {
                req.extensions_mut().insert(SessionUser {
                    email: session.get::<String>("email")?.unwrap(),
                });
                Ok(srv.call(req).await?.map_into_boxed_body())
            }
        })
    }
}

async fn index(data: web::Data<Mutex<AppState>>) -> impl Responder {
    let conn = &data.lock().unwrap().client;
    let result = conn.query_json("SELECT User { email, name }", &()).await;
    match result {
        Ok(result) => Result::Ok(result.to_string()),
        Err(e) => Result::Err(actix_web::error::ErrorBadRequest(e)),
    }
}

async fn get_current_user(
    data: web::Data<Mutex<AppState>>,
    session_user: Option<web::ReqData<SessionUser>>,
) -> impl Responder {
    let conn = &data.lock().unwrap().client;
    let email = &session_user
        .ok_or(actix_web::error::ErrorInternalServerError(
            "Session user is missing.",
        ))?
        .email;
    let json = conn
        .query_json(
            "select User { name } filter User.email = <str>$0",
            &(email.as_str(),),
        )
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    let parsed: Value = serde_json::from_str(json.as_ref())
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    let users = parsed
        .as_array()
        .ok_or(actix_web::error::ErrorInternalServerError(
            "JSON from database is not in the expected shape.",
        ))?;
    let name = users
        .get(0)
        .ok_or(actix_web::error::ErrorBadRequest("User not found."))?
        .as_object()
        .ok_or(actix_web::error::ErrorInternalServerError(
            "JSON from database is not in the expected shape.",
        ))?
        .get("name")
        .ok_or(actix_web::error::ErrorInternalServerError(
            "JSON from database is not in the expected shape.",
        ))?
        .as_str()
        .ok_or(actix_web::error::ErrorInternalServerError(
            "JSON from database is not in the expected shape.",
        ))?;
    Result::<String, Error>::Ok(json!({"email": email.as_str(), "name": name}).to_string())
}

#[derive(Deserialize)]
struct CreatePollInput {
    question_text: String,
    prompt_a: String,
    prompt_b: String,
}

async fn suggest_poll(
    input: web::Json<CreatePollInput>,
    data: web::Data<Mutex<AppState>>,
) -> impl Responder {
    let conn = &data.lock().unwrap().client;
    let result = conn
        .query_required_single_json(
            "insert Poll {
        question_text := <str>$0,
        prompt_a := <str>$1,
        prompt_b := <str>$2,
        is_approved := false
    };",
            &(&input.question_text, &input.prompt_a, &input.prompt_b),
        )
        .await;
    match result {
        Ok(result) => Result::Ok(result.to_string()),
        Err(e) => Result::Err(actix_web::error::ErrorBadRequest(e)),
    }
}

#[derive(Deserialize)]
struct CreatePollResponseInput {
    choice: bool,
    poll_id: String,
}

async fn create_poll_input(
    input: web::Json<CreatePollResponseInput>,
    data: web::Data<Mutex<AppState>>,
    session_user: Option<web::ReqData<SessionUser>>,
) -> impl Responder {
    let email = &session_user
        .ok_or(actix_web::error::ErrorInternalServerError(
            "Session user is missing.",
        ))?
        .email;
    let conn = &data.lock().unwrap().client;
    let result = conn
        .query_required_single_json(
            "insert PollResponse {
            choice := <Choice><str>$0,
            user := (select User filter .email = <str>$1),
            poll := (select Poll filter .id = <uuid><str>$2)
            };",
            &(
                if input.choice { "ChoiceA" } else { "ChoiceB" },
                email,
                &input.poll_id,
            ),
        )
        .await;
    match result {
        Ok(result) => Result::Ok(result.to_string()),
        Err(e) => Result::Err(actix_web::error::ErrorBadRequest(e)),
    }
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct SearchParams {
    hasVotedIn: bool,
}

async fn get_polls(
    data: web::Data<Mutex<AppState>>,
    query: web::Query<SearchParams>,
    session_user: Option<web::ReqData<SessionUser>>,
) -> impl Responder {
    let conn = &data.lock().unwrap().client;
    /*
        1. Show all approved polls we did not vote in (Date)
        2. Show all approved polls that we already voted in, and our responses (Date)
        3. Show all completed polls with calculated results
    */

    let result = conn
        .query_json(
            "Select Poll {
                id,
                question_text,
                prompt_a,
                prompt_b,
                user_response :=(
                    Select PollResponse { choice }
                        filter PollResponse.poll.id = Poll.id and PollResponse.user.email = <str>$0
                )
            } filter count((
                Select PollResponse 
                    filter PollResponse.poll.id = Poll.id and PollResponse.user.email = <str>$0
            )) = <int64>$1 and Poll.is_approved = true;",
            &(
                &session_user
                    .ok_or(actix_web::error::ErrorInternalServerError(
                        "Session user is missing.",
                    ))?
                    .email,
                if query.hasVotedIn { 1i64 } else { 0i64 },
            ),
        )
        .await;
    match result {
        Ok(result) => Result::Ok(result.to_string()),
        Err(e) => Result::Err(actix_web::error::ErrorBadRequest(e)),
    }
}

async fn get_poll_results(
    data: web::Data<Mutex<AppState>>,
    path: web::Path<(String,)>,
) -> impl Responder {
    let poll_id = path.into_inner().0;
    let conn = &data.lock().unwrap().client;

    let result = conn
        .query_json(
            "select Poll {
                id,
                question_text,
                prompt_a,
                prompt_b,
                a_count :=(
                    count((
                        select PollResponse
                            filter PollResponse.poll.id = Poll.id and PollResponse.choice = Choice.ChoiceA
                    ))
                ),
                b_count :=(
                    count((
                        select PollResponse
                            filter PollResponse.poll.id = Poll.id and PollResponse.choice = Choice.ChoiceB
                    ))
                )
            } filter Poll.id = <uuid><str>$0 and Poll.created_at + <duration>'168 hours' <= datetime_current();",
            &(&poll_id,),
        )
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Result::<String, Error>::Ok(result.to_string())
}

#[derive(Deserialize)]
struct RegisterInput {
    email: String,
    password: String,
    name: String,
}

async fn register(
    input: web::Json<RegisterInput>,
    data: web::Data<Mutex<AppState>>,
) -> impl Responder {
    let conn = &data.lock().unwrap().client;

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(input.password.as_bytes(), &salt)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    conn.query_json(
        "insert User { email := <str>$0, password_hash := <str>$1, name := <str>$2 };",
        &(&input.email, password_hash.to_string(), &input.name),
    )
    .await
    .map_err(|e| actix_web::error::ErrorBadRequest(e))?;

    Result::<String, Error>::Ok("{}".to_string())
}

#[derive(Deserialize)]
struct LoginInput {
    email: String,
    password: String,
}

async fn login(
    session: Session,
    input: web::Json<LoginInput>,
    data: web::Data<Mutex<AppState>>,
) -> impl Responder {
    // Check to see if registered and if password is correct
    let conn = &data.lock().unwrap().client;
    let json = conn
        .query_json(
            "select User { password_hash, name } filter User.email = <str>$0;",
            &(&input.email,),
        )
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    let parsed: Value = serde_json::from_str(json.as_ref())
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    let users = parsed
        .as_array()
        .ok_or(actix_web::error::ErrorInternalServerError(
            "JSON from database is not in the expected shape.",
        ))?;

    let user = users
        .get(0)
        .ok_or(actix_web::error::ErrorBadRequest("User not found."))?
        .as_object()
        .ok_or(actix_web::error::ErrorInternalServerError(
            "JSON from database is not in the expected shape.",
        ))?;

    let password_hash = user
        .get("password_hash")
        .ok_or(actix_web::error::ErrorInternalServerError(
            "JSON from database is not in the expected shape.",
        ))?
        .as_str()
        .ok_or(actix_web::error::ErrorInternalServerError(
            "JSON from database is not in the expected shape.",
        ))?;

    let parsed_hash = PasswordHash::new(&password_hash)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    if !Argon2::default()
        .verify_password(input.password.as_bytes(), &parsed_hash)
        .is_ok()
    {
        return Err(actix_web::error::ErrorBadRequest("Incorrect password."));
    }

    session.insert("email", &input.email)?;

    let name = match user.get("name") {
        Some(s) => s
            .as_str()
            .ok_or(actix_web::error::ErrorInternalServerError(
                "JSON from database is not in the expected shape.",
            ))?,
        None => "",
    };

    Result::<String, Error>::Ok(json!({"email": input.email, "name": name}).to_string())
}

async fn logout(session: Session) -> impl Responder {
    // WARNING: if we never read the session, session.purge() won't do anything
    // the Authenticate middleware should have read already
    session.purge();
    "{}"
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = edgedb_tokio::create_client().await?;
    client.ensure_connected().await?;
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    let data = web::Data::new(Mutex::new(AppState { client }));

    HttpServer::new(move || {
        let cors = Cors::default()
        .allowed_origin("http://localhost:3000")
        .allowed_origin("http://127.0.0.1:5173")
        .allowed_methods(vec!["GET", "POST"])
        .allowed_headers(vec![http::header::ACCEPT])
        .allowed_header(http::header::CONTENT_TYPE)
        .supports_credentials();
        App::new()
            .wrap(cors)
            .wrap(
                // create cookie based session middleware
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    Key::from(
                        b"9c53a04cdc7e61db9b9fb1636f345293eac36ad55e653edb0c8524a0a873eb3c5b958534a3f7f26a266472103df5b419de1315d90ade5523d0029362ddf457c0",
                    ),
                )
                .cookie_secure(false)
                .build(),
            )
            .wrap(Logger::default())
            .app_data(web::Data::clone(&data))
            .route("/", web::get().to(index))
            .route("/me", web::get().to(get_current_user).wrap(Authenticate))
            .route("/polls", web::post().to(suggest_poll).wrap(Authenticate))
            .route("/polls", web::get().to(get_polls).wrap(Authenticate))
            .route("/polls/{poll_id}", web::get().to(get_poll_results).wrap(Authenticate))
            .route("/pollResponses", web::post().to(create_poll_input).wrap(Authenticate))
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/logout", web::get().to(logout).wrap(Authenticate))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
