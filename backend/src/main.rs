use actix_web::{web::{self, Data}, App, HttpServer, Responder};
use edgedb_tokio::Client;
use std::io::Error;
use std::sync::Mutex;

struct AppState {
    pub client: Client
}

async fn index(data: Data<Mutex<AppState>>) -> impl Responder {
    "Hello world!"
    
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let conn = edgedb_tokio::create_client().await?;
    let val = conn
        .query_required_single::<i64, _>("SELECT 7*8", &())
        .await?;
    println!("7*8 is: {}", val);

    let data = Data::new(Mutex::new(AppState { client: conn }));

    HttpServer::new(|| App::new().app_data(Data::clone(&data))
        .route("/", web::get().to(index)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;

    Ok(())
}
