/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use serde::{Deserialize, Serialize};
use socketioxide::operators::RoomParam;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;

pub enum Room {
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
    pub fn increase(&self) -> usize {
        self.0.fetch_add(1, SeqCst) + 1
    }
    pub fn decrease(&self) -> usize {
        self.0.fetch_sub(1, SeqCst) - 1
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct ProgressData {
    pub id: u32,
    pub percentage: u8,
    pub pause: bool,
    #[serde(rename = "isComplete")]
    pub is_complete: bool,
}
