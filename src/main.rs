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
use env::Env;
use handler::{download, message, socket, upload};
use utils::into_layer;

#[tokio::main]
async fn main() {
    let env = Env::new();

    let mut args = Arguments::from_env();

    if args.contains("--init") {
        init::init(&env).await;
    } else {
        server(env).await;
    }
}

async fn server(env: Env) {
    let port = env.port;

    let storage = get_storage(&env);
    let database = get_database(&env).await;

    let secret_key = database.get_secret_key().await.unwrap();
    let crypto = Crypto::new(&secret_key).unwrap();

    let (socketio_layer, socketio) = SocketIo::builder()
        .with_state(socket::ConnectionNumber::new())
        .build_layer();

    socketio.ns(
        "/",
        |s: SocketRef, connection_number: State<socket::ConnectionNumber>| {
            socket::connect(&s, connection_number);
            s.on_disconnect(socket::disconnect);
            s.on(socket::PROGRESS_EVENT, socket::progress);
        },
    );

    let router = Router::new()
        .route(download::DOWNLOAD_URL_PATH, get(download::download_url))
        .route(message::PAGE_PATH, get(message::page))
        .route(message::SYNC_PATH, get(message::sync))
        .route(message::NEW_ITEM_PATH, post(message::new_item))
        .route(message::REMOVE_ITEM_PATH, post(message::remove_item))
        .route(message::REMOVE_ALL_PATH, get(message::remove_all))
        .route(upload::FETCH_UPLOAD_ID_PATH, post(upload::fetch_upload_id))
        .route(upload::UPLOAD_PART_PATH, post(upload::upload_part))
        .route(upload::COMPLETE_UPLOAD_PATH, post(upload::complete_upload))
        .layer(socketio_layer)
        .layer(into_layer(socketio))
        .layer(into_layer(env))
        .layer(into_layer(storage))
        .layer(into_layer(database))
        .layer(into_layer(crypto));

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .unwrap();

    axum::serve(listener, router).await.unwrap();
}
