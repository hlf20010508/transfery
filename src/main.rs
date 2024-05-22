/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::routing::{get, post};
use axum::Router;
use pico_args::Arguments;
use socketioxide::extract::{SocketRef, State};
use socketioxide::SocketIo;

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
use handler::{download, message, socket, upload};
use utils::into_layer;

#[tokio::main]
async fn main() {
    let mut args = Arguments::from_env();

    if args.contains("--init") {
        init::init().await;
    } else {
        server().await;
    }
}

async fn server() {
    let storage = into_layer(get_storage());
    let database = into_layer(get_database().await);

    let secret_key = database.get_secret_key().await.unwrap();
    let crypto = into_layer(Crypto::new(&secret_key).unwrap());

    let (socketio_layer, socketio) = SocketIo::builder()
        .with_state(socket::ConnectionNumber::new())
        .build_layer();

    socketio.ns(
        "/",
        |socket: SocketRef, connection_number: State<socket::ConnectionNumber>| {
            socket::connect(&socket, connection_number);
        },
    );

    let router = Router::new()
        .route(download::DOWNLOAD_URL_PATH, get(download::download_url))
        .route(message::PAGE_PATH, get(message::page))
        .route(message::SYNC_PATH, get(message::sync))
        .route(upload::FETCH_UPLOAD_ID_PATH, post(upload::fetch_upload_id))
        .route(upload::UPLOAD_PART_PATH, post(upload::upload_part))
        .route(upload::COMPLETE_UPLOAD_PATH, post(upload::complete_upload))
        .layer(storage)
        .layer(database)
        .layer(crypto)
        .layer(socketio_layer);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", PORT.clone()))
        .await
        .unwrap();

    axum::serve(listener, router).await.unwrap();
}
