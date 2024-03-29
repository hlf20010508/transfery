/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use actix_web::{App, HttpServer, web};
use pico_args::Arguments;

mod client;
mod env;
mod error;
mod init;

use env::PORT;
use client::{get_storage, get_database};

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

    HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(storage.clone()))
        .app_data(web::Data::new(database.clone()))
    })
    .bind(("127.0.0.1", PORT.clone()))?
    .run()
    .await
}
