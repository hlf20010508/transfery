/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use minio::s3::args::PutObjectArgs;
use minio::s3::types::Part;
use std::io::Cursor;

use super::Storage;

use crate::env::tests::get_env;
use crate::error::Error::StorageObjectError;
use crate::error::Result;
use crate::utils::tests::{sleep, sleep_async};

// s3 minimum allowed size is 5MB
pub static PART_SIZE: u32 = 5 * 1024 * 1024; // 5MB

pub fn get_storage() -> Storage {
    let env = get_env();

    let storage = Storage::new(
        &env.minio_endpoint,
        &env.minio_username,
        &env.minio_password,
        &env.minio_bucket,
    )
    .unwrap();

    storage
}

pub async fn init(storage: &Storage) -> Result<()> {
    storage.init().await
}

pub async fn reset(storage: &Storage) {
    storage._remove_bucket().await.unwrap();
}

fn fake_data() -> Vec<u8> {
    let data = Vec::from("hello world!");

    let repeat_times: usize = 1024 * 1024;

    let data = data
        .iter()
        .cycle()
        .take(data.len() * repeat_times)
        .cloned()
        .collect();

    data
}

pub async fn upload_data(storage: &Storage, remote_path: &str) -> Result<()> {
    let mut data = Cursor::new(fake_data());
    let size = data.clone().into_inner().len();

    let mut args = PutObjectArgs::new(&storage.bucket, remote_path, &mut data, Some(size), None)
        .map_err(|e| StorageObjectError(format!("Storage create put object args failed: {}", e)))?;

    storage
        .client
        .put_object(&mut args)
        .await
        .map_err(|e| StorageObjectError(format!("Storage put object failed: {}", e)))?;

    Ok(())
}

#[test]
fn test_storage_new() {
    let env = get_env();

    Storage::new(
        &env.minio_endpoint,
        &env.minio_username,
        &env.minio_password,
        &env.minio_bucket,
    )
    .unwrap();

    sleep(1);
}

#[tokio::test]
async fn test_storage_init() {
    let storage = get_storage();

    let result = storage.init().await;
    reset(&storage).await;
    result.unwrap();

    sleep_async(1).await;
}

#[tokio::test]
async fn test_storage_create_buffer_if_not_exists() {
    let storage = get_storage();

    let result = storage.create_buffer_if_not_exists().await;
    reset(&storage).await;
    result.unwrap();

    sleep_async(1).await;
}

#[tokio::test]
async fn test_storage_is_bucket_exists() {
    async fn inner_true(storage: &Storage) -> Result<bool> {
        init(storage).await?;
        let result = storage.is_bucket_exists().await?;

        Ok(result)
    }

    async fn inner_false(storage: &Storage) -> Result<bool> {
        let result = storage.is_bucket_exists().await?;

        Ok(result)
    }

    let storage = get_storage();

    let result_false = inner_false(&storage).await;

    let result_true = inner_true(&storage).await;
    reset(&storage).await;

    assert_eq!(result_false.unwrap(), false);
    assert_eq!(result_true.unwrap(), true);

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

    let storage = get_storage();

    let result = inner(&storage).await;
    reset(&storage).await;
    result.unwrap();

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

    let storage = get_storage();

    let result = inner(&storage).await;
    reset(&storage).await;
    result.unwrap();

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
            .await?;

        Ok(())
    }

    let storage = get_storage();

    let result = inner(&storage).await;
    reset(&storage).await;
    result.unwrap();

    sleep_async(1).await;
}

#[tokio::test]
async fn test_storage_list_objects() {
    async fn inner(storage: &Storage) -> Result<()> {
        init(storage).await?;

        storage.list_objects().await?;

        Ok(())
    }

    let storage = get_storage();

    let result = inner(&storage).await;
    reset(&storage).await;
    result.unwrap();

    sleep_async(1).await;
}

#[tokio::test]
async fn test_storage_remove_object() {
    async fn inner(storage: &Storage) -> Result<()> {
        let remote_path = "test_remove_object.txt";

        init(storage).await?;

        upload_data(storage, remote_path).await?;
        storage.remove_object(remote_path).await?;

        Ok(())
    }

    let storage = get_storage();

    let result = inner(&storage).await;
    reset(&storage).await;
    result.unwrap();

    sleep_async(1).await;
}

#[tokio::test]
async fn test_storage_remove_objects_all() {
    async fn inner(storage: &Storage) -> Result<()> {
        let remote_path = "test_remove_objects_all.txt";

        init(storage).await?;

        upload_data(storage, remote_path).await?;
        storage.remove_objects_all().await?;

        Ok(())
    }

    let storage = get_storage();

    let result = inner(&storage).await;
    reset(&storage).await;
    result.unwrap();

    sleep_async(1).await;
}

#[tokio::test]
async fn test_storage_remove_bucket() {
    let storage = get_storage();

    init(&storage).await.unwrap();

    storage._remove_bucket().await.unwrap();

    sleep_async(1).await;
}

#[tokio::test]
async fn test_storage_get_download_url() {
    async fn inner(storage: &Storage) -> Result<()> {
        let remote_path = "get_download_url.txt";

        init(storage).await?;

        upload_data(storage, remote_path).await?;
        storage.get_download_url(remote_path).await?;

        Ok(())
    }

    let storage = get_storage();

    let result = inner(&storage).await;
    reset(&storage).await;
    result.unwrap();

    sleep_async(1).await;
}
