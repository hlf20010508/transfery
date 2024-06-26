/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

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
use env::{args_contains, Env};
use handler::{admin, api, download, index, message, socket, upload};
use utils::into_layer;

use axum::body::Body;
use axum::extract::DefaultBodyLimit;
use axum::http::Request;
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::{get, post};
use axum::Router;
use socketioxide::extract::{SocketRef, State};
use socketioxide::SocketIo;
use std::time::Duration;
use tokio::time::Instant;
use tower_http::services::ServeDir;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

async fn trace_middleware(req: Request<Body>, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let version = req.version();

    let user_agent = req
        .headers()
        .get(axum::http::header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("-")
        .to_string();

    let start = Instant::now();
    let response = next.run(req).await;
    let latency = start.elapsed();

    tracing::info!(
        "method={}, uri={}, version={:?}, latency={:?}, status={}, user-agent={}",
        method,
        uri,
        version,
        latency,
        response.status(),
        user_agent
    );

    response
}

#[tokio::main]
async fn main() {
    let env = Env::new();

    if args_contains("--init") {
        init::init(&env).await;
    }

    server(env).await;
}

async fn server(env: Env) {
    let port = env.port;
    const HOST: &str = "0.0.0.0";

    tracing_subscriber::registry()
        .with(
            EnvFilter::from_default_env().add_directive(
                format!("transfery={}", env.mode.tracing_level())
                    .parse()
                    .unwrap(),
            ),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("listening on http://{}:{}", HOST, port);

    let storage = get_storage(&env).await;
    let database = get_database(&env).await;

    let secret_key = database.get_secret_key().await.unwrap();
    let crypto = Crypto::new(&secret_key).unwrap();

    let (socketio_layer, socketio) = SocketIo::builder()
        .ping_interval(Duration::from_secs(3))
        .ping_timeout(Duration::from_secs(2))
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
        .nest_service("/static", ServeDir::new("./static"))
        .route(index::INDEX_PATH, get(index::index))
        .route(download::DOWNLOAD_PATH, get(download::download))
        .route(message::PAGE_PATH, get(message::page))
        .route(message::SYNC_PATH, get(message::sync))
        .route(message::NEW_ITEM_PATH, post(message::new_item))
        .route(message::REMOVE_ITEM_PATH, post(message::remove_item))
        .route(message::REMOVE_ALL_PATH, get(message::remove_all))
        .route(upload::FETCH_UPLOAD_ID_PATH, post(upload::fetch_upload_id))
        .route(upload::UPLOAD_PART_PATH, post(upload::upload_part))
        .route(upload::COMPLETE_UPLOAD_PATH, post(upload::complete_upload))
        .route(admin::AUTH_PATH, post(admin::auth))
        .route(admin::AUTO_LOGIN_PATH, get(admin::auto_login))
        .route(admin::SIGN_OUT_PATH, get(admin::sign_out))
        .route(admin::DEVICE_PATH, get(admin::device))
        .route(admin::DEVICE_SIGN_OUT_PATH, post(admin::device_sign_out))
        .route(admin::CREATE_TOKEN_PATH, post(admin::create_token))
        .route(admin::GET_TOKEN_PATH, get(admin::get_token))
        .route(admin::REMOVE_TOKEN_PATH, post(admin::remove_token))
        .route(
            api::PUSH_TEXT_PATH,
            get(api::push_text).post(api::push_text),
        )
        .route(api::LATEST_TEXT_PATH, get(api::latest_text))
        .layer(middleware::from_fn(trace_middleware))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 10)) // 10 MB, must larger than 5 MB for minio
        .layer(socketio_layer)
        .layer(into_layer(socketio))
        .layer(into_layer(env))
        .layer(into_layer(storage))
        .layer(into_layer(database))
        .layer(into_layer(crypto));

    let listener = tokio::net::TcpListener::bind((HOST, port)).await.unwrap();

    axum::serve(listener, router).await.unwrap();
}
