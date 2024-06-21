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

use crate::client::database::models::message::{self, MessageItem};
use crate::client::Database;
use crate::crypto::Crypto;
use crate::env::Env;
use crate::error::Error;
use crate::error::ErrorType::InternalServerError;
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
        let account = Account::from(&params.token, &crypto)?;
        account.is_valid(&env)
    };

    if is_valid {
        database
            .update_token(&params.token, get_current_timestamp())
            .await?;

        if !params.content.trim().is_empty() {
            let message_item =
                MessageItem::new_text(&params.content, get_current_timestamp(), true);

            let id = database.insert_message_item(message_item.clone()).await?;

            socketio
                .to(Room::Private)
                .emit("newItem", message::Model::from((id, message_item)))
                .map_err(|e| {
                    Error::context(InternalServerError, e, "failed to emit event newItem")
                })?;

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
        let account = Account::from(&token, &crypto)?;
        account.is_valid(&env)
    };

    if is_valid {
        match database.query_message_latest().await? {
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
