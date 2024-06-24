/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use minio::s3::args::PutObjectArgs;
use std::io::Cursor;

use super::super::utils::tests::fake_data;
use super::Minio;
use crate::client::storage::models::StorageClient;
use crate::client::Storage;
use crate::env::tests::{get_env, DBType, STType};
use crate::error::ErrorType::InternalServerError;
use crate::error::{Error, Result};
use crate::utils::tests::sleep_async;

pub async fn get_storage() -> Minio {
    let env = get_env(DBType::Sqlite, STType::Minio);

    let storage = match Storage::new(&env.storage).await.unwrap().client {
        StorageClient::Minio(storage) => storage,
        _ => unreachable!(),
    };

    storage
}

pub async fn init(storage: &Minio) -> Result<()> {
    storage.init().await
}

pub async fn reset(storage: &Minio) {
    storage._remove_bucket().await.unwrap();
}

pub async fn upload_data(storage: &Minio, object: &str) -> Result<()> {
    let data = fake_data();

    let size = data.len();
    let mut data = Cursor::new(data);

    let mut args = PutObjectArgs::new(&storage.bucket, object, &mut data, Some(size), None)
        .map_err(|e| Error::context(InternalServerError, e, "failed to create put object args"))?;

    storage
        .client
        .put_object(&mut args)
        .await
        .map_err(|e| Error::context(InternalServerError, e, "failed to put object"))?;

    Ok(())
}

#[tokio::test]
async fn test_minio_create_buffer_if_not_exists() {
    let storage = get_storage().await;

    let result = storage.create_buffer_if_not_exists().await;
    reset(&storage).await;
    result.unwrap();

    sleep_async(1).await;
}

#[tokio::test]
async fn test_minio_is_bucket_exists() {
    async fn inner_true(storage: &Minio) -> Result<bool> {
        init(storage).await?;
        let result = storage.is_bucket_exists().await?;

        Ok(result)
    }

    async fn inner_false(storage: &Minio) -> Result<bool> {
        let result = storage.is_bucket_exists().await?;

        Ok(result)
    }

    let storage = get_storage().await;

    let result_false = inner_false(&storage).await;

    let result_true = inner_true(&storage).await;
    reset(&storage).await;

    assert_eq!(result_false.unwrap(), false);
    assert_eq!(result_true.unwrap(), true);

    sleep_async(1).await;
}

#[tokio::test]
async fn test_minio_list_objects() {
    async fn inner(storage: &Minio) -> Result<()> {
        init(storage).await?;

        storage.list_objects().await?;

        Ok(())
    }

    let storage = get_storage().await;

    let result = inner(&storage).await;
    reset(&storage).await;
    result.unwrap();

    sleep_async(1).await;
}

#[tokio::test]
async fn test_minio_remove_bucket() {
    let storage = get_storage().await;

    init(&storage).await.unwrap();

    storage._remove_bucket().await.unwrap();

    sleep_async(1).await;
}
