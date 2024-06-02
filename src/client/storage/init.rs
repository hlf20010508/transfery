/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use minio::s3::args::{BucketExistsArgs, MakeBucketArgs};
use minio::s3::client::Client;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;

use super::Storage;

use crate::error::Error::{StorageClientError, StorageInitError, UrlParseError};
use crate::error::Result;

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

    pub async fn create_buffer_if_not_exists(&self) -> Result<()> {
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

    pub async fn is_bucket_exists(&self) -> Result<bool> {
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
}
