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

#[derive(Debug)]
pub struct Database {
    conn: MySqlConnection,
    name: String,
}

impl Database {
    pub async fn new(endpoint: &str, username: &str, password: &str, name: &str) -> Result<Self> {
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

        Ok(Self {
            conn,
            name: name.to_string(),
        })
    }

    async fn create_database_if_not_exists(&mut self) -> Result<()> {
        let sql = format!("create database if not exists `{}`", self.name);
        let query = sqlx::query::<MySql>(&sql);

        self.conn.execute(query).await.map_err(|e| {
            SqlExecuteError(format!("MySql create database failed: {}", e.to_string()))
        })?;

        Ok(())
    }

    async fn set_database(&mut self) -> Result<()> {
        let sql = format!("use `{}`", self.name);
        let query = sqlx::query::<MySql>(&sql);

        self.conn.execute(query).await.map_err(|e| {
            SqlExecuteError(format!("MySql set database failed: {}", e.to_string()))
        })?;

        Ok(())
    }

    pub async fn init(&mut self) -> Result<()> {
        self.create_database_if_not_exists().await?;
        self.set_database().await?;
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

    async fn drop_database_if_exists(&mut self) -> Result<()> {
        let sql = format!("drop database if exists `{}`", self.name);
        let query = sqlx::query::<MySql>(&sql);

        self.conn.execute(query).await.map_err(|e| {
            SqlExecuteError(format!("MySql drop database failed: {}", e.to_string()))
        })?;

        Ok(())
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

#[cfg(test)]
mod tests {
    use dotenv::dotenv;
    use std::env;

    use super::*;

    async fn get_database() -> Database {
        dotenv().ok();

        let endpoint = env::var("MYSQL_ENDPOINT").unwrap();
        let username = env::var("MYSQL_USERNAME").unwrap();
        let password = env::var("MYSQL_PASSWORD").unwrap();
        let name = env::var("MYSQL_DATABASE").unwrap();

        let database = Database::new(&endpoint, &username, &password, &name)
            .await
            .unwrap();

        database
    }

    async fn init(database: &mut Database) {
        database.create_database_if_not_exists().await.unwrap();
    }

    async fn reset(database: &mut Database) {
        database.drop_database_if_exists().await.unwrap();
    }

    #[actix_web::test]
    async fn test_database_new() {
        dotenv().ok();

        let endpoint = env::var("MYSQL_ENDPOINT").unwrap();
        let username = env::var("MYSQL_USERNAME").unwrap();
        let password = env::var("MYSQL_PASSWORD").unwrap();
        let name = env::var("MYSQL_DATABASE").unwrap();

        Database::new(&endpoint, &username, &password, &name)
            .await
            .unwrap();
    }

    #[actix_web::test]
    async fn test_database_create_database_if_not_exists() {
        let mut database = get_database().await;

        let result = database.drop_database_if_exists().await;
        reset(&mut database).await;
        result.unwrap();
    }

    #[actix_web::test]
    async fn test_database_set_database() {
        let mut database = get_database().await;

        init(&mut database).await;
        let result = database.set_database().await;
        reset(&mut database).await;
        result.unwrap();
    }

    #[actix_web::test]
    async fn test_database_init() {
        let mut database = get_database().await;

        let result = database.init().await;
        reset(&mut database).await;
        result.unwrap();
    }

    #[actix_web::test]
    async fn test_database_create_table_message_if_not_exists() {
        async fn inner(database: &mut Database) -> Result<()> {
            init(database).await;
            database.set_database().await?;
            database.create_table_message_if_not_exists().await?;

            Ok(())
        }

        let mut database = get_database().await;

        let result = inner(&mut database).await;
        reset(&mut database).await;
        result.unwrap();
    }

    #[actix_web::test]
    async fn test_database_create_table_auth_if_not_exists() {
        async fn inner(database: &mut Database) -> Result<()> {
            init(database).await;
            database.set_database().await?;
            database.create_table_auth_if_not_exists().await?;

            Ok(())
        }

        let mut database = get_database().await;

        let result = inner(&mut database).await;
        reset(&mut database).await;
        result.unwrap();
    }

    #[actix_web::test]
    async fn test_database_create_table_device_if_not_exists() {
        async fn inner(database: &mut Database) -> Result<()> {
            init(database).await;
            database.set_database().await?;
            database.create_table_device_if_not_exists().await?;

            Ok(())
        }

        let mut database = get_database().await;

        let result = inner(&mut database).await;
        reset(&mut database).await;
        result.unwrap();
    }

    #[actix_web::test]
    async fn test_database_create_secret_key_if_not_exists() {
        async fn inner(database: &mut Database) -> Result<()> {
            init(database).await;
            database.set_database().await?;
            database.create_table_auth_if_not_exists().await?;
            database.create_secret_key_if_not_exists().await?;

            Ok(())
        }

        let mut database = get_database().await;

        let result = inner(&mut database).await;
        reset(&mut database).await;
        result.unwrap();
    }

    #[actix_web::test]
    async fn test_database_is_secret_key_exist() {
        async fn inner(database: &mut Database) -> Result<()> {
            init(database).await;
            database.set_database().await?;
            database.create_table_auth_if_not_exists().await?;

            Ok(())
        }

        async fn inner_true(database: &mut Database) -> Result<bool> {
            inner(database).await?;

            database.create_secret_key_if_not_exists().await?;

            let result = database.is_secret_key_exist().await?;

            Ok(result)
        }

        async fn inner_false(database: &mut Database) -> Result<bool> {
            inner(database).await?;

            let result = database.is_secret_key_exist().await?;

            Ok(result)
        }

        let mut database = get_database().await;

        let result_false = inner_false(&mut database).await;
        reset(&mut database).await;

        let result_true = inner_true(&mut database).await;
        reset(&mut database).await;

        assert_eq!(result_true.unwrap(), true);
        assert_eq!(result_false.unwrap(), false);
    }

    #[actix_web::test]
    async fn test_database_drop_database_if_exists() {
        async fn inner(database: &mut Database) -> Result<()> {
            init(database).await;
            database.set_database().await?;
            database.drop_database_if_exists().await?;

            Ok(())
        }

        let mut database = get_database().await;

        let result = inner(&mut database).await;
        result.unwrap();
    }

    #[test]
    fn test_gen_secret_key() {
        let result = gen_secret_key();
        assert_eq!(result.unwrap().len(), 44);
    }
}
