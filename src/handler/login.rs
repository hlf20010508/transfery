/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use socketioxide::socket::Sid;
use socketioxide::SocketIo;

use crate::client::database::DeviceItem;
use crate::client::Database;
use crate::crypto::Crypto;
use crate::env::Env;
use crate::error::Error::{SocketEmitError, ToJsonError};
use crate::error::Result;
use crate::handler::socket::Room;
use crate::utils::get_current_timestamp;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthParams {
    username: String,
    password: String,
    #[serde(rename = "rememberMe")]
    remember_me: bool,
    fingerprint: String,
    browser: String,
    sid: Sid,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Certificate {
    fingerprint: String,
    timestamp: i64,
}

pub static AUTH_PATH: &str = "/auth";

pub async fn auth(
    Extension(env): Extension<Arc<Env>>,
    Extension(crypto): Extension<Arc<Crypto>>,
    Extension(database): Extension<Arc<Database>>,
    Extension(socketio): Extension<Arc<SocketIo>>,
    Json(params): Json<AuthParams>,
) -> Result<Response> {
    println!("received auth request");

    if params.username == env.username && params.password == env.password {
        let max_age = if params.remember_me {
            1000 * 3600 * 24 * 365 // 1 year
        } else {
            1000 * 60 * 5 // 5 minutes
        };

        let current_timestamp = get_current_timestamp();
        let expiration_timestamp = current_timestamp + max_age;

        let certificate = {
            let certificate = Certificate {
                fingerprint: params.fingerprint.clone(),
                timestamp: expiration_timestamp,
            };

            let certificate_raw = serde_json::to_string(&certificate).map_err(|e| {
                ToJsonError(format!("Failed to convert certificate to json: {}", e))
            })?;

            let certificate = crypto.encrypt(&certificate_raw)?;

            certificate
        };

        let device_item = DeviceItem {
            fingerprint: params.fingerprint,
            browser: params.browser,
            last_use_timestamp: current_timestamp,
            expiration_timestamp,
        };

        database.insert_device(device_item).await?;

        socketio
            .to(params.sid)
            .join(Room::Private)
            .map_err(|e| SocketEmitError(format!("socketio join private error: {}", e)))?;

        socketio
            .within(Room::Private)
            .except(params.sid)
            .emit("signIn", ())
            .map_err(|e| SocketEmitError(format!("socketio emit error for event signIn: {}", e)))?;

        println!("broadcasted");

        return Ok(certificate.into_response());
    } else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }
}

#[cfg(test)]
mod tests {
    use axum::routing::post;
    use axum::Router;
    use futures::FutureExt;
    use rust_socketio::asynchronous::ClientBuilder;
    use socketioxide::extract::SocketRef;
    use tokio::net::TcpListener;

    use crate::client::database::tests::{get_database, reset as reset_database};
    use crate::crypto::tests::get_crypto;
    use crate::env::tests::get_env;
    use crate::error::Error::DefaultError;
    use crate::error::Result;
    use crate::utils::into_layer;
    use crate::utils::tests::sleep_async;

    use super::*;

    #[tokio::test]
    async fn test_login_auth() {
        async fn inner(database: &Database, env: Env) -> Result<reqwest::Response> {
            database.create_table_device_if_not_exists().await?;

            let (socketio_layer, socketio) = SocketIo::new_layer();

            socketio.ns("/", |_socket: SocketRef| {});

            let crypto = get_crypto();

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
                .map_err(|e| {
                    DefaultError(format!("failed to connect to socketio server: {}", e))
                })?;

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
        let env = get_env();
        let result = inner(&database, env).await;
        reset_database(database).await;

        assert_eq!(result.unwrap().status(), StatusCode::OK);

        sleep_async(1).await;
    }
}
