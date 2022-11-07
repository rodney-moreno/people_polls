use actix_web::{web, App, HttpServer, Responder};
use edgedb_tokio::Client;
use serde::Deserialize;
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = edgedb_tokio::create_client().await?;
    client.ensure_connected().await?;

    let data = web::Data::new(Mutex::new(AppState { client }));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::clone(&data))
            .route("/", web::get().to(index))
            .route("/users", web::post().to(create_user))
            .route("/polls", web::post().to(suggest_poll))
        // .route("/polls/{pollID}", web::get().to())
        // .route("/pollResponses", web::post().to())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
