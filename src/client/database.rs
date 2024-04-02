/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use sqlx::mysql::{MySql, MySqlConnectOptions, MySqlPoolOptions};
use sqlx::pool::Pool;
use sqlx::{Executor, Row};

use crate::crypto::Crypto;
use crate::env::{MYSQL_TABLE_AUTH, MYSQL_TABLE_DEVICE, MYSQL_TABLE_MESSAGE};
use crate::error::Error::{
    DatabaseClientError, PortParseError, SqlExecuteError, SqlGetValueError, SqlQueryError,
};
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct Database {
    pool: Pool<MySql>,
    name: String,
}

// init
impl Database {
    pub async fn new(endpoint: &str, username: &str, password: &str, name: &str) -> Result<Self> {
        let endpoint_collection = endpoint.split(':').collect::<Vec<&str>>();
        let host = endpoint_collection[0];
        let port = endpoint_collection[1]
            .parse::<u16>()
            .map_err(|e| PortParseError(format!("MySql port parsing failed: {}", e)))?;

        let options = MySqlConnectOptions::new()
            .host(host)
            .port(port)
            .username(username)
            .password(password);

        let pool = Database::get_pool(options).await?;

        Self::create_database_if_not_exists(&pool, name).await?;

        let options = MySqlConnectOptions::new()
            .host(host)
            .port(port)
            .username(username)
            .password(password)
            .database(name);

        let pool = Database::get_pool(options).await?;

        Ok(Self {
            pool,
            name: name.to_string(),
        })
    }

    async fn get_pool(options: MySqlConnectOptions) -> Result<Pool<MySql>> {
        let pool = MySqlPoolOptions::new()
            .connect_with(options)
            .await
            .map_err(|e| DatabaseClientError(format!("MySql pool creation failed: {}", e)))?;

        Ok(pool)
    }

    async fn create_database_if_not_exists(pool: &Pool<MySql>, name: &str) -> Result<()> {
        let sql = format!("create database if not exists `{}`", name);
        let query = sqlx::query::<MySql>(&sql);

        pool.execute(query)
            .await
            .map_err(|e| SqlExecuteError(format!("MySql create database failed: {}", e)))?;

        Ok(())
    }

    pub async fn init(&self) -> Result<()> {
        self.create_table_message_if_not_exists().await?;
        self.create_table_auth_if_not_exists().await?;
        self.create_table_device_if_not_exists().await?;
        self.create_secret_key_if_not_exists().await?;

        Ok(())
    }

    async fn create_table_message_if_not_exists(&self) -> Result<()> {
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

        self.pool
            .execute(query)
            .await
            .map_err(|e| SqlExecuteError(format!("MySql create table message failed: {}", e)))?;

        Ok(())
    }

    async fn create_table_auth_if_not_exists(&self) -> Result<()> {
        let sql = format!(
            "create table if not exists `{}`(
                id int primary key auto_increment,
                secretKey text not null
            )",
            MYSQL_TABLE_AUTH
        );
        let query = sqlx::query::<MySql>(&sql);

        self.pool
            .execute(query)
            .await
            .map_err(|e| SqlExecuteError(format!("MySql create table auth failed: {}", e)))?;

        Ok(())
    }

    async fn create_table_device_if_not_exists(&self) -> Result<()> {
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

        self.pool
            .execute(query)
            .await
            .map_err(|e| SqlExecuteError(format!("MySql create table device failed: {}", e)))?;

        Ok(())
    }

    async fn create_secret_key_if_not_exists(&self) -> Result<()> {
        if !self.is_secret_key_exist().await? {
            let secret_key = Crypto::gen_secret_key()?;

            let sql = format!(
                "insert into `{}` (secretKey)
                select ?
                where not exists (select 1 from auth)
                ",
                MYSQL_TABLE_AUTH,
            );
            let query = sqlx::query::<MySql>(&sql).bind(secret_key);

            self.pool
                .execute(query)
                .await
                .map_err(|e| SqlExecuteError(format!("MySql insert secret key failed: {}", e)))?;
        }

        Ok(())
    }

    async fn is_secret_key_exist(&self) -> Result<bool> {
        let sql = format!("select count(*) from `{}`", MYSQL_TABLE_AUTH);
        let query = sqlx::query::<MySql>(&sql)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| SqlQueryError(format!("MySql query secret key failed: {}", e)))?;

        let has_secret_key = query.try_get::<bool, &str>("count(*)").map_err(|e| {
            SqlGetValueError(format!("MySql get number of secret key failed: {}", e))
        })?;

        Ok(has_secret_key)
    }

    pub async fn get_secret_key(&self) -> Result<String> {
        let sql = format!("select secretKey from {} limit 1", MYSQL_TABLE_AUTH);
        let query = sqlx::query::<MySql>(&sql)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| SqlQueryError(format!("MySql query secret key failed: {}", e)))?;

        let secret_key = query
            .try_get::<String, &str>("secretKey")
            .map_err(|e| SqlGetValueError(format!("MySql get secret key failed: {}", e)))?;

        Ok(secret_key)
    }

    async fn drop_database_if_exists(&self) -> Result<()> {
        let sql = format!("drop database if exists `{}`", self.name);
        let query = sqlx::query::<MySql>(&sql);

        self.pool
            .execute(query)
            .await
            .map_err(|e| SqlExecuteError(format!("MySql drop database failed: {}", e)))?;

        Ok(())
    }
}

// main
impl Database {
    pub async fn query_message_items(
        &self,
        start: u32,
        number: u8,
        access_private: bool,
    ) -> Result<()> {
        let mut sql = format!("select * from {} ", MYSQL_TABLE_MESSAGE);
        if !access_private {
            sql.push_str("where isPrivate = false ");
        }
        sql.push_str("order by timestamp desc, id desc limit ?, ?");

        let query = sqlx::query::<MySql>(&sql)
            .bind(start)
            .bind(number)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| SqlQueryError(format!("MySql query message items failed: {}", e)))?;

        println!("{:#?}", query);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use dotenv::dotenv;
    use std::env;

    use super::*;

    fn get_endpoint() -> String {
        env::var("MYSQL_ENDPOINT").unwrap()
    }

    fn get_username() -> String {
        env::var("MYSQL_USERNAME").unwrap()
    }

    fn get_password() -> String {
        env::var("MYSQL_PASSWORD").unwrap()
    }

    fn get_name() -> String {
        env::var("MYSQL_DATABASE").unwrap()
    }

    async fn get_database() -> Database {
        dotenv().ok();

        let endpoint = get_endpoint();
        let username = get_username();
        let password = get_password();
        let name = get_name();

        let database = Database::new(&endpoint, &username, &password, &name)
            .await
            .unwrap();

        database
    }

    async fn reset(database: &Database) {
        database.drop_database_if_exists().await.unwrap();
    }

    #[actix_web::test]
    async fn test_database_new() {
        dotenv().ok();

        let endpoint = get_endpoint();
        let username = get_username();
        let password = get_password();
        let name = get_name();

        Database::new(&endpoint, &username, &password, &name)
            .await
            .unwrap();
    }

    #[actix_web::test]
    async fn test_database_create_database_if_not_exists() {
        let database = get_database().await;

        let result =
            Database::create_database_if_not_exists(&database.pool, database.name.as_str()).await;
        reset(&database).await;
        result.unwrap();
    }

    #[actix_web::test]
    async fn test_database_init() {
        let database = get_database().await;

        let result = database.init().await;
        reset(&database).await;
        result.unwrap();
    }

    #[actix_web::test]
    async fn test_database_create_table_message_if_not_exists() {
        let database = get_database().await;

        let result = database.create_table_message_if_not_exists().await;
        reset(&database).await;
        result.unwrap();
    }

    #[actix_web::test]
    async fn test_database_create_table_auth_if_not_exists() {
        let database = get_database().await;

        let result = database.create_table_auth_if_not_exists().await;
        reset(&database).await;
        result.unwrap();
    }

    #[actix_web::test]
    async fn test_database_create_table_device_if_not_exists() {
        let database = get_database().await;

        let result = database.create_table_device_if_not_exists().await;
        reset(&database).await;
        result.unwrap();
    }

    #[actix_web::test]
    async fn test_database_create_secret_key_if_not_exists() {
        async fn inner(database: &Database) -> Result<()> {
            database.create_table_auth_if_not_exists().await?;
            database.create_secret_key_if_not_exists().await?;

            Ok(())
        }

        let database = get_database().await;

        let result = inner(&database).await;
        reset(&database).await;
        result.unwrap();
    }

    #[actix_web::test]
    async fn test_database_is_secret_key_exist() {
        async fn inner(database: &Database) -> Result<()> {
            database.create_table_auth_if_not_exists().await?;

            Ok(())
        }

        async fn inner_true(database: &Database) -> Result<bool> {
            inner(database).await?;

            database.create_secret_key_if_not_exists().await?;

            let result = database.is_secret_key_exist().await?;

            Ok(result)
        }

        async fn inner_false(database: &Database) -> Result<bool> {
            inner(database).await?;

            let result = database.is_secret_key_exist().await?;

            Ok(result)
        }

        let database = get_database().await;

        let result_false = inner_false(&database).await;
        let result_true = inner_true(&database).await;
        reset(&database).await;

        assert_eq!(result_true.unwrap(), true);
        assert_eq!(result_false.unwrap(), false);
    }

    #[actix_web::test]
    async fn test_database_get_secret_key() {
        async fn inner(database: &Database) -> Result<String> {
            database.create_table_auth_if_not_exists().await?;
            database.create_secret_key_if_not_exists().await?;

            let secret_key = database.get_secret_key().await?;

            Ok(secret_key)
        }

        let database = get_database().await;

        let result = inner(&database).await;
        reset(&database).await;

        assert_eq!(result.unwrap().len(), 44);
    }

    #[actix_web::test]
    async fn test_database_drop_database_if_exists() {
        let database = get_database().await;

        database.drop_database_if_exists().await.unwrap();
    }
}
