use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware, SessionInsertError};
use actix_web::{
    cookie::Key, middleware::Logger, web, App, Error, HttpResponse, HttpServer, Responder,
};
use edgedb_tokio::Client;
use env_logger::Env;
use serde::Deserialize;
use std::sync::Mutex;
use std::error;

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
struct LoginInput {
    email: String,
    password: String
}

// check to see if registered
async fn login(session: Session, input: web::Json<LoginInput>) -> impl Responder {
    let conn = &data.lock().unwrap().client;
    session.insert("email", &input.email)?;

    // session.get::<i32>("counter")?.unwrap();
    Result::<String, Error>::Ok("{}".to_string())
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
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .cookie_secure(false)
                    .build(),
            )
            .app_data(web::Data::clone(&data))
            .route("/", web::get().to(index))
            .route("/users", web::post().to(create_user))
            .route("/polls", web::post().to(suggest_poll))
            .route("/polls", web::get().to(get_polls))
            .route("/pollResponses", web::post().to(create_poll_input))
            .route("/login", web::post().to(login))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
