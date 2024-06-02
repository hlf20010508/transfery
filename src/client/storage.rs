/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use minio::s3::args::{
    BucketArgs, BucketExistsArgs, CompleteMultipartUploadArgs, CreateMultipartUploadArgs,
    GetPresignedObjectUrlArgs, ListObjectsV2Args, MakeBucketArgs, ObjectVersionArgs,
    RemoveObjectsArgs, UploadPartArgs,
};
use minio::s3::client::Client;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use minio::s3::types::{DeleteObject, Item, Part};

use crate::error::Error::{
    StorageClientError, StorageInitError, StorageObjectError, UrlParseError,
};
use crate::error::Result;

// s3 minimum allowed size is 5MB
pub static _PART_SIZE: u32 = 5 * 1024 * 1024; // 5MB

#[derive(Debug, Clone)]
pub struct Storage {
    pub client: Client,
    pub bucket: String,
}

impl Storage {
    pub fn new(endpoint: &str, username: &str, password: &str, bucket: &str) -> Result<Self> {
        let base_url = endpoint
            .parse::<BaseUrl>()
            .map_err(|e| UrlParseError(format!("Minio endpoint parse failed: {}", e)))?;

        let static_provider = StaticProvider::new(username, password, None);

        let client = Client::new(base_url, Some(Box::new(static_provider)), None, None)
            .map_err(|e| StorageClientError(format!("Minio client creation failed: {}", e)))?;

        Ok(Self {
            client,
            bucket: bucket.to_string(),
        })
    }

    pub async fn init(&self) -> Result<()> {
        self.create_buffer_if_not_exists().await?;

        Ok(())
    }

    async fn create_buffer_if_not_exists(&self) -> Result<()> {
        let args = MakeBucketArgs::new(&self.bucket).map_err(|e| {
            StorageInitError(format!("Minio bucket name invalid when making: {}", e))
        })?;

        if !self.is_bucket_exists().await? {
            self.client.make_bucket(&args).await.map_err(|e| {
                StorageInitError(format!("Minio making bucket await failed: {}", e))
            })?;
        }

        Ok(())
    }

    async fn is_bucket_exists(&self) -> Result<bool> {
        let args = BucketExistsArgs::new(&self.bucket).map_err(|e| {
            StorageInitError(format!(
                "Minio bucket name invalid when checking existence: {}",
                e
            ))
        })?;

        let exists = self.client.bucket_exists(&args).await.map_err(|e| {
            StorageInitError(format!(
                "Minio checking bucket existence await failed: {}",
                e
            ))
        })?;

        Ok(exists)
    }

    pub async fn create_multipart_upload_id(&self, remote_path: &str) -> Result<String> {
        let args = CreateMultipartUploadArgs::new(&self.bucket, remote_path).map_err(|e| {
            StorageObjectError(format!(
                "Storage create multipart upload args failed: {}",
                e
            ))
        })?;

        let multipart_upload_response =
            self.client
                .create_multipart_upload(&args)
                .await
                .map_err(|e| {
                    StorageObjectError(format!(
                        "Storage get multipart upload response failed: {}",
                        e
                    ))
                })?;

        let upload_id = multipart_upload_response.upload_id;

        Ok(upload_id)
    }

    pub async fn multipart_upload(
        &self,
        remote_path: &str,
        upload_id: &str,
        part_data: &[u8],
        part_number: u16,
    ) -> Result<Part> {
        let args =
            UploadPartArgs::new(&self.bucket, remote_path, upload_id, part_number, part_data)
                .map_err(|e| {
                    StorageObjectError(format!("Storage create upload part args failed: {}", e))
                })?;

        let response = self
            .client
            .upload_part(&args)
            .await
            .map_err(|e| StorageObjectError(format!("Storage upload part failed: {}", e)))?;

        let etag = response.etag;

        Ok(Part {
            number: part_number,
            etag,
        })
    }

    pub async fn complete_multipart_upload(
        &self,
        remote_path: &str,
        upload_id: &str,
        parts: &Vec<Part>,
    ) -> Result<()> {
        let args = CompleteMultipartUploadArgs::new(&self.bucket, remote_path, upload_id, parts)
            .map_err(|e| {
                StorageObjectError(format!(
                    "Storage create complete multipart upload args failed: {}",
                    e
                ))
            })?;

        self.client
            .complete_multipart_upload(&args)
            .await
            .map_err(|e| {
                StorageObjectError(format!("Storage complete multipart upload failed: {}", e))
            })?;

        Ok(())
    }

    async fn list_objects(&self) -> Result<Vec<Item>> {
        let args = ListObjectsV2Args::new(&self.bucket).map_err(|e| {
            StorageObjectError(format!("Storage create list objects v2 args failed: {}", e))
        })?;

        let response = self
            .client
            .list_objects_v2(&args)
            .await
            .map_err(|e| StorageObjectError(format!("Storage list objects failed: {}", e)))?;

        let objects = response.contents;

        Ok(objects)
    }

    pub async fn remove_object(&self, remote_path: &str) -> Result<()> {
        let args = ObjectVersionArgs::new(&self.bucket, remote_path).map_err(|e| {
            StorageObjectError(format!("Storage create object version args failed: {}", e))
        })?;

        self.client
            .remove_object(&args)
            .await
            .map_err(|e| StorageObjectError(format!("Storage remove object failed: {}", e)))?;

        Ok(())
    }

    pub async fn remove_objects_all(&self) -> Result<()> {
        let objects: Vec<Item> = self.list_objects().await?;
        let mut objects_delete: Vec<DeleteObject> = Vec::new();

        for object in objects.iter() {
            let object_delete = DeleteObject {
                name: &object.name,
                version_id: None,
            };

            objects_delete.push(object_delete);
        }

        let mut objects_delete_iter = objects_delete.iter();

        let mut args =
            RemoveObjectsArgs::new(&self.bucket, &mut objects_delete_iter).map_err(|e| {
                StorageObjectError(format!("Storage create remove objects args failed: {}", e))
            })?;

        let response = self
            .client
            .remove_objects(&mut args)
            .await
            .map_err(|e| StorageObjectError(format!("Storage remove objects failed: {}", e)))?;

        if response.errors.len() == 0 {
            return Ok(());
        } else {
            return Err(StorageObjectError(format!(
                "Storage remove objects success but got errors:\nobjects: {:#?}\nerrors: {:#?}",
                response.objects, response.errors
            )));
        }
    }

    async fn _remove_bucket(&self) -> Result<()> {
        self.remove_objects_all().await?;

        let args = BucketArgs::new(&self.bucket)
            .map_err(|e| StorageObjectError(format!("Storage create bucket args failed: {}", e)))?;

        self.client
            .remove_bucket(&args)
            .await
            .map_err(|e| StorageObjectError(format!("Storage remove bucket failed: {}", e)))?;

        Ok(())
    }

    pub async fn get_download_url(&self, remote_path: &str) -> Result<String> {
        let args =
            GetPresignedObjectUrlArgs::new(&self.bucket, remote_path, http::method::Method::GET)
                .map_err(|e| {
                    StorageObjectError(format!(
                        "Storage create get presigned object url args failed: {}",
                        e
                    ))
                })?;

        let response = self
            .client
            .get_presigned_object_url(&args)
            .await
            .map_err(|e| {
                StorageObjectError(format!("Storage get presigned object url failed: {}", e))
            })?;

        Ok(response.url)
    }
}

#[cfg(test)]
pub mod tests {
    use minio::s3::args::PutObjectArgs;
    use std::io::Cursor;

    use super::*;

    use crate::env::tests::get_env;
    use crate::utils::tests::{sleep, sleep_async};

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

        let mut args =
            PutObjectArgs::new(&storage.bucket, remote_path, &mut data, Some(size), None).map_err(
                |e| StorageObjectError(format!("Storage create put object args failed: {}", e)),
            )?;

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

            for (part_number, part_data) in data.chunks(_PART_SIZE as usize).enumerate() {
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
}
