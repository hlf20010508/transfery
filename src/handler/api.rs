/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use std::sync::Arc;

use axum::extract::{FromRequest, Query, Request};
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{async_trait, Extension, Json};
use serde::{Deserialize, Serialize};
use socketioxide::SocketIo;

use crate::client::database::MessageItem;
use crate::client::Database;
use crate::crypto::Crypto;
use crate::env::Env;
use crate::error::Error::{
    self, FieldParseError, FromRequestError, SocketEmitError, UnauthorizedError,
};
use crate::error::Result;
use crate::utils::get_current_timestamp;

use super::socket::Room;

#[derive(Debug, Serialize, Deserialize)]
pub struct Token(String);

impl Token {
    fn to_string(self) -> String {
        self.0
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PushTextParams {
    content: String,
    token: Token,
}

#[async_trait]
impl<S> FromRequest<S> for PushTextParams
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request(req: Request, state: &S) -> Result<Self> {
        let method = req.method().clone();

        let data = match method {
            Method::GET => {
                let Query(data) = Query::<PushTextParams>::from_request(req, state)
                    .await
                    .map_err(|e| {
                        FromRequestError(format!("failed to parse query for PushTextParams: {}", e))
                    })?;

                data
            }
            Method::POST => {
                let Json(data) = Json::<PushTextParams>::from_request(req, state)
                    .await
                    .map_err(|e| {
                        FromRequestError(format!("failed to parse json for PushTextParams: {}", e))
                    })?;

                data
            }
            _ => {
                return Err(FromRequestError(
                    "unsupported method for PushTextParams".to_string(),
                ))
            }
        };

        Ok(data)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Account {
    username: String,
    password: String,
}

impl Account {
    fn from(token: Token, crypto: &Crypto) -> Result<Self> {
        let account_json = crypto
            .decrypt(&token.to_string())
            .map_err(|e| UnauthorizedError(format!("failed to decrypt token: {}", e)))?;

        let account = serde_json::from_str::<Account>(&account_json)
            .map_err(|e| FieldParseError(format!("failed to parse account from token: {}", e)))?;

        Ok(account)
    }

    fn is_valid(&self, env: &Env) -> bool {
        self.username == env.username && self.password == env.password
    }
}

pub static PUSH_TEXT_PATH: &str = "/pushText";

pub async fn push_text(
    Extension(crypto): Extension<Arc<Crypto>>,
    Extension(env): Extension<Arc<Env>>,
    Extension(database): Extension<Arc<Database>>,
    Extension(socketio): Extension<Arc<SocketIo>>,
    PushTextParams { content, token }: PushTextParams,
) -> Result<Response> {
    let is_valid = {
        let account = Account::from(token, &crypto)?;
        account.is_valid(&env)
    };

    if is_valid {
        if !content.trim().is_empty() {
            let mut message_item = MessageItem::new_text(&content, get_current_timestamp(), true);

            let id = database.insert_message_item(message_item.clone()).await?;

            message_item.id = Some(id as i64);

            socketio
                .to(Room::Private)
                .emit("newItem", message_item)
                .map_err(|e| SocketEmitError(format!("failed to emit newItem: {}", e)))?;

            println!("text pushed");

            Ok(StatusCode::OK.into_response())
        } else {
            Ok(StatusCode::NOT_ACCEPTABLE.into_response())
        }
    } else {
        Ok(StatusCode::UNAUTHORIZED.into_response())
    }
}

#[derive(Debug, Deserialize)]
pub struct LatestTextParams {
    token: Token,
}

pub static LATEST_TEXT_PATH: &str = "/latestText";

pub async fn latest_text(
    Extension(database): Extension<Arc<Database>>,
    Extension(crypto): Extension<Arc<Crypto>>,
    Extension(env): Extension<Arc<Env>>,
    Query(LatestTextParams { token }): Query<LatestTextParams>,
) -> Result<Response> {
    println!("received get latest text request");

    let is_valid = {
        let account = Account::from(token, &crypto)?;
        account.is_valid(&env)
    };

    if is_valid {
        match database.query_message_latest().await {
            Some(item) => Ok(item.content.into_response()),
            None => Ok(StatusCode::NOT_FOUND.into_response()),
        }
    } else {
        Ok(StatusCode::UNAUTHORIZED.into_response())
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Method, Request};
    use axum::routing::get;
    use axum::Router;
    use futures::FutureExt;
    use rust_socketio::asynchronous::ClientBuilder;
    use socketioxide::extract::SocketRef;
    use tokio::net::TcpListener;
    use tower::ServiceExt;

    use crate::client::database::tests::{get_database, reset};
    use crate::crypto::tests::get_crypto;
    use crate::env::tests::get_env;
    use crate::error::Error::{DefaultError, ToStrError};
    use crate::error::Result;
    use crate::utils::into_layer;
    use crate::utils::tests::{sleep_async, ResponseExt};

    use super::*;

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
                crypto
                    .encrypt(&serde_json::to_string(&account).map_err(|e| {
                        ToStrError(format!("failed to serialize account: {}", e))
                    })?)?,
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
                .map_err(|e| {
                    DefaultError(format!("failed to connect to socketio server: {}", e))
                })?;

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
        assert_eq!(status, StatusCode::OK);
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
                crypto
                    .encrypt(&serde_json::to_string(&account).map_err(|e| {
                        ToStrError(format!("failed to serialize account: {}", e))
                    })?)?,
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
}
