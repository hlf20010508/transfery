/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod models;
#[cfg(test)]
mod tests;

use models::{AuthParams, DeviceSignOutParams};

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{debug_handler, Extension, Json};
use socketioxide::SocketIo;
use std::sync::Arc;

use crate::auth::{AuthChecker, Authorization, Certificate};
use crate::client::database::DeviceItem;
use crate::client::Database;
use crate::crypto::Crypto;
use crate::env::Env;
use crate::error::Error::{SocketEmitError, ToJsonError};
use crate::error::Result;
use crate::handler::socket::Room;
use crate::utils::get_current_timestamp;

pub static AUTH_PATH: &str = "/auth";

#[debug_handler]
pub async fn auth(
    Extension(env): Extension<Arc<Env>>,
    Extension(crypto): Extension<Arc<Crypto>>,
    Extension(database): Extension<Arc<Database>>,
    Extension(socketio): Extension<Arc<SocketIo>>,
    Json(params): Json<AuthParams>,
) -> Result<Response> {
    tracing::info!("received auth request");
    tracing::debug!("auth params: {:#?}", params);

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

            tracing::debug!("certificate: {}", certificate);

            certificate
        };

        let device_item = DeviceItem {
            fingerprint: params.fingerprint,
            browser: Some(params.browser),
            last_use_timestamp: Some(current_timestamp),
            expiration_timestamp: Some(expiration_timestamp),
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

        tracing::info!("broadcasted");

        return Ok(certificate.into_response());
    } else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }
}

pub static AUTO_LOGIN_PATH: &str = "/autoLogin";

#[debug_handler]
pub async fn auto_login(
    _: AuthChecker,
    Authorization { fingerprint, .. }: Authorization,
    Extension(database): Extension<Arc<Database>>,
) -> Result<Response> {
    tracing::info!("received auto login request");
    tracing::debug!("auto login fingerprint: {}", fingerprint);

    let device_item = DeviceItem {
        fingerprint,
        browser: None,
        last_use_timestamp: Some(get_current_timestamp()),
        expiration_timestamp: None,
    };

    database.update_device(device_item).await?;

    Ok(StatusCode::OK.into_response())
}

pub static DEVICE_PATH: &str = "/device";

#[debug_handler]
pub async fn device(
    _: AuthChecker,
    Extension(database): Extension<Arc<Database>>,
) -> Result<Json<Vec<DeviceItem>>> {
    tracing::info!("received device request");

    let device_items = database.query_device_items().await?;

    tracing::debug!("device items: {:#?}", device_items);

    Ok(Json(device_items))
}

pub static DEVICE_SIGN_OUT_PATH: &str = "/deviceSignOut";

#[debug_handler]
pub async fn device_sign_out(
    _: AuthChecker,
    Extension(database): Extension<Arc<Database>>,
    Extension(socketio): Extension<Arc<SocketIo>>,
    Json(DeviceSignOutParams { fingerprint, sid }): Json<DeviceSignOutParams>,
) -> Result<Response> {
    tracing::info!("received device sign out request");
    tracing::debug!("device sign out fingerprint: {}, sid: {}", fingerprint, sid);

    database.remove_device(&fingerprint).await?;

    socketio
        .to(Room::Private)
        .except(sid)
        .emit("signOut", fingerprint)
        .map_err(|e| SocketEmitError(format!("socketio emit error for event signOut: {}", e)))?;

    tracing::info!("broadcasted");

    Ok(StatusCode::OK.into_response())
}
