/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod models;
#[cfg(test)]
mod tests;

use models::{
    NewItemData, NewItemParams, NewItemResponse, PageQueryParams, RemoveAllParams,
    RemoveItemParams, SyncQueryParams,
};

use axum::extract::{Extension, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{debug_handler, Json};
use socketioxide::SocketIo;
use std::sync::Arc;

use crate::auth::{AuthChecker, AuthState};
use crate::client::database::{MessageItem, MessageItemType};
use crate::client::{Database, Storage};
use crate::env::Env;
use crate::error::Error::{FieldParseError, SocketEmitError};
use crate::error::Result;
use crate::handler::socket::Room;

pub static PAGE_PATH: &str = "/page";

#[debug_handler]
pub async fn page(
    AuthState(is_authorized): AuthState,
    Extension(env): Extension<Arc<Env>>,
    Extension(database): Extension<Arc<Database>>,
    Query(PageQueryParams { size }): Query<PageQueryParams>,
) -> Result<Json<Vec<MessageItem>>> {
    tracing::info!("received new page request");
    tracing::debug!("page size: {}", size);

    let result = database
        .query_message_items(size, env.item_per_page, is_authorized)
        .await?;

    tracing::info!("new page pushed");
    tracing::debug!("page result: {:#?}", result);

    Ok(Json(result))
}

pub static SYNC_PATH: &str = "/sync";

#[debug_handler]
pub async fn sync(
    AuthState(is_authorized): AuthState,
    Extension(database): Extension<Arc<Database>>,
    Query(SyncQueryParams { latest_id }): Query<SyncQueryParams>,
) -> Result<Json<Vec<MessageItem>>> {
    tracing::info!("received sync request");
    tracing::debug!("sync latest id: {}", latest_id);

    let result = database
        .query_message_items_after_id(latest_id, is_authorized)
        .await?;

    tracing::info!("synced");
    tracing::debug!("sync result: {:#?}", result);

    Ok(Json(result))
}

pub static NEW_ITEM_PATH: &str = "/newItem";

#[debug_handler]
pub async fn new_item(
    Extension(database): Extension<Arc<Database>>,
    Extension(socketio): Extension<Arc<SocketIo>>,
    Json(item): Json<NewItemParams>,
) -> Result<Json<NewItemResponse>> {
    tracing::info!("received new item request");
    tracing::debug!("new item: {:#?}", item);

    let sid = item.sid;
    let item_id = database
        .insert_message_item(Result::<MessageItem>::from(&item)?)
        .await?;

    tracing::info!("pushed to db");
    tracing::debug!("new item id: {}", item_id);

    match item.is_private {
        true => socketio
            .to(Room::Private)
            .except(sid)
            .emit("newItem", NewItemData::from((item_id, item)))
            .map_err(|e| {
                SocketEmitError(format!(
                    "socketio emit error for event newItem private: {}",
                    e
                ))
            })?,
        false => socketio
            .to(Room::Public)
            .except(sid)
            .emit("newItem", NewItemData::from((item_id, item)))
            .map_err(|e| {
                SocketEmitError(format!(
                    "socketio emit error for event newItem public: {}",
                    e
                ))
            })?,
    };

    tracing::info!("broadcasted");

    Ok(Json(NewItemResponse { id: item_id }))
}

pub static REMOVE_ITEM_PATH: &str = "/removeItem";

#[debug_handler]
pub async fn remove_item(
    _: AuthChecker,
    Extension(database): Extension<Arc<Database>>,
    Extension(storage): Extension<Arc<Storage>>,
    Extension(socketio): Extension<Arc<SocketIo>>,
    Json(item): Json<RemoveItemParams>,
) -> Result<Response> {
    tracing::info!("received remove item request");
    tracing::debug!("received item to be removed: {:#?}", item);

    let sid = item.sid;

    database.remove_message_item(item.id as i64).await?;

    tracing::info!("removed item in db");

    match item.type_field {
        MessageItemType::File => {
            let file_name = item.file_name;

            match file_name {
                Some(file_name) => {
                    storage.remove_object(&file_name).await?;
                    tracing::info!("removed item in storage");
                }
                None => {
                    return Err(FieldParseError(
                        "RemoveItemParams field fileName missed for file type".to_string(),
                    ));
                }
            }
        }
        _ => {}
    }

    socketio
        .to(Room::Public)
        .except(sid)
        .emit("removeItem", item.id)
        .map_err(|e| SocketEmitError(format!("socketio emit error for event removeItem: {}", e)))?;

    tracing::info!("broadcasted");

    Ok(StatusCode::OK.into_response())
}

pub static REMOVE_ALL_PATH: &str = "/removeAll";

#[debug_handler]
pub async fn remove_all(
    _: AuthChecker,
    Extension(database): Extension<Arc<Database>>,
    Extension(storage): Extension<Arc<Storage>>,
    Extension(socketio): Extension<Arc<SocketIo>>,
    Query(item): Query<RemoveAllParams>,
) -> Result<Response> {
    tracing::info!("received remove all request");

    let sid = item.sid;

    database.remove_message_all().await?;

    tracing::info!("removed all in db");

    storage.remove_objects_all().await?;

    tracing::info!("removed all in storage");

    socketio
        .to(Room::Public)
        .except(sid)
        .emit("removeAll", ())
        .map_err(|e| SocketEmitError(format!("socketio emit error for event removeAll: {}", e)))?;

    tracing::info!("broadcasted");

    Ok(StatusCode::OK.into_response())
}
