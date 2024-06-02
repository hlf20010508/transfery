/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::{Database, DeviceItem, MessageItem};

use crate::env::tests::get_env;
use crate::error::Result;
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
        let item = MessageItem::new_file(content, get_current_timestamp(), false, content, false);

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
