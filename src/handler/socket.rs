/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use socketioxide::extract::{SocketRef, State};
use socketioxide::operators::RoomParam;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;

enum Room {
    Public,
    Private,
}

impl ToString for Room {
    fn to_string(&self) -> String {
        match self {
            Room::Public => "public".to_string(),
            Room::Private => "private".to_string(),
        }
    }
}

impl RoomParam for Room {
    type IntoIter = std::iter::Once<socketioxide::adapter::Room>;

    fn into_room_iter(self) -> Self::IntoIter {
        std::iter::once(std::borrow::Cow::Owned(self.to_string()))
    }
}

pub struct ConnectionNumber(pub AtomicUsize);
impl ConnectionNumber {
    pub fn new() -> Self {
        Self(AtomicUsize::new(0))
    }
    fn increase(&self) -> usize {
        self.0.fetch_add(1, SeqCst) + 1
    }
    fn decrease(&self) -> usize {
        self.0.fetch_sub(1, SeqCst) - 1
    }
}

pub fn connect(socket: &SocketRef, connection_number: State<ConnectionNumber>) {
    socket.join(Room::Public).ok();

    let sid = socket.id.clone();

    let connection_number = connection_number.increase();

    println!(
        "client {} connected, connection number {}",
        sid, connection_number
    );

    socket
        .within(Room::Public)
        .emit("connectionNumber", connection_number)
        .ok();
}

pub fn disconnect(socket: SocketRef, connection_number: State<ConnectionNumber>) {
    socket.join(Room::Public).ok();

    let sid = socket.id.clone();

    let connection_number = connection_number.decrease();

    println!(
        "client {} disconnected, connection number {}",
        sid, connection_number
    );

    socket
        .within(Room::Public)
        .emit("connectionNumber", connection_number)
        .ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::Router;
    use futures::FutureExt;
    use rust_socketio::asynchronous::{Client, ClientBuilder};
    use rust_socketio::Payload;
    use socketioxide::SocketIo;
    use std::future::Future;
    use std::pin::Pin;
    use tokio::net::TcpListener;

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
}
