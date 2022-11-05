use actix_web::{web, App, HttpServer, Responder};
use edgedb_tokio::Client;
use serde::Deserialize;
use std::sync::Mutex;

#[derive(Deserialize)]
struct CreateUserInput {
    email: String,
    name: String,
}

struct AppState {
    pub client: Client,
}

async fn index(data: web::Data<Mutex<AppState>>) -> impl Responder {
    let conn = &data.lock().unwrap().client;
    let result = conn
        .query_json("SELECT User { email, name }", &())
        .await;
    match result {
        Ok(val) => val.to_string(),
        Err(_) => "Error".to_string(),
    }
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let conn = edgedb_tokio::create_client().await?;

    let data = web::Data::new(Mutex::new(AppState { client: conn }));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::clone(&data))
            .route("/", web::get().to(index))
            .route("/user", web::post().to(create_user))
            // //.route("/polls", web::post().to())
            // //.route("/polls/{pollID}", web::get().to())
            // .route("/pollResponses", web::post().to())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
