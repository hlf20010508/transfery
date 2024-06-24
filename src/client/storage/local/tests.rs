/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::http::StatusCode;
use axum::response::Response;
use tokio::fs;

use super::super::utils::tests::fake_data;
use super::utils::LocalStorageUtils;
use super::LocalStorage;
use crate::client::{storage::models::StorageClient, Storage};
use crate::env::tests::{get_env, DBType, STType};
use crate::error::ErrorType::InternalServerError;
use crate::error::{Error, Result};

pub async fn get_storage() -> LocalStorage {
    let env = get_env(DBType::Sqlite, STType::LocalStorage);

    let storage = match Storage::new(&env.storage).await.unwrap().client {
        StorageClient::Local(storage) => storage,
        _ => unreachable!(),
    };

    storage
}

pub async fn init(storage: &LocalStorage) -> Result<()> {
    storage.init().await
}

pub async fn reset(storage: &LocalStorage) {
    storage.remove_dir().await.unwrap();
}

pub async fn upload_data(storage: &LocalStorage, object: &str) -> Result<()> {
    let data = fake_data();

    fs::write(storage.get_path(object), data)
        .await
        .map_err(|e| Error::context(InternalServerError, e, "failed to write file"))?;

    Ok(())
}

#[tokio::test]
async fn test_local_storage_get_download_response() {
    async fn inner(storage: &LocalStorage) -> Result<Response> {
        init(storage).await?;

        let file_name = "test.txt";

        upload_data(storage, file_name).await?;

        storage.get_download_response(file_name).await
    }

    let storage = get_storage().await;
    let result = inner(&storage).await;
    reset(&storage).await;

    assert_eq!(result.unwrap().status(), StatusCode::OK);
}
