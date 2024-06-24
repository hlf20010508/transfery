/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::models::config::{Config, MySqlConfig, SqliteConfig};
use super::models::device::{self, DeviceItem};
use super::models::message::{self, MessageItem};
use super::models::token::{self, TokenNewItem};
use super::Database;
use crate::client::database::models::device::DeviceUpdateItem;
use crate::env::tests::{get_env, DBType, STType};
use crate::env::{DatabaseEnv, Env};
use crate::error::Result;
use crate::utils::get_current_timestamp;
use crate::utils::tests::sleep_async;

pub async fn get_database(db_type: DBType) -> Database {
    let Env { database, .. } = get_env(db_type, STType::LocalStorage);

    match database {
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

pub async fn reset(database: Database) {
    database._drop_database_if_exists().await.unwrap();
}

#[tokio::test]
async fn test_database_new() {
    get_database(DBType::MySql).await;
    get_database(DBType::Sqlite).await;

    sleep_async(1).await;
}

#[tokio::test]
async fn test_database_create_database_if_not_exists() {
    let database = get_database(DBType::MySql).await;

    let result =
        Database::create_database_if_not_exists(&database.connection, &database._name).await;
    reset(database).await;
    result.unwrap();

    sleep_async(1).await;
}

#[tokio::test]
async fn test_database_init() {
    async fn check(db_type: DBType) -> Result<()> {
        let database = get_database(db_type).await;

        let result = database.init().await;
        reset(database).await;

        result
    }

    check(DBType::MySql).await.unwrap();
    check(DBType::Sqlite).await.unwrap();

    sleep_async(1).await;
}

#[tokio::test]
async fn test_database_create_table_message_if_not_exists() {
    async fn check(db_type: DBType) -> Result<()> {
        let database = get_database(db_type).await;

        let result = database.create_table_message_if_not_exists().await;
        reset(database).await;

        result
    }

    check(DBType::MySql).await.unwrap();
    check(DBType::Sqlite).await.unwrap();

    sleep_async(1).await;
}

#[tokio::test]
async fn test_database_create_table_auth_if_not_exists() {
    async fn check(db_type: DBType) -> Result<()> {
        let database = get_database(db_type).await;

        let result = database.create_table_auth_if_not_exists().await;
        reset(database).await;

        result
    }

    check(DBType::MySql).await.unwrap();
    check(DBType::Sqlite).await.unwrap();

    sleep_async(1).await;
}

#[tokio::test]
async fn test_database_create_table_device_if_not_exists() {
    async fn check(db_type: DBType) -> Result<()> {
        let database = get_database(db_type).await;

        let result = database.create_table_device_if_not_exists().await;
        reset(database).await;

        result
    }

    check(DBType::MySql).await.unwrap();
    check(DBType::Sqlite).await.unwrap();

    sleep_async(1).await;
}

#[tokio::test]
async fn test_database_create_table_token_if_not_exists() {
    async fn check(db_type: DBType) -> Result<()> {
        let database = get_database(db_type).await;

        let result = database.create_table_token_if_not_exists().await;
        reset(database).await;

        result
    }

    check(DBType::MySql).await.unwrap();
    check(DBType::Sqlite).await.unwrap();

    sleep_async(1).await;
}

#[tokio::test]
async fn test_database_create_secret_key_if_not_exists() {
    async fn inner(database: &Database) -> Result<()> {
        database.create_table_auth_if_not_exists().await?;
        database.create_secret_key_if_not_exists().await
    }

    async fn check(db_type: DBType) {
        let database = get_database(db_type).await;

        let result = inner(&database).await;
        reset(database).await;
        result.unwrap();
    }

    check(DBType::MySql).await;
    check(DBType::Sqlite).await;

    sleep_async(1).await;
}

#[tokio::test]
async fn test_database_is_secret_key_exist() {
    async fn inner(database: &Database) -> Result<()> {
        database.create_table_auth_if_not_exists().await
    }

    async fn inner_true(database: &Database) -> Result<bool> {
        inner(database).await?;

        database.create_secret_key_if_not_exists().await?;

        database.is_secret_key_exist().await
    }

    async fn inner_false(database: &Database) -> Result<bool> {
        inner(database).await?;

        database.is_secret_key_exist().await
    }

    async fn check(db_type: DBType) {
        let database = get_database(db_type).await;

        let result_false = inner_false(&database).await;
        let result_true = inner_true(&database).await;
        reset(database).await;

        assert_eq!(result_true.unwrap(), true);
        assert_eq!(result_false.unwrap(), false);
    }

    check(DBType::MySql).await;
    check(DBType::Sqlite).await;

    sleep_async(1).await;
}

#[tokio::test]
async fn test_database_get_secret_key() {
    async fn inner(database: &Database) -> Result<String> {
        database.create_table_auth_if_not_exists().await?;
        database.create_secret_key_if_not_exists().await?;

        database.get_secret_key().await
    }

    async fn check(db_type: DBType) {
        let database = get_database(db_type).await;

        let result = inner(&database).await;
        reset(database).await;

        assert_eq!(result.unwrap().len(), 44);
    }

    check(DBType::MySql).await;
    check(DBType::Sqlite).await;

    sleep_async(1).await;
}

#[tokio::test]
async fn test_database_drop_database_if_exists() {
    async fn check(db_type: DBType) {
        let database = get_database(db_type).await;

        database._drop_database_if_exists().await.unwrap();
    }

    check(DBType::MySql).await;
    check(DBType::Sqlite).await;

    sleep_async(1).await;
}

#[tokio::test]
async fn test_database_insert_message_item() {
    async fn inner(database: &Database, item: MessageItem) -> Result<i64> {
        database.create_table_message_if_not_exists().await?;
        database.insert_message_item(item).await
    }

    async fn inner_text(database: &Database) -> Result<i64> {
        let item = MessageItem::new_text(
            "test database insert message item text",
            get_current_timestamp(),
            false,
        );

        inner(database, item).await
    }

    async fn inner_file(database: &Database) -> Result<i64> {
        let item = MessageItem::new_file(
            "test database insert message item file",
            get_current_timestamp(),
            false,
            "test_database_insert_message_item.txt",
            true,
        );

        inner(database, item).await
    }

    async fn check(db_type: DBType) {
        let database = get_database(db_type).await;

        let result_text = inner_text(&database).await;
        let result_file = inner_file(&database).await;
        reset(database).await;

        assert_eq!(result_text.unwrap(), 1);
        assert_eq!(result_file.unwrap(), 2);
    }

    check(DBType::MySql).await;
    check(DBType::Sqlite).await;

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
        database.remove_message_item(1).await
    }

    async fn check(db_type: DBType) {
        let database = get_database(db_type).await;

        let result = inner(&database).await;
        reset(database).await;
        result.unwrap();
    }

    check(DBType::MySql).await;
    check(DBType::Sqlite).await;

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
        database.remove_message_all().await
    }

    async fn check(db_type: DBType) {
        let database = get_database(db_type).await;

        let result = inner(&database).await;
        reset(database).await;
        result.unwrap();
    }

    check(DBType::MySql).await;
    check(DBType::Sqlite).await;

    sleep_async(1).await;
}

#[tokio::test]
async fn test_database_query_message_items() {
    async fn inner(database: &Database) -> Result<Vec<message::Model>> {
        let item = MessageItem::new_text(
            "test database query message items",
            get_current_timestamp(),
            false,
        );

        database.create_table_message_if_not_exists().await?;
        database.insert_message_item(item).await?;
        database.query_message_items(0, 1, false).await
    }

    async fn check(db_type: DBType) {
        let database = get_database(db_type).await;

        let result = inner(&database).await;
        reset(database).await;
        assert_eq!(
            result.unwrap().get(0).unwrap().content,
            "test database query message items"
        );
    }

    check(DBType::MySql).await;
    check(DBType::Sqlite).await;

    sleep_async(1).await;
}

#[tokio::test]
async fn test_database_query_message_items_after_id() {
    async fn inner(database: &Database) -> Result<Vec<message::Model>> {
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
        database.query_message_items_after_id(0, false).await
    }

    async fn check(db_type: DBType) {
        let database = get_database(db_type).await;

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
    }

    check(DBType::MySql).await;
    check(DBType::Sqlite).await;

    sleep_async(1).await;
}

#[tokio::test]
async fn test_database_query_message_latest() {
    async fn inner(database: &Database, item: MessageItem) -> Result<Option<message::Model>> {
        database.create_table_message_if_not_exists().await?;
        database.insert_message_item(item).await?;
        database.query_message_latest().await
    }

    async fn check(db_type: DBType) {
        let database = get_database(db_type).await;

        let item = MessageItem::new_text(
            "test database query message latest",
            get_current_timestamp(),
            true,
        );

        let result = inner(&database, item.clone()).await;
        reset(database).await;

        let result = result.unwrap();

        assert_eq!(result.unwrap().content, item.content);
    }

    check(DBType::MySql).await;
    check(DBType::Sqlite).await;

    sleep_async(1).await;
}

#[tokio::test]
async fn test_database_update_complete() {
    async fn inner(database: &Database) -> Result<()> {
        let content = "test_database_update_complete.txt";
        let item = MessageItem::new_file(content, get_current_timestamp(), false, content, false);

        database.create_table_message_if_not_exists().await?;
        let id = database.insert_message_item(item).await?;
        database.update_complete(id).await
    }

    async fn check(db_type: DBType) {
        let database = get_database(db_type).await;

        let result = inner(&database).await;
        reset(database).await;
        result.unwrap();
    }

    check(DBType::MySql).await;
    check(DBType::Sqlite).await;

    sleep_async(1).await;
}

#[tokio::test]
async fn test_database_insert_device() {
    async fn inner(database: &Database) -> Result<()> {
        let device_item = DeviceItem {
            fingerprint: "fingerprint".to_string(),
            browser: "browser".to_string(),
            last_use_timestamp: get_current_timestamp(),
            expiration_timestamp: get_current_timestamp(),
        };

        database.create_table_device_if_not_exists().await?;
        database.insert_device(device_item).await
    }

    async fn check(db_type: DBType) {
        let database = get_database(db_type).await;

        let result = inner(&database).await;
        reset(database).await;
        result.unwrap();
    }

    check(DBType::MySql).await;
    check(DBType::Sqlite).await;

    sleep_async(1).await;
}

#[tokio::test]
async fn test_database_update_device() {
    async fn inner(database: &Database) -> Result<()> {
        let device_item_old = DeviceItem {
            fingerprint: "fingerprint".to_string(),
            browser: "browser_old".to_string(),
            last_use_timestamp: get_current_timestamp(),
            expiration_timestamp: get_current_timestamp(),
        };

        database.create_table_device_if_not_exists().await?;
        database.insert_device(device_item_old).await?;

        let device_item_new = DeviceUpdateItem {
            fingerprint: "fingerprint".to_string(),
            browser: Some("browser_new".to_string()),
            last_use_timestamp: Some(get_current_timestamp()),
            expiration_timestamp: Some(get_current_timestamp()),
        };

        database.update_device(device_item_new).await
    }

    async fn check(db_type: DBType) {
        let database = get_database(db_type).await;

        let result = inner(&database).await;
        reset(database).await;
        result.unwrap();
    }

    check(DBType::MySql).await;
    check(DBType::Sqlite).await;

    sleep_async(1).await;
}

#[tokio::test]
async fn test_database_query_device_items() {
    async fn inner(database: &Database, device_item: DeviceItem) -> Result<Vec<device::Model>> {
        database.create_table_device_if_not_exists().await?;
        database.insert_device(device_item).await?;
        database.query_device_items().await
    }

    async fn check(db_type: DBType) {
        let database = get_database(db_type).await;

        let device_item = DeviceItem {
            fingerprint: "fingerprint".to_string(),
            browser: "browser".to_string(),
            last_use_timestamp: get_current_timestamp(),
            expiration_timestamp: get_current_timestamp(),
        };

        let result = inner(&database, device_item.clone()).await;
        reset(database).await;
        let result = result.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].fingerprint, device_item.fingerprint);
        assert_eq!(result[0].browser, device_item.browser);
        assert_eq!(result[0].last_use_timestamp, device_item.last_use_timestamp);
        assert_eq!(
            result[0].expiration_timestamp,
            device_item.expiration_timestamp
        );
    }

    check(DBType::MySql).await;
    check(DBType::Sqlite).await;

    sleep_async(1).await;
}

#[tokio::test]
async fn test_database_remove_device() {
    async fn inner(database: &Database) -> Result<()> {
        let fingerprint = "fingerprint";

        let device_item = DeviceItem {
            fingerprint: fingerprint.to_string(),
            browser: "browser".to_string(),
            last_use_timestamp: get_current_timestamp(),
            expiration_timestamp: get_current_timestamp(),
        };

        database.create_table_device_if_not_exists().await?;
        database.insert_device(device_item).await?;
        database.remove_device(fingerprint).await
    }

    async fn check(db_type: DBType) {
        let database = get_database(db_type).await;

        let result = inner(&database).await;
        reset(database).await;

        result.unwrap();
    }

    check(DBType::MySql).await;
    check(DBType::Sqlite).await;

    sleep_async(1).await;
}

#[tokio::test]
async fn test_database_insert_token() {
    async fn inner(database: &Database) -> Result<()> {
        database.create_table_token_if_not_exists().await?;
        let new_token_item = TokenNewItem {
            token: "test_token".to_string(),
            name: "test name".to_string(),
            expiration_timestamp: get_current_timestamp(),
        };
        database.insert_token(new_token_item).await
    }

    async fn check(db_type: DBType) {
        let database = get_database(db_type).await;
        let result = inner(&database).await;
        reset(database).await;
        result.unwrap();
    }

    check(DBType::MySql).await;
    check(DBType::Sqlite).await;

    sleep_async(1).await;
}

#[tokio::test]
async fn test_database_update_token() {
    async fn inner(database: &Database) -> Result<()> {
        database.create_table_token_if_not_exists().await?;

        let timestamp = get_current_timestamp();
        let token = "test_token".to_string();

        let new_token_item = TokenNewItem {
            token: token.clone(),
            name: "test name".to_string(),
            expiration_timestamp: timestamp,
        };
        database.insert_token(new_token_item).await?;

        database.update_token(&token, timestamp + 1000).await
    }

    async fn check(db_type: DBType) {
        let database = get_database(db_type).await;
        let result = inner(&database).await;
        reset(database).await;
        result.unwrap();
    }

    check(DBType::MySql).await;
    check(DBType::Sqlite).await;

    sleep_async(1).await;
}

#[tokio::test]
async fn test_database_query_token_items() {
    async fn inner(database: &Database, new_token_item: TokenNewItem) -> Result<Vec<token::Model>> {
        database.create_table_token_if_not_exists().await?;
        database.insert_token(new_token_item).await?;

        database.query_token_items().await
    }

    async fn check(db_type: DBType) {
        let database = get_database(db_type).await;

        let new_token_item = TokenNewItem {
            token: "test_token".to_string(),
            name: "test name".to_string(),
            expiration_timestamp: get_current_timestamp(),
        };

        let result = inner(&database, new_token_item.clone()).await;
        reset(database).await;

        let result = result.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].token, new_token_item.token);
        assert_eq!(result[0].name, new_token_item.name);
    }

    check(DBType::MySql).await;
    check(DBType::Sqlite).await;

    sleep_async(1).await;
}

#[tokio::test]
async fn test_database_remove_token() {
    async fn inner(database: &Database) -> Result<()> {
        database.create_table_token_if_not_exists().await?;

        let new_token_item = TokenNewItem {
            token: "test_token".to_string(),
            name: "test name".to_string(),
            expiration_timestamp: get_current_timestamp(),
        };
        database.insert_token(new_token_item.clone()).await?;

        database.remove_token(new_token_item.token).await
    }

    async fn check(db_type: DBType) {
        let database = get_database(db_type).await;
        let result = inner(&database).await;
        reset(database).await;
        result.unwrap();
    }

    check(DBType::MySql).await;
    check(DBType::Sqlite).await;

    sleep_async(1).await;
}
