/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use sqlx::mysql::{MySqlConnectOptions, MySqlConnection};
use sqlx::{ConnectOptions, Executor};

use crate::error;

pub struct Database {
    client: MySqlConnection,
}

impl Database {
    pub async fn new(
        endpoint: &str,
        username: &str,
        password: &str,
        database: &str,
    ) -> Result<Self, error::Error> {
        let endpoint_collection = endpoint.split(':').collect::<Vec<&str>>();
        let host = endpoint_collection[0];
        let port = endpoint_collection[1]
            .parse::<u16>()
            .map_err(|_| error::Error::PortParseError("Mysql port parsing failed.".to_string()))?;

        let conn = MySqlConnectOptions::new()
            .host(host)
            .port(port)
            .username(username)
            .password(password)
            .database(database)
            .connect()
            .await
            .map_err(|_| {
                error::Error::DatabaseClientError("Mysql connection failed.".to_string())
            })?;

        Ok(Self { client: conn })
    }
}
