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

use crate::error::Error::{StorageClientError, StorageInitError, UrlParseError};
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
}
