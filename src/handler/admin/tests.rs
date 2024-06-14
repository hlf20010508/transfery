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
use crate::auth::Authorization;
use crate::client::database::tests::{get_database, reset as reset_database};
use crate::client::database::{Database, DeviceItem, NewTokenItem, TokenItem};
use crate::crypto::tests::get_crypto;
use crate::env::tests::get_env;
use crate::error::Error::DefaultError;
use crate::error::Result;
use crate::handler::admin::models::{
    AutoLoginParams, CreateTokenParams, RemoveTokenParams, SignOutParams,
};
use crate::handler::admin::{
    create_token, get_token, remove_token, sign_out, CREATE_TOKEN_PATH, GET_TOKEN_PATH,
    REMOVE_TOKEN_PATH, SIGN_OUT_PATH,
};
use crate::utils::tests::{sleep_async, ResponseExt};
use crate::utils::{get_current_timestamp, into_layer};

#[tokio::test]
async fn test_admin_auth() {
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
            .on("device", |_payload, _socket| async {}.boxed())
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
async fn test_admin_auto_login() {
    async fn inner(database: &Database) -> Result<reqwest::Response> {
        database.create_table_device_if_not_exists().await?;

        let (socketio_layer, socketio) = SocketIo::new_layer();

        socketio.ns("/", |_socket: SocketRef| {});

        let crypto = get_crypto();
        let auth = gen_auth(&crypto);

        let router = Router::new()
            .route(AUTO_LOGIN_PATH, get(auto_login))
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

        sleep_async(1).await;

        let data = AutoLoginParams { sid: Sid::new() };

        let client = reqwest::Client::new();
        let res = client
            .get(format!("http://{}{}", addr, AUTO_LOGIN_PATH))
            .query(&data)
            .header("Authorization", auth)
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
async fn test_admin_sign_out() {
    async fn inner(database: &Database) -> Result<reqwest::Response> {
        database.create_table_device_if_not_exists().await?;

        let (socketio_layer, socketio) = SocketIo::new_layer();

        socketio.ns("/", |_socket: SocketRef| {});

        let crypto = get_crypto();
        let auth = gen_auth(&crypto);

        let fingerprint = serde_json::from_str::<Authorization>(&auth)
            .map_err(|e| DefaultError(format!("failed to parse authorization: {}", e)))?
            .fingerprint;

        database
            .insert_device(DeviceItem {
                fingerprint,
                browser: Some("browser".to_string()),
                last_use_timestamp: Some(get_current_timestamp()),
                expiration_timestamp: Some(get_current_timestamp() + 1000 * 60),
            })
            .await?;

        let router = Router::new()
            .route(SIGN_OUT_PATH, get(sign_out))
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

        sleep_async(1).await;

        let data = SignOutParams { sid: Sid::new() };

        let client = reqwest::Client::new();
        let res = client
            .get(format!("http://{}{}", addr, SIGN_OUT_PATH))
            .query(&data)
            .header("Authorization", auth)
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
async fn test_admin_device() {
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
async fn test_admin_device_sign_out() {
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

#[tokio::test]
async fn test_admin_create_token() {
    async fn inner(database: &Database) -> Result<reqwest::Response> {
        database.create_table_token_if_not_exists().await?;

        let (socketio_layer, socketio) = SocketIo::new_layer();

        socketio.ns("/", |_socket: SocketRef| {});

        let crypto = get_crypto();
        let auth = gen_auth(&crypto);
        let env = get_env();

        let router = Router::new()
            .route(CREATE_TOKEN_PATH, post(create_token))
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
            .on("token", |_payload, _socket| async {}.boxed())
            .connect()
            .await
            .map_err(|e| DefaultError(format!("failed to connect to socketio server: {}", e)))?;

        sleep_async(1).await;

        let data = CreateTokenParams {
            name: "test name".to_string(),
            expiration_timestamp: get_current_timestamp(),
        };

        let client = reqwest::Client::new();
        let res = client
            .post(format!("http://{}{}", addr, CREATE_TOKEN_PATH))
            .json(&data)
            .header("Authorization", auth)
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
async fn test_admin_get_token() {
    async fn inner(database: &Database, new_token_item: NewTokenItem) -> Result<Response> {
        database.create_table_token_if_not_exists().await?;

        database.insert_token(new_token_item).await?;

        let crypto = get_crypto();
        let auth = gen_auth(&crypto);

        let router = Router::new()
            .route(GET_TOKEN_PATH, get(get_token))
            .layer(into_layer(database.clone()))
            .layer(into_layer(crypto));

        let req = Request::builder()
            .method(Method::GET)
            .uri(GET_TOKEN_PATH)
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

    let new_token_item = NewTokenItem {
        token: "test_token".to_string(),
        name: "test name".to_string(),
        expiration_timestamp: get_current_timestamp(),
    };

    let result = inner(&database, new_token_item.clone()).await;
    reset_database(database).await;

    let result = result.unwrap();
    let status = result.status();
    let body = result.to_string().await.unwrap();
    let tokens = serde_json::from_str::<Vec<TokenItem>>(&body).unwrap();

    assert_eq!(status, StatusCode::OK);
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token, new_token_item.token);
    assert_eq!(tokens[0].name, new_token_item.name);

    sleep_async(1).await;
}

#[tokio::test]
async fn test_admin_remove_token() {
    async fn inner(database: &Database) -> Result<reqwest::Response> {
        database.create_table_token_if_not_exists().await?;

        let new_token_item = NewTokenItem {
            token: "test_token".to_string(),
            name: "test name".to_string(),
            expiration_timestamp: get_current_timestamp(),
        };
        database.insert_token(new_token_item.clone()).await?;

        let (socketio_layer, socketio) = SocketIo::new_layer();

        socketio.ns("/", |_socket: SocketRef| {});

        let crypto = get_crypto();
        let auth = gen_auth(&crypto);

        let router = Router::new()
            .route(REMOVE_TOKEN_PATH, post(remove_token))
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
            .on("token", |_payload, _socket| async {}.boxed())
            .connect()
            .await
            .map_err(|e| DefaultError(format!("failed to connect to socketio server: {}", e)))?;

        sleep_async(1).await;

        let data = RemoveTokenParams {
            token: new_token_item.token,
        };

        let client = reqwest::Client::new();
        let res = client
            .post(format!("http://{}{}", addr, REMOVE_TOKEN_PATH))
            .json(&data)
            .header("Authorization", auth)
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
