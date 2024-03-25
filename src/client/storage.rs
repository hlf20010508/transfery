/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use minio::s3::args::{BucketArgs, BucketExistsArgs, CreateMultipartUploadArgs, MakeBucketArgs};
use minio::s3::client::Client;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;

use crate::error::Error::{
    StorageClientError, StorageInitError, StorageObjectError, UrlParseError,
};
use crate::error::Result;

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
        let exists = self
            .client
            .bucket_exists(&BucketExistsArgs::new(&self.bucket).map_err(|e| {
                StorageInitError(format!(
                    "Minio bucket name invalid when checking existence: {}",
                    e.to_string()
                ))
            })?)
            .await
            .map_err(|e| {
                StorageInitError(format!(
                    "Minio checking bucket existence await failed: {}",
                    e.to_string()
                ))
            })?;

        if !exists {
            self.client
                .make_bucket(&MakeBucketArgs::new(&self.bucket).map_err(|e| {
                    StorageInitError(format!(
                        "Minio bucket name invalid when making: {}",
                        e.to_string()
                    ))
                })?)
                .await
                .map_err(|e| {
                    StorageInitError(format!(
                        "Minio making bucket await failed: {}",
                        e.to_string()
                    ))
                })?;
        }

        Ok(())
    }

    pub async fn create_multipart_upload_id(&self, remote_path: &str) -> Result<String> {
        // let headers_map = Multimap::new()
        //     .insert(k, v)
        let multipart_upload_args = CreateMultipartUploadArgs::new(&self.bucket, remote_path)
            .map_err(|e| {
                StorageObjectError(format!(
                    "Storage create multipart upload args failed: {}",
                    e.to_string()
                ))
            })?;

        let multipart_upload_response = self
            .client
            .create_multipart_upload(&multipart_upload_args)
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

    pub async fn remove_bucket(&self) -> Result<()> {
        let bucket_args = BucketArgs::new(&self.bucket).map_err(|e| {
            StorageObjectError(format!(
                "Storage create bucket args failed: {}",
                e.to_string()
            ))
        })?;

        self.client.remove_bucket(&bucket_args).await.map_err(|e| {
            StorageObjectError(format!("Storage remove bucket failed: {}", e.to_string()))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn test_new() {
        let storage = Storage::new(
            "play.min.io",
            "Q3AM3UQ867SPQQA43P2F",
            "zuf+tfteSlswRu7BJ86wekitnifILbZam1KYY3TG",
            "transfery",
        );

        assert!(storage.is_ok());
    }

    #[actix_web::test]
    async fn test_init() {
        let storage = get_storage();

        let result = storage.init().await;

        reset(&storage).await;

        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn test_create_buffer_if_not_exists() {
        let storage = get_storage();

        let result = storage.create_buffer_if_not_exists().await;

        reset(&storage).await;

        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn test_create_multipart_upload_id() {
        let storage = get_storage();

        init(&storage).await;

        let upload_id = storage
            .create_multipart_upload_id("test-create-multipart-upload-id")
            .await;

        reset(&storage).await;

        assert!(upload_id.is_ok());
    }

    #[actix_web::test]
    async fn test_remove_bucket() {
        let storage = get_storage();

        init(&storage).await;

        let result = storage.remove_bucket().await;

        assert!(result.is_ok());
    }
}
