/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

pub mod database;
pub mod storage;

pub use database::Database;
pub use storage::Storage;

use crate::env::Env;

pub async fn get_storage(env: &Env) -> Storage {
    Storage::new(&env.storage).await.unwrap()
}

pub async fn get_database(env: &Env) -> Database {
    Database::new(&env.database).await.unwrap()
}
