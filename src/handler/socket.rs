/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use socketioxide::extract::SocketRef;
use std::sync::Mutex;

struct MutexNumber(Mutex<usize>);

impl MutexNumber {
    const fn new(number: usize) -> Self {
        Self(Mutex::new(number))
    }

    fn get(&self) -> usize {
        *self
            .0
            .lock()
            .unwrap_or_else(|e| panic!("failed to acquire lock for MutexNumber in get: {}", e))
    }

    fn increase(&self) {
        let mut number = self.0.lock().unwrap_or_else(|e| {
            panic!("failed to acquire lock for MutexNumber in increase: {}", e)
        });

        *number = number
            .checked_add(1)
            .unwrap_or_else(|| panic!("MutexNumber overflowed in increase"));
    }

    fn decrease(&self) {
        let mut number = self.0.lock().unwrap_or_else(|e| {
            panic!("failed to acquire lock for MutexNumber in decrease: {}", e)
        });

        *number = number
            .checked_add_signed(-1)
            .unwrap_or_else(|| panic!("MutexNumber overflowed in decrease"));
    }
}

static CONNECTION_NUMBER: MutexNumber = MutexNumber::new(0);

pub fn connect(socket: &SocketRef) {
    let sid = socket.id.clone();

    CONNECTION_NUMBER.increase();
    let connection_number = CONNECTION_NUMBER.get();

    println!(
        "client {} connected, connection number {}",
        sid, connection_number
    );

    socket.emit("connectionNumber", connection_number).ok();
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

    fn connection_number(
        payload: Payload,
        _socket: Client,
    ) -> Pin<Box<(dyn Future<Output = ()> + Send + 'static)>> {
        async move {
            match payload {
                Payload::Text(value) => match value.get(0) {
                    Some(value) => match value {
                        serde_json::Value::Number(number) => {
                            assert_eq!(number.as_u64(), Some(1));
                        }
                        _ => panic!("Unexpected payload value type"),
                    },
                    None => panic!("No connection number received"),
                },
                _ => panic!("Unexpected payload type"),
            };
        }
        .boxed()
    }

    #[tokio::test]
    async fn test_socket_connect() {
        let (socketio_layer, socketio) = SocketIo::new_layer();

        socketio.ns("/", |socket: SocketRef| {
            connect(&socket);
        });

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

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        socket
            .disconnect()
            .await
            .unwrap_or_else(|e| panic!("Disconnect failed: {}", e));
    }
}
