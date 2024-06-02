/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod device;
mod init;
mod message;
mod models;
#[cfg(test)]
pub mod tests;

use sqlx::mysql::MySql;
use sqlx::pool::Pool;

pub use models::{DeviceItem, MessageItem, MessageItemType};

#[derive(Debug, Clone)]
pub struct Database {
    pool: Pool<MySql>,
    _name: String,
}
