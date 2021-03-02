#![warn(clippy::all)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use actix_web::{get, middleware::Logger, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use dotenv::dotenv;
use listenfd::ListenFd;
use oauth2::basic::BasicClient;
use sqlx::PgPool;
use std::env;

mod auth;
mod backend;
mod config;
mod error;
mod models;
mod tts;

pub struct AppState {
    pub oauth: BasicClient,
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("it works!")
}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    let mut listenfd = ListenFd::from_env();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let pool = PgPool::new(&database_url).await?;

    let config = config::Config::from_config()?;

    let mut server = HttpServer::new(move || {
        let oauth = auth::create_auth_client();
        App::new()
            .data(AppState { oauth })
            .data(pool.clone())
            .data(config.clone())
            .wrap(Logger::default())
            .service(index)
            .configure(tts::init)
            .configure(auth::init)
    });

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => {
            let host = env::var("HOST").unwrap_or("0.0.0.0".to_string());
            let port = env::var("PORT").unwrap_or("8080".to_string());
            server.bind(format!("{}:{}", host, port))?
        }
    };

    info!("Starting server");
    server.run().await?;

    Ok(())
}
