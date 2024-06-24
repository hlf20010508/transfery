/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use strum::IntoEnumIterator;

use super::local::tests::{
    get_storage as get_storage_local, init as init_local, reset as reset_local,
    upload_data as upload_data_local,
};
use super::minio::tests::{
    get_storage as get_storage_minio, init as init_minio, reset as reset_minio,
    upload_data as upload_data_minio,
};
use super::models::StorageClient;
use super::utils::tests::fake_data;
use super::Storage;
use crate::client::storage::models::Part;
use crate::env::tests::STType;
use crate::error::Result;
use crate::utils::tests::{sleep, sleep_async};

// s3 minimum allowed size is 5MB
pub static PART_SIZE: u32 = 5 * 1024 * 1024; // 5MB

pub async fn get_storage(st_type: STType) -> Storage {
    let client = match st_type {
        STType::Minio => StorageClient::Minio(get_storage_minio().await),
        STType::LocalStorage => StorageClient::Local(get_storage_local().await),
    };

    Storage { client }
}

pub async fn init(storage: &Storage) -> Result<()> {
    match &storage.client {
        StorageClient::Minio(storage) => init_minio(storage).await,
        StorageClient::Local(storage) => init_local(storage).await,
    }
}

pub async fn reset(storage: &Storage) {
    match &storage.client {
        StorageClient::Minio(storage) => reset_minio(storage).await,
        StorageClient::Local(storage) => reset_local(storage).await,
    }
}

pub async fn upload_data(storage: &Storage, remote_path: &str) -> Result<()> {
    match &storage.client {
        StorageClient::Minio(storage) => upload_data_minio(storage, remote_path).await,
        StorageClient::Local(storage) => upload_data_local(storage, remote_path).await,
    }
}

#[tokio::test]
async fn test_storage_new() {
    for st_type in STType::iter() {
        get_storage(st_type).await;
    }

    sleep(1);
}

#[tokio::test]
async fn test_storage_init() {
    async fn check(st_type: STType) {
        let storage = get_storage(st_type).await;

        let result = storage.init().await;
        reset(&storage).await;
        result.unwrap();
    }

    for st_type in STType::iter() {
        check(st_type).await;
    }

    sleep_async(1).await;
}

#[tokio::test]
async fn test_storage_create_multipart_upload_id() {
    async fn inner(storage: &Storage) -> Result<()> {
        let remote_path = "test-create-multipart-upload-id.txt";

        init(storage).await?;
        storage.create_multipart_upload_id(remote_path).await?;

        Ok(())
    }

    async fn check(st_type: STType) {
        let storage = get_storage(st_type).await;

        let result = inner(&storage).await;
        reset(&storage).await;
        result.unwrap();
    }

    for st_type in STType::iter() {
        check(st_type).await;
    }

    sleep_async(1).await;
}

#[tokio::test]
async fn test_storage_multipart_upload() {
    async fn inner(storage: &Storage) -> Result<()> {
        let remote_path = "test-multipart-upload.txt";

        init(storage).await?;

        let upload_id = storage.create_multipart_upload_id(remote_path).await?;
        let data = fake_data();
        let part_number: u16 = 1;

        storage
            .multipart_upload(remote_path, &upload_id, &data, part_number)
            .await?;

        Ok(())
    }

    async fn check(st_type: STType) {
        let storage = get_storage(st_type).await;

        let result = inner(&storage).await;
        reset(&storage).await;
        result.unwrap();
    }

    for st_type in STType::iter() {
        check(st_type).await;
    }

    sleep_async(1).await;
}

#[tokio::test]
async fn test_storage_complete_multipart_upload() {
    async fn inner(storage: &Storage) -> Result<()> {
        let remote_path = "test-complete-multipart-upload.txt";

        init(storage).await?;

        let upload_id = storage.create_multipart_upload_id(remote_path).await?;
        let data = fake_data();
        let mut parts: Vec<Part> = Vec::new();

        for (part_number, part_data) in data.chunks(PART_SIZE as usize).enumerate() {
            let part_number = part_number as u16 + 1;

            let part = storage
                .multipart_upload(remote_path, &upload_id, part_data, part_number)
                .await?;

            parts.push(part);
        }

        storage
            .complete_multipart_upload(remote_path, &upload_id, &parts)
            .await
    }

    async fn check(st_type: STType) {
        let storage = get_storage(st_type).await;

        let result = inner(&storage).await;
        reset(&storage).await;
        result.unwrap();
    }

    for st_type in STType::iter() {
        check(st_type).await;
    }

    sleep_async(1).await;
}

#[tokio::test]
async fn test_storage_get_download_response() {
    async fn inner(storage: &Storage) -> Result<()> {
        let remote_path = "get_download_url.txt";

        init(storage).await?;

        upload_data(storage, remote_path).await?;
        storage.get_download_response(remote_path).await?;

        Ok(())
    }

    async fn check(st_type: STType) {
        let storage = get_storage(st_type).await;

        let result = inner(&storage).await;
        reset(&storage).await;
        result.unwrap();
    }

    for st_type in STType::iter() {
        check(st_type).await;
    }

    sleep_async(1).await;
}

#[tokio::test]
async fn test_storage_remove_object() {
    async fn inner(storage: &Storage) -> Result<()> {
        let remote_path = "test_remove_object.txt";

        init(storage).await?;

        upload_data(storage, remote_path).await?;
        storage.remove_object(remote_path).await
    }

    async fn check(st_type: STType) {
        let storage = get_storage(st_type).await;

        let result = inner(&storage).await;
        reset(&storage).await;
        result.unwrap();
    }

    for st_type in STType::iter() {
        check(st_type).await;
    }

    sleep_async(1).await;
}

#[tokio::test]
async fn test_storage_remove_objects_all() {
    async fn inner(storage: &Storage) -> Result<()> {
        let remote_path = "test_remove_objects_all.txt";

        init(storage).await?;

        upload_data(storage, remote_path).await?;
        storage.remove_objects_all().await
    }

    async fn check(st_type: STType) {
        let storage = get_storage(st_type).await;

        let result = inner(&storage).await;
        reset(&storage).await;
        result.unwrap();
    }

    for st_type in STType::iter() {
        check(st_type).await;
    }

    sleep_async(1).await;
}
