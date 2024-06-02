/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::response::Response;
use axum::routing::{get, post};
use axum::Router;
use futures::FutureExt;
use rust_socketio::asynchronous::ClientBuilder;
use socketioxide::extract::SocketRef;
use socketioxide::socket::Sid;
use socketioxide::SocketIo;
use tokio::net::TcpListener;
use tower::ServiceExt;

use super::models::{AuthParams, DeviceSignOutParams};
use super::{
    auth, auto_login, device, device_sign_out, AUTH_PATH, AUTO_LOGIN_PATH, DEVICE_PATH,
    DEVICE_SIGN_OUT_PATH,
};

use crate::auth::tests::gen_auth;
use crate::client::database::tests::{get_database, reset as reset_database};
use crate::client::database::{Database, DeviceItem};
use crate::crypto::tests::get_crypto;
use crate::env::tests::get_env;
use crate::error::Error::DefaultError;
use crate::error::Result;
use crate::utils::tests::sleep_async;
use crate::utils::{get_current_timestamp, into_layer};

#[tokio::test]
async fn test_login_auth() {
    async fn inner(database: &Database) -> Result<reqwest::Response> {
        database.create_table_device_if_not_exists().await?;

        let (socketio_layer, socketio) = SocketIo::new_layer();

        socketio.ns("/", |_socket: SocketRef| {});

        let crypto = get_crypto();
        let env = get_env();

        let router = Router::new()
            .route(AUTH_PATH, post(auth))
            .layer(into_layer(env.clone()))
            .layer(into_layer(crypto))
            .layer(into_layer(database.clone()))
            .layer(socketio_layer)
            .layer(into_layer(socketio));

        let server = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| DefaultError(format!("failed to create tcp listener: {}", e)))?;
        let addr = server
            .local_addr()
            .map_err(|e| DefaultError(format!("failed to get local address: {}", e)))?;

        tokio::spawn(async move {
            axum::serve(server, router).await.unwrap();
        });

        ClientBuilder::new(format!("http://{}/", addr))
            .on("signIn", |_payload, _socket| async {}.boxed())
            .connect()
            .await
            .map_err(|e| DefaultError(format!("failed to connect to socketio server: {}", e)))?;

        sleep_async(1).await;

        let data = AuthParams {
            username: env.username,
            password: env.password,
            remember_me: true,
            fingerprint: "fingerprint".to_string(),
            browser: "browser".to_string(),
            sid: Sid::new(),
        };

        let client = reqwest::Client::new();
        let res = client
            .post(format!("http://{}{}", addr, AUTH_PATH))
            .json(&data)
            .send()
            .await
            .map_err(|e| DefaultError(format!("failed to make request: {}", e)))?;

        sleep_async(1).await;

        Ok(res)
    }

    let database = get_database().await;
    let result = inner(&database).await;
    reset_database(database).await;

    assert_eq!(result.unwrap().status(), reqwest::StatusCode::OK);

    sleep_async(1).await;
}

#[tokio::test]
async fn test_login_login() {
    async fn inner(database: &Database) -> Result<Response> {
        database.create_table_device_if_not_exists().await?;

        let crypto = get_crypto();
        let auth = gen_auth(&crypto);

        let router = Router::new()
            .route(AUTO_LOGIN_PATH, get(auto_login))
            .layer(into_layer(database.clone()))
            .layer(into_layer(crypto));

        let req = Request::builder()
            .method(Method::GET)
            .uri(AUTO_LOGIN_PATH)
            .header("Authorization", auth)
            .body(Body::empty())
            .map_err(|e| DefaultError(format!("failed to create request: {}", e)))?;

        let res = router
            .oneshot(req)
            .await
            .map_err(|e| DefaultError(format!("failed to make request: {}", e)))?;

        Ok(res)
    }

    let database = get_database().await;

    let result = inner(&database).await;
    reset_database(database).await;

    assert_eq!(result.unwrap().status(), reqwest::StatusCode::OK);

    sleep_async(1).await;
}

#[tokio::test]
async fn test_login_device() {
    async fn inner(database: &Database) -> Result<Response> {
        let device_item = DeviceItem {
            fingerprint: "fingerprint".to_string(),
            browser: Some("browser".to_string()),
            last_use_timestamp: Some(get_current_timestamp()),
            expiration_timestamp: Some(get_current_timestamp()),
        };

        database.create_table_device_if_not_exists().await?;
        database.insert_device(device_item).await?;

        let crypto = get_crypto();
        let auth = gen_auth(&crypto);

        let router = Router::new()
            .route(DEVICE_PATH, get(device))
            .layer(into_layer(database.clone()))
            .layer(into_layer(crypto));

        let req = Request::builder()
            .method(Method::GET)
            .uri(DEVICE_PATH)
            .header("Authorization", auth)
            .body(Body::empty())
            .map_err(|e| DefaultError(format!("failed to create request: {}", e)))?;

        let res = router
            .oneshot(req)
            .await
            .map_err(|e| DefaultError(format!("failed to make request: {}", e)))?;

        Ok(res)
    }

    let database = get_database().await;

    let result = inner(&database).await;
    reset_database(database).await;

    assert_eq!(result.unwrap().status(), StatusCode::OK);

    sleep_async(1).await;
}

#[tokio::test]
async fn test_login_device_sign_out() {
    async fn inner(database: &Database) -> Result<reqwest::Response> {
        database.create_table_device_if_not_exists().await?;

        let (socketio_layer, socketio) = SocketIo::new_layer();

        socketio.ns("/", |_socket: SocketRef| {});

        let crypto = get_crypto();
        let auth = gen_auth(&crypto);

        let router = Router::new()
            .route(DEVICE_SIGN_OUT_PATH, post(device_sign_out))
            .layer(into_layer(crypto))
            .layer(into_layer(database.clone()))
            .layer(socketio_layer)
            .layer(into_layer(socketio));

        let server = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| DefaultError(format!("failed to create tcp listener: {}", e)))?;
        let addr = server
            .local_addr()
            .map_err(|e| DefaultError(format!("failed to get local address: {}", e)))?;

        tokio::spawn(async move {
            axum::serve(server, router).await.unwrap();
        });

        ClientBuilder::new(format!("http://{}/", addr))
            .on("signOut", |_payload, _socket| async {}.boxed())
            .connect()
            .await
            .map_err(|e| DefaultError(format!("failed to connect to socketio server: {}", e)))?;

        sleep_async(1).await;

        let data = DeviceSignOutParams {
            fingerprint: "fingerprint".to_string(),
            sid: Sid::new(),
        };

        let client = reqwest::Client::new();
        let res = client
            .post(format!("http://{}{}", addr, DEVICE_SIGN_OUT_PATH))
            .header("Authorization", auth)
            .json(&data)
            .send()
            .await
            .map_err(|e| DefaultError(format!("failed to make request: {}", e)))?;

        sleep_async(1).await;

        Ok(res)
    }

    let database = get_database().await;
    let result = inner(&database).await;
    reset_database(database).await;

    assert_eq!(result.unwrap().status(), StatusCode::OK);

    sleep_async(1).await;
}
