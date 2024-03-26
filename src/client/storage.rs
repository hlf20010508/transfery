/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use actix_web::http::Method;
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
pub static PART_SIZE: u32 = 5 * 1024 * 1024; // 5MB

#[derive(Debug)]
pub struct Storage {
    client: Client,
    bucket: String,
}

impl Storage {
    pub fn new(endpoint: &str, username: &str, password: &str, bucket: &str) -> Result<Self> {
        let base_url = endpoint.parse::<BaseUrl>().map_err(|e| {
            UrlParseError(format!("Minio endpoint parse failed: {}", e.to_string()))
        })?;

        let static_provider = StaticProvider::new(username, password, None);

        let client =
            Client::new(base_url, Some(Box::new(static_provider)), None, None).map_err(|e| {
                StorageClientError(format!("Minio client creation failed: {}", e.to_string()))
            })?;

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
            StorageInitError(format!(
                "Minio bucket name invalid when making: {}",
                e.to_string()
            ))
        })?;

        if !self.is_bucket_exists().await? {
            self.client.make_bucket(&args).await.map_err(|e| {
                StorageInitError(format!(
                    "Minio making bucket await failed: {}",
                    e.to_string()
                ))
            })?;
        }

        Ok(())
    }

    async fn is_bucket_exists(&self) -> Result<bool> {
        let args = BucketExistsArgs::new(&self.bucket).map_err(|e| {
            StorageInitError(format!(
                "Minio bucket name invalid when checking existence: {}",
                e.to_string()
            ))
        })?;

        let exists = self.client.bucket_exists(&args).await.map_err(|e| {
            StorageInitError(format!(
                "Minio checking bucket existence await failed: {}",
                e.to_string()
            ))
        })?;

        Ok(exists)
    }

    pub async fn create_multipart_upload_id(&self, remote_path: &str) -> Result<String> {
        let args = CreateMultipartUploadArgs::new(&self.bucket, remote_path).map_err(|e| {
            StorageObjectError(format!(
                "Storage create multipart upload args failed: {}",
                e.to_string()
            ))
        })?;

        let multipart_upload_response =
            self.client
                .create_multipart_upload(&args)
                .await
                .map_err(|e| {
                    StorageObjectError(format!(
                        "Storage get multipart upload response failed: {}",
                        e.to_string()
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
                    StorageObjectError(format!(
                        "Storage create upload part args failed: {}",
                        e.to_string()
                    ))
                })?;

        let response = self.client.upload_part(&args).await.map_err(|e| {
            StorageObjectError(format!("Storage upload part failed: {}", e.to_string()))
        })?;

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
                    e.to_string()
                ))
            })?;

        self.client
            .complete_multipart_upload(&args)
            .await
            .map_err(|e| {
                StorageObjectError(format!(
                    "Storage complete multipart upload failed: {}",
                    e.to_string()
                ))
            })?;

        Ok(())
    }

    async fn list_objects(&self) -> Result<Vec<Item>> {
        let args = ListObjectsV2Args::new(&self.bucket).map_err(|e| {
            StorageObjectError(format!(
                "Storage create list objects v2 args failed: {}",
                e.to_string()
            ))
        })?;

        let response = self.client.list_objects_v2(&args).await.map_err(|e| {
            StorageObjectError(format!("Storage list objects failed: {}", e.to_string()))
        })?;

        let objects = response.contents;

        Ok(objects)
    }

    pub async fn remove_object(&self, remote_path: &str) -> Result<()> {
        let args = ObjectVersionArgs::new(&self.bucket, remote_path).map_err(|e| {
            StorageObjectError(format!(
                "Storage create object version args failed: {}",
                e.to_string()
            ))
        })?;

        self.client.remove_object(&args).await.map_err(|e| {
            StorageObjectError(format!("Storage remove object failed: {}", e.to_string()))
        })?;

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
                StorageObjectError(format!(
                    "Storage create remove objects args failed: {}",
                    e.to_string()
                ))
            })?;

        let response = self.client.remove_objects(&mut args).await.map_err(|e| {
            StorageObjectError(format!("Storage remove objects failed: {}", e.to_string()))
        })?;

        if response.errors.len() == 0 {
            return Ok(());
        } else {
            return Err(StorageObjectError(format!(
                "Storage remove objects success but got errors:\nobjects: {:#?}\nerrors: {:#?}",
                response.objects, response.errors
            )));
        }
    }

    pub async fn remove_bucket(&self) -> Result<()> {
        self.remove_objects_all().await?;

        let args = BucketArgs::new(&self.bucket).map_err(|e| {
            StorageObjectError(format!(
                "Storage create bucket args failed: {}",
                e.to_string()
            ))
        })?;

        self.client.remove_bucket(&args).await.map_err(|e| {
            StorageObjectError(format!("Storage remove bucket failed: {}", e.to_string()))
        })?;

        Ok(())
    }

    pub async fn get_download_url(&self, remote_path: &str) -> Result<String> {
        let args = GetPresignedObjectUrlArgs::new(&self.bucket, remote_path, Method::GET).map_err(
            |e| {
                StorageObjectError(format!(
                    "Storage create get presigned object url args failed: {}",
                    e.to_string()
                ))
            },
        )?;

        let response = self
            .client
            .get_presigned_object_url(&args)
            .await
            .map_err(|e| {
                StorageObjectError(format!(
                    "Storage get presigned object url failed: {}",
                    e.to_string()
                ))
            })?;

        Ok(response.url)
    }
}

#[cfg(test)]
mod tests {
    use minio::s3::args::PutObjectArgs;
    use std::io::Cursor;

    use super::*;

    fn get_storage() -> Storage {
        let storage = Storage::new(
            "play.min.io",
            "Q3AM3UQ867SPQQA43P2F",
            "zuf+tfteSlswRu7BJ86wekitnifILbZam1KYY3TG",
            "transfery",
        )
        .unwrap();

        storage
    }

    async fn init(storage: &Storage) {
        storage.init().await.unwrap();
    }

    async fn reset(storage: &Storage) {
        storage.remove_bucket().await.unwrap();
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

    async fn upload_data(storage: &Storage, remote_path: &str) {
        let mut data = Cursor::new(fake_data());
        let size = data.clone().into_inner().len();

        let mut args =
            PutObjectArgs::new(&storage.bucket, remote_path, &mut data, Some(size), None).unwrap();

        storage.client.put_object(&mut args).await.unwrap();
    }

    #[test]
    fn test_new() {
        let storage = Storage::new(
            "play.min.io",
            "Q3AM3UQ867SPQQA43P2F",
            "zuf+tfteSlswRu7BJ86wekitnifILbZam1KYY3TG",
            "transfery",
        );

        assert!(storage.is_ok(), "{}", storage.unwrap_err());
    }

    #[actix_web::test]
    async fn test_init() {
        let storage = get_storage();

        let result = storage.init().await;

        reset(&storage).await;

        assert!(result.is_ok(), "{}", result.unwrap_err());
    }

    #[actix_web::test]
    async fn test_create_buffer_if_not_exists() {
        let storage = get_storage();

        let result = storage.create_buffer_if_not_exists().await;

        reset(&storage).await;

        assert!(result.is_ok(), "{}", result.unwrap_err());
    }

    #[actix_web::test]
    async fn test_is_bucket_exists() {
        let storage = get_storage();

        let result = storage.is_bucket_exists().await;

        assert!(result.is_ok(), "{}", result.unwrap_err());

        init(&storage).await;

        let result = storage.is_bucket_exists().await;

        reset(&storage).await;

        assert!(result.is_ok(), "{}", result.unwrap_err());
    }

    #[actix_web::test]
    async fn test_create_multipart_upload_id() {
        let storage = get_storage();

        init(&storage).await;

        let remote_path = "test-create-multipart-upload-id.txt";

        let upload_id = storage.create_multipart_upload_id(remote_path).await;

        reset(&storage).await;

        assert!(upload_id.is_ok(), "{}", upload_id.unwrap_err());
    }

    #[actix_web::test]
    async fn test_multipart_upload() {
        let storage = get_storage();

        init(&storage).await;

        let remote_path = "test-multipart-upload.txt";

        let upload_id = storage
            .create_multipart_upload_id(remote_path)
            .await
            .unwrap();

        let data = fake_data();

        let part_number: u16 = 1;

        let part = storage
            .multipart_upload(remote_path, &upload_id, &data, part_number)
            .await;

        reset(&storage).await;

        assert!(part.is_ok(), "{}", part.unwrap_err());
    }

    #[actix_web::test]
    async fn test_complete_multipart_upload() {
        let storage = get_storage();

        init(&storage).await;

        let remote_path = "test-complete-multipart-upload.txt";

        let upload_id = storage
            .create_multipart_upload_id(remote_path)
            .await
            .unwrap();

        let data = fake_data();

        let mut parts: Vec<Part> = Vec::new();

        for (part_number, part_data) in data.chunks(PART_SIZE as usize).enumerate() {
            let part_number = part_number as u16 + 1;

            let part = storage
                .multipart_upload(remote_path, &upload_id, part_data, part_number)
                .await
                .unwrap();

            parts.push(part);
        }

        let result = storage
            .complete_multipart_upload(remote_path, &upload_id, &parts)
            .await;

        reset(&storage).await;

        assert!(result.is_ok(), "{}", result.unwrap_err());
    }

    #[actix_web::test]
    async fn test_list_objects() {
        let storage = get_storage();

        init(&storage).await;

        let result = storage.list_objects().await;

        reset(&storage).await;

        assert!(result.is_ok(), "{}", result.unwrap_err());
    }

    #[actix_web::test]
    async fn test_remove_object() {
        let storage = get_storage();

        init(&storage).await;

        let remote_path = "test_remove_object.txt";
        upload_data(&storage, remote_path).await;

        let result = storage.remove_object(remote_path).await;

        reset(&storage).await;

        assert!(result.is_ok(), "{}", result.unwrap_err());
    }

    #[actix_web::test]
    async fn test_remove_objects_all() {
        let storage = get_storage();

        init(&storage).await;

        let remote_path = "test_remove_objects_all.txt";
        upload_data(&storage, remote_path).await;

        let result = storage.remove_objects_all().await;

        reset(&storage).await;

        assert!(result.is_ok(), "{}", result.unwrap_err());
    }

    #[actix_web::test]
    async fn test_remove_bucket() {
        let storage = get_storage();

        init(&storage).await;

        let result = storage.remove_bucket().await;

        assert!(result.is_ok(), "{}", result.unwrap_err());
    }

    #[actix_web::test]
    async fn test_get_download_url() {
        let storage = get_storage();

        init(&storage).await;

        let remote_path = "get_download_url.txt";
        upload_data(&storage, remote_path).await;

        let result = storage.get_download_url(remote_path).await;

        reset(&storage).await;

        assert!(result.is_ok(), "{}", result.unwrap_err());
    }
}
