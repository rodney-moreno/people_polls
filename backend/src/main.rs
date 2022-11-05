use actix_web::{web, App, HttpServer, Responder};
use edgedb_tokio::Client;
use std::sync::Mutex;

struct AppState {
    pub client: Client,
}

async fn index(data: web::Data<Mutex<AppState>>) -> impl Responder {
    let conn = &data.lock().unwrap().client;
    let result = conn
        .query_required_single::<i64, _>("SELECT 7*8", &())
        .await;
    match result {
        Ok(val) => format!("7*8 is: {}", val),
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
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
