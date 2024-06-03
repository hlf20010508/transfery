/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod models;
#[cfg(test)]
mod tests;

use models::ProgressData;
pub use models::{ConnectionNumber, Room};

use socketioxide::extract::{Data, SocketRef, State};

pub fn connect(socket: &SocketRef, connection_number: State<ConnectionNumber>) {
    let sid = socket.id.clone();

    socket.join(sid).ok();
    socket.join(Room::Public).ok();

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
    let sid = socket.id.clone();

    socket.join(sid).ok();
    socket.join(Room::Public).ok();

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

pub static PROGRESS_EVENT: &str = "progress";

pub fn progress(socket: SocketRef, Data::<ProgressData>(data): Data<ProgressData>) {
    socket.broadcast().emit("progress", data).ok();
}
