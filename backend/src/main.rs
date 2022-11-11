use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{cookie::Key, middleware::Logger, web, App, Error, HttpServer, Responder};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use edgedb_tokio::Client;
use env_logger::Env;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Mutex;

struct AppState {
    pub client: Client,
}

async fn index(data: web::Data<Mutex<AppState>>) -> impl Responder {
    let conn = &data.lock().unwrap().client;
    let result = conn.query_json("SELECT User { email, name }", &()).await;
    match result {
        Ok(val) => val.to_string(),
        Err(_) => "Error".to_string(),
    }
}

// TODO auth middleware
// session.get::<String>("email")?.unwrap();

#[derive(Deserialize)]
struct CreateUserInput {
    email: String,
    name: String,
}

async fn create_user(
    info: web::Json<CreateUserInput>,
    data: web::Data<Mutex<AppState>>,
) -> impl Responder {
    let conn = &data.lock().unwrap().client;
    let result = conn
        .query_required_single_json(
            "insert User {
        email := <str>$0,
        name := <str>$1
    };",
            &(&info.email, &info.name),
        )
        .await;
    match result {
        Ok(result) => result.to_string(),
        Err(_) => "Error".to_string(),
    }
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
        Ok(result) => result.to_string(),
        Err(_) => "Error".to_string(),
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
) -> impl Responder {
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
                "maxmo@gmail.com",
                &input.poll_id,
            ),
        )
        .await;
    match result {
        Ok(result) => result.to_string(),
        Err(e) => {
            println!("{}", e);
            "Error".to_string()
        }
    }
}

#[derive(Deserialize)]
struct SearchParams {
    hasVotedIn: bool,
}

async fn get_polls(
    data: web::Data<Mutex<AppState>>,
    query: web::Query<SearchParams>,
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
                question_text,
                user_response :=(
                    Select PollResponse { choice }
                        filter PollResponse.poll.id = Poll.id and PollResponse.user.email = <str>$0
                )
            } filter count((
                Select PollResponse 
                    filter PollResponse.poll.id = Poll.id and PollResponse.user.email = <str>$0
            )) = <int64>$1;",
            &(
                "maxmo@gmail.com",
                if query.hasVotedIn { 1i64 } else { 0i64 },
            ),
        )
        .await;
    match result {
        Ok(result) => result.to_string(),
        Err(e) => {
            println!("{}", e);
            "Error".to_string()
        }
    }
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
            "select User { password_hash } filter User.email = <str>$0;",
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
        .ok_or(actix_web::error::ErrorInternalServerError(
            "User not found.",
        ))?
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

    Result::<String, Error>::Ok("{}".to_string())
}

async fn logout(session: Session) -> impl Responder {
    println!("{:#?}", session.entries()); // if we don't read the session, session.purge() won't do anything
    session.purge();
    "{}"
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = edgedb_tokio::create_client().await?;
    client.ensure_connected().await?;
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    let data = web::Data::new(Mutex::new(AppState { client }));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
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
            .app_data(web::Data::clone(&data))
            .route("/", web::get().to(index))
            .route("/users", web::post().to(create_user))
            .route("/polls", web::post().to(suggest_poll))
            .route("/polls", web::get().to(get_polls))
            .route("/pollResponses", web::post().to(create_poll_input))
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/logout", web::get().to(logout))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
