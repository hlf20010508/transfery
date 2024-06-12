/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod models;
#[cfg(test)]
mod tests;

use models::{Account, LatestTextParams, PushTextParams};

use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{debug_handler, Extension};
use socketioxide::SocketIo;
use std::sync::Arc;

use super::socket::Room;

use crate::client::database::MessageItem;
use crate::client::Database;
use crate::crypto::Crypto;
use crate::env::Env;
use crate::error::Error::SocketEmitError;
use crate::error::Result;
use crate::utils::get_current_timestamp;

pub static PUSH_TEXT_PATH: &str = "/pushText";

#[debug_handler]
pub async fn push_text(
    Extension(crypto): Extension<Arc<Crypto>>,
    Extension(env): Extension<Arc<Env>>,
    Extension(database): Extension<Arc<Database>>,
    Extension(socketio): Extension<Arc<SocketIo>>,
    params: PushTextParams,
) -> Result<Response> {
    tracing::info!("received push text request");
    tracing::debug!("push text params: {:#?}", params);

    let is_valid = {
        let account = Account::from(params.token, &crypto)?;
        account.is_valid(&env)
    };

    if is_valid {
        if !params.content.trim().is_empty() {
            let mut message_item =
                MessageItem::new_text(&params.content, get_current_timestamp(), true);

            let id = database.insert_message_item(message_item.clone()).await?;

            message_item.id = Some(id as i64);

            socketio
                .to(Room::Private)
                .emit("newItem", message_item)
                .map_err(|e| SocketEmitError(format!("failed to emit newItem: {}", e)))?;

            tracing::info!("text uploaded");

            Ok(StatusCode::OK.into_response())
        } else {
            Ok(StatusCode::NOT_ACCEPTABLE.into_response())
        }
    } else {
        Ok(StatusCode::UNAUTHORIZED.into_response())
    }
}

pub static LATEST_TEXT_PATH: &str = "/latestText";

#[debug_handler]
pub async fn latest_text(
    Extension(database): Extension<Arc<Database>>,
    Extension(crypto): Extension<Arc<Crypto>>,
    Extension(env): Extension<Arc<Env>>,
    Query(LatestTextParams { token }): Query<LatestTextParams>,
) -> Result<Response> {
    tracing::info!("received get latest text request");

    let is_valid = {
        let account = Account::from(token, &crypto)?;
        account.is_valid(&env)
    };

    if is_valid {
        match database.query_message_latest().await {
            Some(item) => {
                tracing::info!("latest text pushed");
                tracing::debug!("latest text item: {:#?}", item);
                Ok(item.content.into_response())
            }
            None => Ok(StatusCode::NOT_FOUND.into_response()),
        }
    } else {
        Ok(StatusCode::UNAUTHORIZED.into_response())
    }
}
