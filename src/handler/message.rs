/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::extract::{Extension, Query};
use axum::{debug_handler, Json};
use serde::{Deserialize, Serialize};
use socketioxide::socket::Sid;
use socketioxide::SocketIo;
use std::sync::Arc;

use crate::auth::AuthState;
use crate::client::database::{MessageItem, MessageItemType};
use crate::client::Database;
use crate::env::ITEM_PER_PAGE;
use crate::error::Error::{FieldParseError, SocketEmitError};
use crate::error::Result;
use crate::handler::socket::Room;

#[derive(Deserialize)]
pub struct PageQueryParams {
    size: u32,
}

#[derive(Deserialize)]
pub struct SyncQueryParams {
    #[serde(rename = "latestId")]
    latest_id: u32,
}

pub static PAGE_PATH: &str = "/page";

pub async fn page(
    Extension(database): Extension<Arc<Database>>,
    Query(params): Query<PageQueryParams>,
    AuthState(is_authorized): AuthState,
) -> Result<Json<Vec<MessageItem>>> {
    println!("received new page request");

    let start = params.size;

    let result = database
        .query_message_items(start, ITEM_PER_PAGE.clone(), is_authorized)
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

#[derive(Deserialize, Serialize, Debug)]
pub struct NewItemParams {
    content: String,
    timestamp: i64,
    #[serde(rename = "isPrivate")]
    is_private: bool,
    #[serde(rename = "fileName")]
    file_name: Option<String>,
    #[serde(rename = "isComplete")]
    is_complete: Option<bool>,
    #[serde(rename = "type")]
    type_field: MessageItemType,
    sid: Sid,
}

impl From<&NewItemParams> for Result<MessageItem> {
    fn from(new_item: &NewItemParams) -> Self {
        match new_item.type_field {
            MessageItemType::Text => Ok(MessageItem::new_text(
                &new_item.content,
                new_item.timestamp,
                new_item.is_private,
            )),
            MessageItemType::File => {
                let file_name = match new_item.file_name.clone() {
                    Some(file_name) => file_name,
                    None => {
                        return Err(FieldParseError(
                            "MessageItem field fileName missed for file type".to_string(),
                        ));
                    }
                };

                let is_complete = match new_item.is_complete {
                    Some(is_complete) => is_complete,
                    None => {
                        return Err(FieldParseError(
                            "MessageItem field isComplete missed for file type".to_string(),
                        ));
                    }
                };

                Ok(MessageItem::new_file(
                    &new_item.content,
                    new_item.timestamp,
                    new_item.is_private,
                    &file_name,
                    is_complete,
                ))
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct NewItemData {
    id: u64,
    content: String,
    timestamp: i64,
    #[serde(rename = "isPrivate")]
    is_private: bool,
    #[serde(rename = "fileName")]
    file_name: Option<String>,
    #[serde(rename = "isComplete")]
    is_complete: Option<bool>,
    #[serde(rename = "type")]
    type_field: MessageItemType,
}

impl From<(u64, NewItemParams)> for NewItemData {
    fn from((id, params): (u64, NewItemParams)) -> Self {
        Self {
            id,
            content: params.content,
            timestamp: params.timestamp,
            is_private: params.is_private,
            file_name: params.file_name,
            is_complete: params.is_complete,
            type_field: params.type_field,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    use axum::body::Body;
    use axum::http::{header, Method, Request, StatusCode};
    use axum::response::Response;
    use axum::routing::{get, post};
    use axum::Router;
    use futures::FutureExt;
    use rust_socketio::asynchronous::{Client, ClientBuilder};
    use rust_socketio::Payload;
    use socketioxide::extract::SocketRef;
    use std::future::Future;
    use std::pin::Pin;
    use tokio::net::TcpListener;
    use tower::ServiceExt;

    use crate::auth::tests::gen_auth;
    use crate::client::database::tests::{get_database, reset};
    use crate::client::database::MessageItem;
    use crate::client::Database;
    use crate::crypto::tests::get_crypto;
    use crate::error::Error::{DefaultError, ToStrError};
    use crate::utils::tests::{sleep_async, ResponseExt};
    use crate::utils::{get_current_timestamp, into_layer};

    async fn fake_message_item(database: &Database) {
        let item = MessageItem::new_text("fake item for message", get_current_timestamp(), false);

        database.create_table_message_if_not_exists().await.unwrap();
        database.insert_message_item(item).await.unwrap();
    }

    fn new_item_handler(
        payload: Payload,
        _socket: Client,
    ) -> Pin<Box<(dyn Future<Output = ()> + Send + 'static)>> {
        async move {
            match payload {
                Payload::Text(value) => match value.get(0) {
                    Some(value) => {
                        let data = serde_json::from_value::<NewItemData>(value.to_owned()).unwrap();
                        println!("{:#?}", data);
                        assert_eq!(
                            data,
                            NewItemData {
                                id: 2,
                                content: "content".to_string(),
                                timestamp: 0,
                                is_private: true,
                                file_name: Some("file name".to_string()),
                                is_complete: Some(true),
                                type_field: MessageItemType::File
                            }
                        );
                    }
                    None => panic!("No new item received"),
                },
                _ => panic!("Unexpected payload type"),
            };
        }
        .boxed()
    }

    #[tokio::test]
    async fn test_message_page() {
        async fn inner(database: &Database) -> Result<Response> {
            let crypto = get_crypto();

            fake_message_item(&database).await;

            let router = Router::new()
                .route(PAGE_PATH, get(page))
                .layer(into_layer(database.clone()))
                .layer(into_layer(crypto.clone()));

            let authorization = gen_auth(&crypto);

            let req = Request::builder()
                .method(Method::GET)
                .uri(format!("{}?size=0", PAGE_PATH))
                .header("Authorization", authorization)
                .body(Body::empty())
                .map_err(|e| DefaultError(format!("failed to build request: {}", e)))?;

            let res = router
                .oneshot(req)
                .await
                .map_err(|e| DefaultError(format!("failed to make request: {}", e)))?;

            Ok(res)
        }

        let database = get_database().await;
        let result = inner(&database).await;
        reset(&database).await;
        assert_eq!(result.unwrap().status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_message_sync() {
        async fn inner(database: &Database) -> Result<Response> {
            let crypto = get_crypto();

            fake_message_item(&database).await;

            let router = Router::new()
                .route(SYNC_PATH, get(sync))
                .layer(into_layer(database.clone()))
                .layer(into_layer(crypto.clone()));

            let authorization = gen_auth(&crypto);

            let req = Request::builder()
                .method(Method::GET)
                .uri(format!("{}?latestId=0", SYNC_PATH))
                .header("Authorization", authorization)
                .body(Body::empty())
                .map_err(|e| DefaultError(format!("failed to build request: {}", e)))?;

            let res = router
                .oneshot(req)
                .await
                .map_err(|e| DefaultError(format!("failed to make request: {}", e)))?;

            Ok(res)
        }

        let database = get_database().await;
        let result = inner(&database).await;
        reset(&database).await;
        assert_eq!(result.unwrap().status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_message_new_item() {
        async fn inner(database: &Database) -> Result<reqwest::Response> {
            fake_message_item(&database).await;

            let (socketio_layer, socketio) = SocketIo::new_layer();

            socketio.ns("/", |_socket: SocketRef| {});

            let router = Router::new()
                .route(NEW_ITEM_PATH, post(new_item))
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
                .on("newItem", new_item_handler)
                .connect()
                .await
                .map_err(|e| {
                    DefaultError(format!("failed to connect to socketio server: {}", e))
                })?;

            sleep_async(1).await;

            let data = NewItemParams {
                content: "content".to_string(),
                timestamp: 0,
                is_private: true,
                file_name: Some("file name".to_string()),
                is_complete: Some(true),
                type_field: MessageItemType::File,
                sid: Sid::new(),
            };

            let client = reqwest::Client::new();
            let res = client
                .post(format!("http://{}{}", addr, NEW_ITEM_PATH))
                .json(&data)
                .send()
                .await
                .map_err(|e| DefaultError(format!("failed to make request: {}", e)))?;

            sleep_async(1).await;

            Ok(res)
        }

        let database = get_database().await;
        let result = inner(&database).await;
        reset(&database).await;

        let res = result.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.text().await.unwrap(), "2".to_string());
    }
}
