/*
:project: actix-sockets
:author: Anthony Oleinik
:copyright: (C) 2020 Anthony Oleinik <antholeinik@gmail.com>
*/

use actix::prelude::{Message, Recipient};
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<WsMessage>,
    pub lobby_id: Uuid, // room
    pub self_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: Uuid,
    pub room_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientActorMessage {
    pub id: Uuid,
    pub msg: String,
    pub room_id: Uuid,
}
