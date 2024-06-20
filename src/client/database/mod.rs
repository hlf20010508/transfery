/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use sea_orm::DatabaseConnection;

mod device;
mod init;
mod message;
pub mod models;
#[cfg(test)]
pub mod tests;
mod token;

#[derive(Debug, Clone)]
pub struct Database {
    connection: DatabaseConnection,
    _name: String,
}
