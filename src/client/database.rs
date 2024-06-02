/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use serde::{Deserialize, Serialize};
use sqlx::mysql::{MySql, MySqlConnectOptions, MySqlPoolOptions, MySqlRow, MySqlValueRef};
use sqlx::pool::Pool;
use sqlx::{Decode, Encode, Executor, Row, Type};
use std::fmt::Display;

use crate::crypto::Crypto;
use crate::env::{MYSQL_TABLE_AUTH, MYSQL_TABLE_DEVICE, MYSQL_TABLE_MESSAGE};
use crate::error::Error::{
    DatabaseClientError, PortParseError, SqlExecuteError, SqlGetValueError, SqlQueryError,
};
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct Database {
    pool: Pool<MySql>,
    _name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum MessageItemType {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "file")]
    File,
}

impl MessageItemType {
    fn to_string(&self) -> String {
        match self {
            Self::Text => self.to_str().to_string(),
            Self::File => self.to_str().to_string(),
        }
    }

    fn to_str(&self) -> &str {
        match self {
            Self::Text => "text",
            Self::File => "file",
        }
    }
}

impl Display for MessageItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl<'r> Decode<'r, MySql> for MessageItemType {
    fn decode(value: MySqlValueRef<'r>) -> std::result::Result<Self, sqlx::error::BoxDynError> {
        let value = <&str as Decode<MySql>>::decode(value)?;
        match value {
            "text" => Ok(Self::Text),
            "file" => Ok(Self::File),
            _ => Err(Box::<dyn std::error::Error + Send + Sync>::from(format!(
                "Invalid message item type: {}",
                value
            ))),
        }
    }
}

impl<'r> Encode<'r, MySql> for MessageItemType {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::database::HasArguments<'r>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        <&str as Encode<MySql>>::encode(&self.to_string(), buf)
    }
}

impl Type<MySql> for MessageItemType {
    fn type_info() -> <MySql as sqlx::Database>::TypeInfo {
        <&str as Type<MySql>>::type_info()
    }
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct MessageItem {
    pub id: Option<i64>,
    pub content: String,
    timestamp: i64,
    #[serde(rename = "isPrivate")]
    is_private: bool,
    #[serde(rename = "type")]
    type_field: MessageItemType,
    #[serde(rename = "fileName")]
    file_name: Option<String>,
    #[serde(rename = "isComplete")]
    is_complete: Option<bool>,
}

impl MessageItem {
    pub fn new_text(content: &str, timestamp: i64, is_private: bool) -> Self {
        Self {
            id: None,
            content: content.to_string(),
            timestamp,
            is_private,
            type_field: MessageItemType::Text,
            file_name: None,
            is_complete: None,
        }
    }

    pub fn new_file(
        content: &str,
        timestamp: i64,
        is_private: bool,
        file_name: &str,
        is_complete: bool,
    ) -> Self {
        Self {
            id: None,
            content: content.to_string(),
            timestamp,
            is_private,
            type_field: MessageItemType::File,
            file_name: Some(file_name.to_string()),
            is_complete: Some(is_complete),
        }
    }
}

impl From<MySqlRow> for MessageItem {
    fn from(row: MySqlRow) -> Self {
        let id = row
            .try_get::<Option<i64>, &str>("id")
            .expect("MySql failed to get id for MessageItem");

        let content = row
            .try_get::<String, &str>("content")
            .expect("MySql failed to get content for MessageItem");

        let timestamp = row
            .try_get::<i64, &str>("timestamp")
            .expect("MySql failed to get timestamp for MessageItem");

        let is_private = row
            .try_get::<bool, &str>("isPrivate")
            .expect("MySql failed to get isPrivate for MessageItem");

        let type_field = row
            .try_get::<MessageItemType, &str>("type")
            .expect("MySql failed to get type for MessageItem");

        let file_name = row
            .try_get::<Option<String>, &str>("fileName")
            .expect("MySql failed to get fileName for MessageItem");

        let is_complete = row
            .try_get::<Option<bool>, &str>("isComplete")
            .expect("MySql failed to get isComplete for MessageItem");

        Self {
            id,
            content,
            timestamp,
            is_private,
            type_field,
            file_name,
            is_complete,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DeviceItem {
    pub fingerprint: String,
    pub browser: Option<String>,
    #[serde(rename = "lastUseTimestamp")]
    pub last_use_timestamp: Option<i64>,
    #[serde(rename = "expirationTimestamp")]
    pub expiration_timestamp: Option<i64>,
}

impl From<MySqlRow> for DeviceItem {
    fn from(row: MySqlRow) -> Self {
        let fingerprint = row
            .try_get::<String, &str>("fingerprint")
            .expect("MySql failed to get fingerprint for DeviceItem");

        let browser = row
            .try_get::<Option<String>, &str>("browser")
            .expect("MySql failed to get browser for DeviceItem");

        let last_use_timestamp = row
            .try_get::<Option<i64>, &str>("lastUseTimestamp")
            .expect("MySql failed to get lastUseTimestamp for DeviceItem");

        let expiration_timestamp = row
            .try_get::<Option<i64>, &str>("expirationTimestamp")
            .expect("MySql failed to get expirationTimestamp for DeviceItem");

        Self {
            fingerprint,
            browser,
            last_use_timestamp,
            expiration_timestamp,
        }
    }
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
            _name: name.to_string(),
        })
    }

    async fn get_pool(options: MySqlConnectOptions) -> Result<Pool<MySql>> {
        let pool = MySqlPoolOptions::new()
            .connect_with(options)
            .await
            .map_err(|e| DatabaseClientError(format!("MySql pool creation failed: {}", e)))?;

        Ok(pool)
    }

    async fn _close(self) {
        self.pool.close().await;
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

    pub async fn create_table_message_if_not_exists(&self) -> Result<()> {
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

    pub async fn create_table_device_if_not_exists(&self) -> Result<()> {
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
        let sql = format!("select secretKey from `{}` limit 1", MYSQL_TABLE_AUTH);
        let query = sqlx::query::<MySql>(&sql)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| SqlQueryError(format!("MySql query secret key failed: {}", e)))?;

        let secret_key = query
            .try_get::<String, &str>("secretKey")
            .map_err(|e| SqlGetValueError(format!("MySql get secret key failed: {}", e)))?;

        Ok(secret_key)
    }

    async fn _drop_database_if_exists(&self) -> Result<()> {
        let sql = format!("drop database if exists `{}`", self._name);
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
    ) -> Result<Vec<MessageItem>> {
        let mut sql = format!("select * from `{}` ", MYSQL_TABLE_MESSAGE);
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

        let result: Vec<MessageItem> = query
            .into_iter()
            .map(|row| MessageItem::from(row))
            .collect();

        Ok(result)
    }

    pub async fn query_message_items_after_id(
        &self,
        id: u32,
        access_private: bool,
    ) -> Result<Vec<MessageItem>> {
        let mut sql = format!("select * from `{}` where id > ?", MYSQL_TABLE_MESSAGE);
        if !access_private {
            sql.push_str(" and isPrivate = false ");
        }

        let query = sqlx::query::<MySql>(&sql)
            .bind(id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                SqlQueryError(format!("MySql query message items after id failed: {}", e))
            })?;

        let result: Vec<MessageItem> = query
            .into_iter()
            .map(|row| MessageItem::from(row))
            .collect();

        Ok(result)
    }

    pub async fn query_message_latest(&self) -> Option<MessageItem> {
        let sql = format!(
            "select * from `{}` where isPrivate = true order by timestamp desc, id desc limit 1",
            MYSQL_TABLE_MESSAGE
        );

        match sqlx::query::<MySql>(&sql).fetch_one(&self.pool).await {
            Ok(row) => Some(MessageItem::from(row)),
            Err(_) => None,
        }
    }

    pub async fn insert_message_item(&self, item: MessageItem) -> Result<u64> {
        let sql = format!(
            "insert into `{}` (
                content,
                timestamp,
                isPrivate,
                type,
                fileName,
                isComplete
            )
            values (?, ?, ?, ?, ?, ?)",
            MYSQL_TABLE_MESSAGE
        );

        let query = sqlx::query::<MySql>(&sql)
            .bind(item.content)
            .bind(item.timestamp)
            .bind(item.is_private)
            .bind(item.type_field)
            .bind(item.file_name)
            .bind(item.is_complete);

        let id = self
            .pool
            .execute(query)
            .await
            .map_err(|e| SqlExecuteError(format!("MySql insert message item failed: {}", e)))?
            .last_insert_id();

        Ok(id)
    }

    pub async fn remove_message_item(&self, id: i64) -> Result<()> {
        let sql = format!("delete from `{}` where id = ?", MYSQL_TABLE_MESSAGE);

        let query = sqlx::query(&sql).bind(id);

        self.pool
            .execute(query)
            .await
            .map_err(|e| SqlExecuteError(format!("MySql remove message item failed: {}", e)))?;

        Ok(())
    }

    pub async fn remove_message_all(&self) -> Result<()> {
        let sql = format!("delete from `{}`", MYSQL_TABLE_MESSAGE);

        let query = sqlx::query(&sql);

        self.pool
            .execute(query)
            .await
            .map_err(|e| SqlExecuteError(format!("MySql remove message all failed: {}", e)))?;

        Ok(())
    }

    pub async fn update_complete(&self, id: i64) -> Result<()> {
        let sql = format!(
            "update `{}` set isComplete = 1 where id = ?",
            MYSQL_TABLE_MESSAGE
        );

        let query = sqlx::query(&sql).bind(id);

        self.pool
            .execute(query)
            .await
            .map_err(|e| SqlExecuteError(format!("MySql update complete failed: {}", e)))?;

        Ok(())
    }

    pub async fn insert_device(&self, device_item: DeviceItem) -> Result<()> {
        let fingerprint_exists = {
            let sql = format!(
                "select count(*) from {} where fingerprint = \"{}\"",
                MYSQL_TABLE_DEVICE, device_item.fingerprint
            );

            let (count,) = sqlx::query_as::<MySql, (i64,)>(&sql)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| {
                    SqlQueryError(format!("MySql query fingerprint count failed: {}", e))
                })?;

            count > 0
        };

        if fingerprint_exists {
            self.update_device(device_item.clone()).await?;
        } else {
            let sql = format!(
                "insert into `{}` (
                    fingerprint,
                    browser,
                    lastUseTimestamp,
                    expirationTimestamp
                )
                values (?, ?, ?, ?)",
                MYSQL_TABLE_DEVICE
            );

            let query = sqlx::query(&sql)
                .bind(device_item.fingerprint)
                .bind(device_item.browser)
                .bind(device_item.last_use_timestamp)
                .bind(device_item.expiration_timestamp);

            self.pool
                .execute(query)
                .await
                .map_err(|e| SqlExecuteError(format!("MySql insert device failed: {}", e)))?;
        }

        Ok(())
    }

    pub async fn update_device(
        &self,
        DeviceItem {
            fingerprint,
            browser,
            last_use_timestamp,
            expiration_timestamp,
        }: DeviceItem,
    ) -> Result<()> {
        let mut sql = format!("update `{}` set", MYSQL_TABLE_DEVICE);
        let mut params = Vec::<String>::new();

        if let Some(browser) = browser {
            sql = format!("{} browser = ?,", sql);
            params.push(browser);
        }

        if let Some(last_use_timestamp) = last_use_timestamp {
            sql = format!("{} lastUseTimestamp = ?,", sql);
            params.push(last_use_timestamp.to_string());
        }

        if let Some(expiration_timestamp) = expiration_timestamp {
            sql = format!("{} expirationTimestamp = ?,", sql);
            params.push(expiration_timestamp.to_string());
        }

        sql = sql.trim_end_matches(",").to_string();
        sql = format!("{} where fingerprint = ?", sql);

        params.push(fingerprint);

        let mut query = sqlx::query(&sql);

        for param in params.iter() {
            query = query.bind(param);
        }

        self.pool
            .execute(query)
            .await
            .map_err(|e| SqlExecuteError(format!("MySql update device failed: {}", e)))?;

        Ok(())
    }

    pub async fn query_device_items(&self) -> Result<Vec<DeviceItem>> {
        let sql = format!("select * from `{}`", MYSQL_TABLE_DEVICE);

        let query = sqlx::query::<MySql>(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| SqlQueryError(format!("MySql query device failed: {}", e)))?;

        let result = query
            .into_iter()
            .map(|row| DeviceItem::from(row))
            .collect::<Vec<DeviceItem>>();

        Ok(result)
    }

    pub async fn remove_device(&self, fingerprint: &str) -> Result<()> {
        let sql = format!("delete from `{}` where fingerprint = ?", MYSQL_TABLE_DEVICE);
        let query = sqlx::query(&sql).bind(fingerprint);

        self.pool
            .execute(query)
            .await
            .map_err(|e| SqlExecuteError(format!("MySql remove device failed: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::env::tests::get_env;
    use crate::utils::get_current_timestamp;
    use crate::utils::tests::sleep_async;

    pub async fn get_database() -> Database {
        let env = get_env();

        let database = Database::new(
            &env.mysql_endpoint,
            &env.mysql_username,
            &env.mysql_password,
            &env.mysql_database,
        )
        .await
        .unwrap();

        database
    }

    pub async fn reset(database: Database) {
        database._drop_database_if_exists().await.unwrap();
        database._close().await;
    }

    #[tokio::test]
    async fn test_database_new() {
        let env = get_env();

        Database::new(
            &env.mysql_endpoint,
            &env.mysql_username,
            &env.mysql_password,
            &env.mysql_database,
        )
        .await
        .unwrap();

        sleep_async(1).await;
    }

    #[tokio::test]
    async fn test_database_create_database_if_not_exists() {
        let database = get_database().await;

        let result =
            Database::create_database_if_not_exists(&database.pool, database._name.as_str()).await;
        reset(database).await;
        result.unwrap();

        sleep_async(1).await;
    }

    #[tokio::test]
    async fn test_database_init() {
        let database = get_database().await;

        let result = database.init().await;
        reset(database).await;
        result.unwrap();

        sleep_async(1).await;
    }

    #[tokio::test]
    async fn test_database_create_table_message_if_not_exists() {
        let database = get_database().await;

        let result = database.create_table_message_if_not_exists().await;
        reset(database).await;
        result.unwrap();

        sleep_async(1).await;
    }

    #[tokio::test]
    async fn test_database_create_table_auth_if_not_exists() {
        let database = get_database().await;

        let result = database.create_table_auth_if_not_exists().await;
        reset(database).await;
        result.unwrap();

        sleep_async(1).await;
    }

    #[tokio::test]
    async fn test_database_create_table_device_if_not_exists() {
        let database = get_database().await;

        let result = database.create_table_device_if_not_exists().await;
        reset(database).await;
        result.unwrap();

        sleep_async(1).await;
    }

    #[tokio::test]
    async fn test_database_create_secret_key_if_not_exists() {
        async fn inner(database: &Database) -> Result<()> {
            database.create_table_auth_if_not_exists().await?;
            database.create_secret_key_if_not_exists().await?;

            Ok(())
        }

        let database = get_database().await;

        let result = inner(&database).await;
        reset(database).await;
        result.unwrap();

        sleep_async(1).await;
    }

    #[tokio::test]
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
        reset(database).await;

        assert_eq!(result_true.unwrap(), true);
        assert_eq!(result_false.unwrap(), false);

        sleep_async(1).await;
    }

    #[tokio::test]
    async fn test_database_get_secret_key() {
        async fn inner(database: &Database) -> Result<String> {
            database.create_table_auth_if_not_exists().await?;
            database.create_secret_key_if_not_exists().await?;

            let secret_key = database.get_secret_key().await?;

            Ok(secret_key)
        }

        let database = get_database().await;

        let result = inner(&database).await;
        reset(database).await;

        assert_eq!(result.unwrap().len(), 44);

        sleep_async(1).await;
    }

    #[tokio::test]
    async fn test_database_drop_database_if_exists() {
        let database = get_database().await;

        database._drop_database_if_exists().await.unwrap();

        sleep_async(1).await;
    }

    #[tokio::test]
    async fn test_database_insert_message_item() {
        async fn inner(database: &Database, item: MessageItem) -> Result<u64> {
            database.create_table_message_if_not_exists().await?;
            let id = database.insert_message_item(item).await?;

            Ok(id)
        }

        async fn inner_text(database: &Database) -> Result<u64> {
            let item = MessageItem::new_text(
                "test database insert message item text",
                get_current_timestamp(),
                false,
            );
            let id = inner(database, item).await?;

            Ok(id)
        }

        async fn inner_file(database: &Database) -> Result<u64> {
            let item = MessageItem::new_file(
                "test database insert message item file",
                get_current_timestamp(),
                false,
                "test_database_insert_message_item.txt",
                true,
            );
            let id = inner(database, item).await?;

            Ok(id)
        }

        let database = get_database().await;

        let result_text = inner_text(&database).await;
        let result_file = inner_file(&database).await;
        reset(database).await;

        assert_eq!(result_text.unwrap(), 1);
        assert_eq!(result_file.unwrap(), 2);

        sleep_async(1).await;
    }

    #[tokio::test]
    async fn test_database_remove_message_item() {
        async fn inner(database: &Database) -> Result<()> {
            let item = MessageItem::new_text(
                "test database remove message item",
                get_current_timestamp(),
                false,
            );

            database.create_table_message_if_not_exists().await?;
            database.insert_message_item(item).await?;
            database.remove_message_item(1).await?;

            Ok(())
        }

        let database = get_database().await;

        let result = inner(&database).await;
        reset(database).await;
        result.unwrap();

        sleep_async(1).await;
    }

    #[tokio::test]
    async fn test_database_remove_message_all() {
        async fn inner(database: &Database) -> Result<()> {
            let item = MessageItem::new_text(
                "test database remove message item",
                get_current_timestamp(),
                false,
            );

            database.create_table_message_if_not_exists().await?;
            database.insert_message_item(item).await?;
            database.remove_message_all().await?;

            Ok(())
        }

        let database = get_database().await;

        let result = inner(&database).await;
        reset(database).await;
        result.unwrap();

        sleep_async(1).await;
    }

    #[tokio::test]
    async fn test_database_query_message_items() {
        async fn inner(database: &Database) -> Result<Vec<MessageItem>> {
            let item = MessageItem::new_text(
                "test database query message items",
                get_current_timestamp(),
                false,
            );

            database.create_table_message_if_not_exists().await?;
            database.insert_message_item(item).await?;
            let items = database.query_message_items(0, 1, false).await?;

            Ok(items)
        }

        let database = get_database().await;

        let result = inner(&database).await;
        reset(database).await;
        assert_eq!(
            result.unwrap().get(0).unwrap().content,
            "test database query message items"
        );

        sleep_async(1).await;
    }

    #[tokio::test]
    async fn test_database_query_message_items_after_id() {
        async fn inner(database: &Database) -> Result<Vec<MessageItem>> {
            let item1 = MessageItem::new_text(
                "test database query message items after id 1",
                get_current_timestamp(),
                false,
            );

            let item2 = MessageItem::new_text(
                "test database query message items after id 2",
                get_current_timestamp(),
                false,
            );

            database.create_table_message_if_not_exists().await?;
            database.insert_message_item(item1).await?;
            database.insert_message_item(item2).await?;
            let items = database.query_message_items_after_id(0, false).await?;

            Ok(items)
        }

        let database = get_database().await;

        let result = inner(&database).await;
        reset(database).await;

        let result = result.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(
            result.get(0).unwrap().content,
            "test database query message items after id 1"
        );
        assert_eq!(
            result.get(1).unwrap().content,
            "test database query message items after id 2"
        );

        sleep_async(1).await;
    }

    #[tokio::test]
    async fn test_database_query_message_latest() {
        async fn inner(database: &Database, item: MessageItem) -> Result<Option<MessageItem>> {
            database.create_table_message_if_not_exists().await?;
            database.insert_message_item(item).await?;
            let items = database.query_message_latest().await;

            Ok(items)
        }

        let database = get_database().await;

        let item = MessageItem::new_text(
            "test database query message latest",
            get_current_timestamp(),
            true,
        );

        let result = inner(&database, item.clone()).await;
        reset(database).await;

        let result = result.unwrap();

        assert_eq!(result.unwrap().content, item.content);

        sleep_async(1).await;
    }

    #[tokio::test]
    async fn test_database_update_complete() {
        async fn inner(database: &Database) -> Result<()> {
            let content = "test_database_update_complete.txt";
            let item =
                MessageItem::new_file(content, get_current_timestamp(), false, content, false);

            database.create_table_message_if_not_exists().await?;
            let id = database.insert_message_item(item).await?;
            database.update_complete(id as i64).await?;

            Ok(())
        }

        let database = get_database().await;

        let result = inner(&database).await;
        reset(database).await;
        result.unwrap();

        sleep_async(1).await;
    }

    #[tokio::test]
    async fn test_database_insert_device() {
        async fn inner(database: &Database) -> Result<()> {
            let device_item = DeviceItem {
                fingerprint: "fingerprint".to_string(),
                browser: Some("browser".to_string()),
                last_use_timestamp: Some(get_current_timestamp()),
                expiration_timestamp: Some(get_current_timestamp()),
            };

            database.create_table_device_if_not_exists().await?;
            database.insert_device(device_item).await?;

            Ok(())
        }

        let database = get_database().await;

        let result = inner(&database).await;
        reset(database).await;
        result.unwrap();

        sleep_async(1).await;
    }

    #[tokio::test]
    async fn test_database_update_device() {
        async fn inner(database: &Database) -> Result<()> {
            let device_item_old = DeviceItem {
                fingerprint: "fingerprint".to_string(),
                browser: Some("browser_old".to_string()),
                last_use_timestamp: Some(get_current_timestamp()),
                expiration_timestamp: Some(get_current_timestamp()),
            };

            database.create_table_device_if_not_exists().await?;
            database.insert_device(device_item_old).await?;

            let device_item_new = DeviceItem {
                fingerprint: "fingerprint".to_string(),
                browser: Some("browser_new".to_string()),
                last_use_timestamp: Some(get_current_timestamp()),
                expiration_timestamp: Some(get_current_timestamp()),
            };

            database.update_device(device_item_new).await?;

            Ok(())
        }

        let database = get_database().await;

        let result = inner(&database).await;
        reset(database).await;
        result.unwrap();

        sleep_async(1).await;
    }

    #[tokio::test]
    async fn test_database_query_device_items() {
        async fn inner(database: &Database, device_item: DeviceItem) -> Result<Vec<DeviceItem>> {
            database.create_table_device_if_not_exists().await?;
            database.insert_device(device_item).await?;
            let result = database.query_device_items().await?;

            Ok(result)
        }

        let database = get_database().await;

        let device_item = DeviceItem {
            fingerprint: "fingerprint".to_string(),
            browser: Some("browser".to_string()),
            last_use_timestamp: Some(get_current_timestamp()),
            expiration_timestamp: Some(get_current_timestamp()),
        };

        let result = inner(&database, device_item.clone()).await;
        reset(database).await;

        assert_eq!(result.unwrap(), vec![device_item]);
    }

    #[tokio::test]
    async fn test_database_remove_device() {
        async fn inner(database: &Database) -> Result<()> {
            let fingerprint = "fingerprint";

            let device_item = DeviceItem {
                fingerprint: fingerprint.to_string(),
                browser: Some("browser".to_string()),
                last_use_timestamp: Some(get_current_timestamp()),
                expiration_timestamp: Some(get_current_timestamp()),
            };

            database.create_table_device_if_not_exists().await?;
            database.insert_device(device_item).await?;
            database.remove_device(fingerprint).await?;

            Ok(())
        }

        let database = get_database().await;

        let result = inner(&database).await;
        reset(database).await;

        result.unwrap();
    }
}
