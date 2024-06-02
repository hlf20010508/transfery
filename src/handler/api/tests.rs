/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use futures::FutureExt;
use rust_socketio::asynchronous::ClientBuilder;
use socketioxide::extract::SocketRef;
use socketioxide::SocketIo;
use tokio::net::TcpListener;
use tower::ServiceExt;

use super::models::{Account, PushTextParams, Token};
use super::{latest_text, push_text, LATEST_TEXT_PATH, PUSH_TEXT_PATH};

use crate::client::database::tests::{get_database, reset};
use crate::client::database::MessageItem;
use crate::client::Database;
use crate::crypto::tests::get_crypto;
use crate::env::tests::get_env;
use crate::error::Error::{DefaultError, ToStrError};
use crate::error::Result;
use crate::utils::tests::{sleep_async, ResponseExt};
use crate::utils::{get_current_timestamp, into_layer};

#[tokio::test]
async fn test_api_push_text() {
    async fn inner(database: &Database) -> Result<reqwest::Response> {
        database.create_table_message_if_not_exists().await?;

        let crypto = get_crypto();
        let env = get_env();

        let account = Account {
            username: env.username.clone(),
            password: env.password.clone(),
        };

        let token = Token(
            crypto.encrypt(
                &serde_json::to_string(&account)
                    .map_err(|e| ToStrError(format!("failed to serialize account: {}", e)))?,
            )?,
        );

        let (socketio_layer, socketio) = SocketIo::new_layer();

        socketio.ns("/", |_: SocketRef| {});

        let router = Router::new()
            .route(PUSH_TEXT_PATH, get(push_text).post(push_text))
            .layer(socketio_layer)
            .layer(into_layer(socketio))
            .layer(into_layer(crypto))
            .layer(into_layer(env))
            .layer(into_layer(database.clone()));

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
            .on("newItem", |_, _| async {}.boxed())
            .connect()
            .await
            .map_err(|e| DefaultError(format!("failed to connect to socketio server: {}", e)))?;

        sleep_async(1).await;

        let params = PushTextParams {
            content: "content".to_string(),
            token,
        };

        let client = reqwest::Client::new();
        let res = client
            .get(format!("http://{}{}", addr, PUSH_TEXT_PATH))
            .query(&params)
            .send()
            .await
            .map_err(|e| DefaultError(format!("failed to make request: {}", e)))?;

        sleep_async(1).await;

        Ok(res)
    }

    let database = get_database().await;

    let result = inner(&database).await;
    reset(database).await;

    let result = result.unwrap();
    let status = result.status();
    println!("{}", result.text().await.unwrap());
    assert_eq!(status, reqwest::StatusCode::OK);
}

#[tokio::test]
async fn test_api_latest_text() {
    async fn inner(database: &Database) -> Result<Response> {
        database.create_table_message_if_not_exists().await?;

        let crypto = get_crypto();
        let env = get_env();

        let account = Account {
            username: env.username.clone(),
            password: env.password.clone(),
        };

        let token = Token(
            crypto.encrypt(
                &serde_json::to_string(&account)
                    .map_err(|e| ToStrError(format!("failed to serialize account: {}", e)))?,
            )?,
        );

        let router = Router::new()
            .route(LATEST_TEXT_PATH, get(latest_text))
            .layer(into_layer(database.clone()))
            .layer(into_layer(crypto))
            .layer(into_layer(env));

        let message_item = MessageItem::new_text("content", get_current_timestamp(), true);
        database.insert_message_item(message_item).await?;

        let req = Request::builder()
            .method(Method::GET)
            .uri(format!("{}?token={}", LATEST_TEXT_PATH, token.to_string()))
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
    reset(database).await;

    let result = result.unwrap();
    let status = result.status();
    println!("{}", result.to_string().await.unwrap());
    assert_eq!(status, StatusCode::OK);
}