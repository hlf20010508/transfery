/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use actix_web::{web, App, HttpServer};
use pico_args::Arguments;

mod auth;
mod client;
mod crypto;
mod env;
mod error;
mod handler;
mod init;
mod utils;

use client::{get_database, get_storage};
use crypto::Crypto;
use env::PORT;
use handler::download;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut args = Arguments::from_env();

    if args.contains("--init") {
        init::init().await;
    } else {
        server().await?;
    }

    Ok(())
}

async fn server() -> std::io::Result<()> {
    let storage = get_storage();
    let database = get_database().await;

    let secret_key = database.get_secret_key().await.unwrap();
    let crypto = Crypto::new(&secret_key).unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(storage.clone()))
            .app_data(web::Data::new(database.clone()))
            .app_data(web::Data::new(crypto.clone()))
            .service(download::download_url)
    })
    .bind(("0.0.0.0", PORT.clone()))?
    .run()
    .await
}
