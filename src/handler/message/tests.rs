/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::response::Response;
use axum::routing::{get, post};
use axum::Router;
use futures::FutureExt;
use rust_socketio::asynchronous::{Client, ClientBuilder};
use rust_socketio::Payload;
use socketioxide::extract::SocketRef;
use socketioxide::socket::Sid;
use socketioxide::SocketIo;
use std::future::Future;
use std::pin::Pin;
use tokio::net::TcpListener;
use tower::ServiceExt;

use super::models::{NewItemData, NewItemParams, RemoveAllParams, RemoveItemParams};
use super::{
    new_item, page, remove_all, remove_item, sync, NEW_ITEM_PATH, PAGE_PATH, REMOVE_ALL_PATH,
    REMOVE_ITEM_PATH, SYNC_PATH,
};

use crate::auth::tests::gen_auth;
use crate::client::database::tests::{get_database, reset as reset_database};
use crate::client::database::{MessageItem, MessageItemType};
use crate::client::storage::tests::{
    get_storage, init as init_storage, reset as reset_storage, upload_data,
};
use crate::client::Database;
use crate::client::Storage;
use crate::crypto::tests::get_crypto;
use crate::env::tests::get_env;
use crate::error::Error::DefaultError;
use crate::error::Result;
use crate::utils::tests::sleep_async;
use crate::utils::{get_current_timestamp, into_layer};

async fn fake_message_item(database: &Database) {
    let item = MessageItem::new_text("fake item for message", get_current_timestamp(), false);

    database.create_table_message_if_not_exists().await.unwrap();
    database.insert_message_item(item).await.unwrap();
}

async fn fake_file(database: &Database, storage: &Storage, file_name: &str) -> Result<()> {
    let item = MessageItem::new_file(
        "fake file for message",
        get_current_timestamp(),
        false,
        file_name,
        true,
    );

    database.create_table_message_if_not_exists().await?;
    database.insert_message_item(item).await?;

    init_storage(storage).await?;
    upload_data(storage, file_name).await?;

    Ok(())
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

fn remove_item_handler(
    payload: Payload,
    _socket: Client,
) -> Pin<Box<(dyn Future<Output = ()> + Send + 'static)>> {
    async move {
        match payload {
            Payload::Text(value) => match value.get(0) {
                Some(value) => {
                    let id = serde_json::from_value::<u64>(value.to_owned()).unwrap();
                    assert_eq!(id, 1);
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
        let env = get_env();

        fake_message_item(&database).await;

        let router = Router::new()
            .route(PAGE_PATH, get(page))
            .layer(into_layer(env))
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
    reset_database(database).await;
    assert_eq!(result.unwrap().status(), StatusCode::OK);

    sleep_async(1).await;
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
    reset_database(database).await;
    assert_eq!(result.unwrap().status(), StatusCode::OK);

    sleep_async(1).await;
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
            .map_err(|e| DefaultError(format!("failed to connect to socketio server: {}", e)))?;

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
    reset_database(database).await;

    let res = result.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.text().await.unwrap(), "2".to_string());

    sleep_async(1).await;
}

#[tokio::test]
async fn test_message_remove_item() {
    async fn inner(database: &Database, storage: &Storage) -> Result<reqwest::Response> {
        let file_name = "test_message_remove_item.txt";

        fake_file(database, storage, file_name).await?;

        let crypto = get_crypto();
        let auth = gen_auth(&crypto);

        let (socketio_layer, socketio) = SocketIo::new_layer();

        socketio.ns("/", |_socket: SocketRef| {});

        let router = Router::new()
            .route(REMOVE_ITEM_PATH, post(remove_item))
            .layer(into_layer(database.clone()))
            .layer(into_layer(storage.clone()))
            .layer(into_layer(crypto))
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
            .on("removeItem", remove_item_handler)
            .connect()
            .await
            .map_err(|e| DefaultError(format!("failed to connect to socketio server: {}", e)))?;

        sleep_async(1).await;

        let data = RemoveItemParams {
            id: 1,
            type_field: MessageItemType::File,
            file_name: Some(file_name.to_string()),
            sid: Sid::new(),
        };

        let client = reqwest::Client::new();
        let res = client
            .post(format!("http://{}{}", addr, REMOVE_ITEM_PATH))
            .json(&data)
            .header("Authorization", auth)
            .send()
            .await
            .map_err(|e| DefaultError(format!("failed to make request: {}", e)))?;

        sleep_async(1).await;

        Ok(res)
    }

    let database = get_database().await;
    let storage = get_storage();

    let result = inner(&database, &storage).await;

    reset_database(database).await;
    reset_storage(&storage).await;

    assert_eq!(result.unwrap().status(), StatusCode::OK);

    sleep_async(1).await;
}

#[tokio::test]
async fn test_message_remove_all() {
    async fn inner(database: &Database, storage: &Storage) -> Result<reqwest::Response> {
        let file_name = "test_message_remove_all.txt";

        fake_file(database, storage, file_name).await?;

        let crypto = get_crypto();
        let auth = gen_auth(&crypto);

        let (socketio_layer, socketio) = SocketIo::new_layer();

        socketio.ns("/", |_socket: SocketRef| {});

        let router = Router::new()
            .route(REMOVE_ALL_PATH, get(remove_all))
            .layer(into_layer(database.clone()))
            .layer(into_layer(storage.clone()))
            .layer(into_layer(crypto))
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
            .on("removeAll", |_payload, _socket| async {}.boxed())
            .connect()
            .await
            .map_err(|e| DefaultError(format!("failed to connect to socketio server: {}", e)))?;

        sleep_async(1).await;

        let data = RemoveAllParams { sid: Sid::new() };

        let client = reqwest::Client::new();
        let res = client
            .get(format!("http://{}{}", addr, REMOVE_ALL_PATH))
            .query(&data)
            .header("Authorization", auth)
            .send()
            .await
            .map_err(|e| DefaultError(format!("failed to make request: {}", e)))?;

        sleep_async(1).await;

        Ok(res)
    }

    let database = get_database().await;
    let storage = get_storage();

    let result = inner(&database, &storage).await;

    reset_database(database).await;
    reset_storage(&storage).await;

    assert_eq!(result.unwrap().status(), StatusCode::OK);

    sleep_async(1).await;
}
