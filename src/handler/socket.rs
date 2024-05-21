/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use socketioxide::extract::{Data, SocketRef};
use socketioxide::socket::Sid;
use std::sync::Mutex;

static CONNECTION_NUMBER: Mutex<usize> = Mutex::new(0);

pub fn connect(socket: SocketRef, Data(sid): Data<Sid>) {
    let connection_number = CONNECTION_NUMBER
        .lock()
        .unwrap()
        .checked_add(1)
        .unwrap()
        .clone();

    println!(
        "client {} connected, connection number {}",
        sid.as_str(),
        connection_number
    );

    socket.emit("connectionNumber", connection_number).ok();
}
