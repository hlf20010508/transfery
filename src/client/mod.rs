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

pub fn get_storage(env: &Env) -> Storage {
    Storage::new(
        &env.minio_endpoint,
        &env.username,
        &env.password,
        &env.minio_bucket,
    )
    .unwrap()
}

pub async fn get_database(env: &Env) -> Database {
    Database::new(
        &env.mysql_endpoint,
        &env.mysql_username,
        &env.mysql_password,
        &env.mysql_database,
    )
    .await
    .unwrap()
}
