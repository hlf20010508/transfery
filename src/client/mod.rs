/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

pub mod database;
pub mod storage;

use database::models::config::{Config, MySqlConfig, SqliteConfig};
pub use database::Database;
pub use storage::Storage;

use crate::env::{DatabaseEnv, Env};

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
    match &env.database {
        DatabaseEnv::MySql(env) => {
            let config =
                MySqlConfig::new(&env.endpoint, &env.username, &env.password, &env.database);

            Database::new(Config::MySql(config)).await.unwrap()
        }
        DatabaseEnv::Sqlite(env) => {
            let config = SqliteConfig::new(&env.path);

            Database::new(Config::Sqlite(config)).await.unwrap()
        }
    }
}
