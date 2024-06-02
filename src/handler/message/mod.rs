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
    NewItemData, NewItemParams, PageQueryParams, RemoveAllParams, RemoveItemParams, SyncQueryParams,
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

pub async fn page(
    Extension(env): Extension<Arc<Env>>,
    Extension(database): Extension<Arc<Database>>,
    Query(params): Query<PageQueryParams>,
    AuthState(is_authorized): AuthState,
) -> Result<Json<Vec<MessageItem>>> {
    println!("received new page request");

    let start = params.size;

    let result = database
        .query_message_items(start, env.item_per_page, is_authorized)
        .await?;

    println!("new page pushed");

    // println!("{:#?}", result);

    Ok(Json(result))
}

pub static SYNC_PATH: &str = "/sync";

pub async fn sync(
    Extension(database): Extension<Arc<Database>>,
    Query(params): Query<SyncQueryParams>,
    AuthState(is_authorized): AuthState,
) -> Result<Json<Vec<MessageItem>>> {
    println!("received sync request");

    let latest_id = params.latest_id;

    let result = database
        .query_message_items_after_id(latest_id, is_authorized)
        .await?;

    println!("synced: {:#?}", result);

    Ok(Json(result))
}

pub static NEW_ITEM_PATH: &str = "/newItem";

#[debug_handler]
pub async fn new_item(
    Extension(database): Extension<Arc<Database>>,
    Extension(socketio): Extension<Arc<SocketIo>>,
    Json(item): Json<NewItemParams>,
) -> Result<String> {
    println!("received item: {:#?}", item);

    let sid = item.sid;
    let item_id = database
        .insert_message_item(Result::<MessageItem>::from(&item)?)
        .await?;
    println!("pushed to db");

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

    println!("broadcasted");

    Ok(item_id.to_string())
}

pub static REMOVE_ITEM_PATH: &str = "/removeItem";

pub async fn remove_item(
    _: AuthChecker,
    Extension(database): Extension<Arc<Database>>,
    Extension(storage): Extension<Arc<Storage>>,
    Extension(socketio): Extension<Arc<SocketIo>>,
    Json(item): Json<RemoveItemParams>,
) -> Result<Response> {
    println!("received item to be removed: {:#?}", item);

    let sid = item.sid;

    database.remove_message_item(item.id as i64).await?;

    println!("removed item in db");

    match item.type_field {
        MessageItemType::File => {
            let file_name = item.file_name;

            match file_name {
                Some(file_name) => {
                    storage.remove_object(&file_name).await?;
                    println!("removed item in storage");
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

    println!("broadcasted");

    Ok(StatusCode::OK.into_response())
}

pub static REMOVE_ALL_PATH: &str = "/removeAll";

pub async fn remove_all(
    _: AuthChecker,
    Extension(database): Extension<Arc<Database>>,
    Extension(storage): Extension<Arc<Storage>>,
    Extension(socketio): Extension<Arc<SocketIo>>,
    Query(item): Query<RemoveAllParams>,
) -> Result<Response> {
    println!("received remove all request");

    let sid = item.sid;

    database.remove_message_all().await?;

    println!("removed all in db");

    storage.remove_objects_all().await?;

    println!("removed all in storage");

    socketio
        .to(Room::Public)
        .except(sid)
        .emit("removeItem", ())
        .map_err(|e| SocketEmitError(format!("socketio emit error for event removeAll: {}", e)))?;

    println!("broadcasted");

    Ok(StatusCode::OK.into_response())
}
