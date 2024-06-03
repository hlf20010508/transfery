/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::Router;
use futures::FutureExt;
use rust_socketio::asynchronous::{Client, ClientBuilder};
use rust_socketio::Payload;
use socketioxide::extract::{SocketRef, State};
use socketioxide::SocketIo;
use std::future::Future;
use std::pin::Pin;
use tokio::net::TcpListener;

use super::models::ProgressData;
use super::{connect, disconnect, progress, ConnectionNumber, PROGRESS_EVENT};

use crate::utils::tests::sleep_async;

fn connection_number(
    payload: Payload,
    _socket: Client,
) -> Pin<Box<(dyn Future<Output = ()> + Send + 'static)>> {
    async move {
        match payload {
            Payload::Text(value) => match value.get(0) {
                Some(value) => {
                    let _number = serde_json::from_value::<u64>(value.to_owned()).unwrap();
                    // assert_eq!(_number, 1)
                }
                None => panic!("No connection number received"),
            },
            _ => panic!("Unexpected payload type"),
        };
    }
    .boxed()
}

fn progress_handler(
    payload: Payload,
    _socket: Client,
) -> Pin<Box<(dyn Future<Output = ()> + Send + 'static)>> {
    async move {
        match payload {
            Payload::Text(value) => match value.get(0) {
                Some(value) => {
                    let data = serde_json::from_value::<ProgressData>(value.to_owned()).unwrap();
                    println!("{:#?}", data);
                    assert_eq!(
                        data,
                        ProgressData {
                            id: 1,
                            percentage: 100,
                            pause: false,
                            is_complete: true
                        }
                    );
                }
                _ => panic!("No progress data received"),
            },
            _ => panic!("Unexpected payload type"),
        };
    }
    .boxed()
}

#[tokio::test]
async fn test_socket_connect() {
    let (socketio_layer, socketio) = SocketIo::builder()
        .with_state(ConnectionNumber::new())
        .build_layer();

    socketio.ns(
        "/",
        |socket: SocketRef, connection_number: State<ConnectionNumber>| {
            connect(&socket, connection_number);
        },
    );

    let router = Router::new().layer(socketio_layer);

    let server = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = server.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(server, router).await.unwrap();
    });

    let socket = ClientBuilder::new(format!("http://{}/", addr))
        .on("connectionNumber", connection_number)
        .connect()
        .await
        .unwrap_or_else(|e| panic!("Connection failed: {}", e));

    sleep_async(1).await;

    socket
        .disconnect()
        .await
        .unwrap_or_else(|e| panic!("Disconnect failed: {}", e));

    sleep_async(1).await;
}

#[tokio::test]
async fn test_socket_disconnect() {
    let (socketio_layer, socketio) = SocketIo::builder()
        .with_state(ConnectionNumber::new())
        .build_layer();

    socketio.ns(
        "/",
        |socket: SocketRef, connection_number: State<ConnectionNumber>| {
            connect(&socket, connection_number);
            socket.on_disconnect(disconnect);
        },
    );

    let router = Router::new().layer(socketio_layer);

    let server = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = server.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(server, router).await.unwrap();
    });

    let socket = ClientBuilder::new(format!("http://{}/", addr))
        .on("connectionNumber", connection_number)
        .connect()
        .await
        .unwrap_or_else(|e| panic!("Connection failed: {}", e));

    sleep_async(1).await;

    socket
        .disconnect()
        .await
        .unwrap_or_else(|e| panic!("Disconnect failed: {}", e));

    sleep_async(1).await;
}

#[tokio::test]
async fn test_socket_progress() {
    let (socketio_layer, socketio) = SocketIo::new_layer();

    socketio.ns("/", |socket: SocketRef| socket.on(PROGRESS_EVENT, progress));

    let router = Router::new().layer(socketio_layer);

    let server = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = server.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(server, router).await.unwrap();
    });

    let socket_sender = ClientBuilder::new(format!("http://{}/", addr))
        .connect()
        .await
        .unwrap_or_else(|e| panic!("Connection failed: {}", e));

    let socket_receiver = ClientBuilder::new(format!("http://{}/", addr))
        .on("progress", progress_handler)
        .connect()
        .await
        .unwrap_or_else(|e| panic!("Connection failed: {}", e));

    sleep_async(1).await;

    let data = ProgressData {
        id: 1,
        percentage: 100,
        pause: false,
        is_complete: true,
    };

    let data_json = serde_json::to_string(&data).unwrap();

    socket_sender.emit(PROGRESS_EVENT, data_json).await.unwrap();

    sleep_async(1).await;

    socket_sender
        .disconnect()
        .await
        .unwrap_or_else(|e| panic!("Disconnect failed: {}", e));

    socket_receiver
        .disconnect()
        .await
        .unwrap_or_else(|e| panic!("Disconnect failed: {}", e));

    sleep_async(1).await;
}
