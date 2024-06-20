/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod models;
#[cfg(test)]
mod tests;

use axum::extract::Query;
use models::{
    AuthParams, AutoLoginParams, CreateTokenParams, DeviceSignOutParams, RemoveTokenParams,
    SignOutParams, TokenRaw,
};

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{debug_handler, Extension, Json};
use socketioxide::SocketIo;
use std::sync::Arc;

use crate::auth::{AuthChecker, Authorization, Certificate};
use crate::client::database::models::device::{self, DeviceItem, DeviceUpdateItem};
use crate::client::database::models::token::{self, TokenNewItem};
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
            browser: params.browser,
            last_use_timestamp: current_timestamp,
            expiration_timestamp: expiration_timestamp,
        };

        database.insert_device(device_item).await?;

        socketio
            .to(params.sid)
            .join(Room::Private)
            .map_err(|e| SocketEmitError(format!("socketio join private error: {}", e)))?;

        tracing::info!("client {} joined room private", params.sid);

        socketio
            .within(Room::Private)
            .except(params.sid)
            .emit("device", ())
            .map_err(|e| SocketEmitError(format!("socketio emit error for event device: {}", e)))?;

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
    Extension(socketio): Extension<Arc<SocketIo>>,
    Query(AutoLoginParams { sid }): Query<AutoLoginParams>,
) -> Result<Response> {
    tracing::info!("received auto login request");

    socketio
        .to(sid)
        .join(Room::Private)
        .map_err(|e| SocketEmitError(format!("socketio join private error: {}", e)))?;

    tracing::info!("client {} joined room private", sid);

    let device_item = DeviceUpdateItem {
        fingerprint,
        browser: None,
        last_use_timestamp: Some(get_current_timestamp()),
        expiration_timestamp: None,
    };

    database.update_device(device_item).await?;

    Ok(StatusCode::OK.into_response())
}

pub static SIGN_OUT_PATH: &str = "/signOut";

#[debug_handler]
pub async fn sign_out(
    _: AuthChecker,
    Authorization { fingerprint, .. }: Authorization,
    Extension(database): Extension<Arc<Database>>,
    Extension(socketio): Extension<Arc<SocketIo>>,
    Query(SignOutParams { sid }): Query<SignOutParams>,
) -> Result<Response> {
    tracing::info!("received sign out request");

    socketio
        .to(sid)
        .leave(Room::Private)
        .map_err(|e| SocketEmitError(format!("socketio leave private error: {}", e)))?;

    tracing::info!("client {} left room private", sid);

    socketio
        .within(Room::Private)
        .except(sid)
        .emit("device", ())
        .map_err(|e| SocketEmitError(format!("socketio emit error for event device: {}", e)))?;

    database.remove_device(&fingerprint).await?;

    Ok(StatusCode::OK.into_response())
}

pub static DEVICE_PATH: &str = "/device";

#[debug_handler]
pub async fn device(
    _: AuthChecker,
    Extension(database): Extension<Arc<Database>>,
) -> Result<Json<Vec<device::Model>>> {
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

pub static CREATE_TOKEN_PATH: &str = "/createToken";

#[debug_handler]
pub async fn create_token(
    _: AuthChecker,
    Extension(env): Extension<Arc<Env>>,
    Extension(crypto): Extension<Arc<Crypto>>,
    Extension(database): Extension<Arc<Database>>,
    Extension(socketio): Extension<Arc<SocketIo>>,
    Json(params): Json<CreateTokenParams>,
) -> Result<Response> {
    tracing::info!("received create token request");
    tracing::debug!("new token item: {:#?}", params);

    let token = {
        let token_raw = TokenRaw::new(&env.username, &env.password, params.expiration_timestamp);
        let token_json = serde_json::to_string(&token_raw)
            .map_err(|e| ToJsonError(format!("failed to convert token raw to json: {}", e)))?;

        crypto.encrypt(&token_json)?
    };

    let new_token_item = TokenNewItem {
        token,
        name: params.name,
        expiration_timestamp: params.expiration_timestamp,
    };

    database.insert_token(new_token_item).await?;

    socketio
        .to(Room::Public)
        .emit("token", ())
        .map_err(|e| SocketEmitError(format!("socketio emit error for event token: {}", e)))?;

    Ok(StatusCode::OK.into_response())
}

pub static GET_TOKEN_PATH: &str = "/getToken";

#[debug_handler]
pub async fn get_token(
    _: AuthChecker,
    Extension(database): Extension<Arc<Database>>,
) -> Result<Json<Vec<token::Model>>> {
    tracing::info!("received get token request");

    let tokens = database.query_token_items().await?;

    tracing::debug!("tokens: {:#?}", tokens);

    Ok(Json(tokens))
}

pub static REMOVE_TOKEN_PATH: &str = "/removeToken";

#[debug_handler]
pub async fn remove_token(
    _: AuthChecker,
    Extension(database): Extension<Arc<Database>>,
    Extension(socketio): Extension<Arc<SocketIo>>,
    Json(RemoveTokenParams { token }): Json<RemoveTokenParams>,
) -> Result<Response> {
    tracing::info!("received remove token request");
    tracing::debug!("remove token: {}", token);

    database.remove_token(token).await?;

    socketio
        .to(Room::Public)
        .emit("token", ())
        .map_err(|e| SocketEmitError(format!("socketio emit error for event token: {}", e)))?;

    Ok(StatusCode::OK.into_response())
}
