/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use base64::engine::general_purpose::URL_SAFE as base64;
use base64::Engine;
use ring::rand::{SecureRandom, SystemRandom};
use sqlx::mysql::{MySql, MySqlConnectOptions, MySqlConnection};
use sqlx::{ConnectOptions, Executor, Row};

use crate::env::{MYSQL_TABLE_AUTH, MYSQL_TABLE_DEVICE, MYSQL_TABLE_MESSAGE};
use crate::error::Error::{
    DatabaseClientError, PortParseError, SecretKeyGenError, SqlExecuteError, SqlGetValueError,
    SqlQueryError,
};
use crate::error::Result;

pub struct Database {
    conn: MySqlConnection,
}

impl Database {
    pub async fn new(
        endpoint: &str,
        username: &str,
        password: &str,
        database: &str,
    ) -> Result<Self> {
        let endpoint_collection = endpoint.split(':').collect::<Vec<&str>>();
        let host = endpoint_collection[0];
        let port = endpoint_collection[1]
            .parse::<u16>()
            .map_err(|e| PortParseError(format!("MySql port parsing failed: {}", e.to_string())))?;

        let conn = MySqlConnectOptions::new()
            .host(host)
            .port(port)
            .username(username)
            .password(password)
            .connect()
            .await
            .map_err(|e| {
                DatabaseClientError(format!("MySql connection failed: {}", e.to_string()))
            })?;

        let mut database_instance = Self { conn };

        database_instance
            .create_database_if_not_exists(database)
            .await?;
        database_instance.set_database(database).await?;

        Ok(database_instance)
    }

    async fn create_database_if_not_exists(&mut self, database: &str) -> Result<()> {
        let sql = format!("create database if not exists `{}`", database);
        let query = sqlx::query::<MySql>(&sql);

        self.conn.execute(query).await.map_err(|e| {
            SqlExecuteError(format!("MySql create database failed: {}", e.to_string()))
        })?;

        Ok(())
    }

    async fn set_database(&mut self, database: &str) -> Result<()> {
        let sql = format!("use `{}`", database);
        let query = sqlx::query::<MySql>(&sql);

        self.conn.execute(query).await.map_err(|e| {
            SqlExecuteError(format!("MySql set database failed: {}", e.to_string()))
        })?;

        Ok(())
    }

    pub async fn init(&mut self) -> Result<()> {
        self.create_table_message_if_not_exists().await?;
        self.create_table_auth_if_not_exists().await?;
        self.create_table_device_if_not_exists().await?;
        self.create_secret_key_if_not_exists().await?;

        Ok(())
    }

    async fn create_table_message_if_not_exists(&mut self) -> Result<()> {
        let sql = format!(
            "create table if not exists `{}`(
                id int primary key auto_increment,
                content text not null,
                timestamp bigint not null,
                isPrivate tinyint not null,
                type varchar(5) not null,
                fileName text,
                isComplete tinyint
            )",
            MYSQL_TABLE_MESSAGE
        );
        let query = sqlx::query::<MySql>(&sql);

        self.conn.execute(query).await.map_err(|e| {
            SqlExecuteError(format!(
                "MySql create table message failed: {}",
                e.to_string()
            ))
        })?;

        Ok(())
    }

    async fn create_table_auth_if_not_exists(&mut self) -> Result<()> {
        let sql = format!(
            "create table if not exists `{}`(
                id int primary key auto_increment,
                secretKey text not null
            )",
            MYSQL_TABLE_AUTH
        );
        let query = sqlx::query::<MySql>(&sql);

        self.conn.execute(query).await.map_err(|e| {
            SqlExecuteError(format!("MySql create table auth failed: {}", e.to_string()))
        })?;

        Ok(())
    }

    async fn create_table_device_if_not_exists(&mut self) -> Result<()> {
        let sql = format!(
            "create table if not exists `{}`(
                id int primary key auto_increment,
                fingerprint text not null unique,
                browser text not null,
                lastUseTimestamp bigint not null,
                expirationTimestamp bigint not null
            )",
            MYSQL_TABLE_DEVICE
        );
        let query = sqlx::query::<MySql>(&sql);

        self.conn.execute(query).await.map_err(|e| {
            SqlExecuteError(format!(
                "MySql create table device failed: {}",
                e.to_string()
            ))
        })?;

        Ok(())
    }

    async fn create_secret_key_if_not_exists(&mut self) -> Result<()> {
        if !self.is_secret_key_exist().await? {
            let secret_key = gen_secret_key()?;

            let sql = format!(
                "insert into `{}` (secretKey)
                select ?
                where not exists (select 1 from auth)
                ",
                MYSQL_TABLE_AUTH,
            );
            let query = sqlx::query::<MySql>(&sql).bind(secret_key);

            self.conn.execute(query).await.map_err(|e| {
                SqlExecuteError(format!("MySql insert secret key failed: {}", e.to_string()))
            })?;
        }

        Ok(())
    }

    async fn is_secret_key_exist(&mut self) -> Result<bool> {
        let sql = format!("select count(*) from `{}`", MYSQL_TABLE_AUTH);
        let query = sqlx::query::<MySql>(&sql)
            .fetch_one(&mut self.conn)
            .await
            .map_err(|e| {
                SqlQueryError(format!("MySql query secret key failed: {}", e.to_string()))
            })?;

        let has_secret_key = query.try_get::<bool, &str>("count(*)").map_err(|e| {
            SqlGetValueError(format!(
                "MySql get number of secret key failed: {}",
                e.to_string()
            ))
        })?;

        Ok(has_secret_key)
    }
}

fn gen_secret_key() -> Result<String> {
    let rng = SystemRandom::new();
    // Fernet keys are typically 32 bytes long
    let mut secret_key = vec![0u8; 32];
    // ring::error::Unspecified contains no info
    rng.fill(&mut secret_key)
        .map_err(|_| SecretKeyGenError("Secret key filling failed".to_string()))?;

    let secret_key_str = base64.encode(secret_key);

    Ok(secret_key_str)
}
