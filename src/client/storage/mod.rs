/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod local;
mod minio;
pub mod models;
#[cfg(test)]
pub mod tests;
mod utils;

use local::LocalStorage;
use minio::Minio;
use models::{Part, StorageClient};

use crate::env::StorageEnv;
use crate::error::Result;

#[derive(Clone)]
pub struct Storage {
    pub client: StorageClient,
}

impl Storage {
    pub async fn new(config: &StorageEnv) -> Result<Self> {
        let client = match config {
            StorageEnv::Minio(config) => StorageClient::Minio(Minio::new(config)?),
            StorageEnv::LocalStorage(config) => StorageClient::Local(LocalStorage::new(config)),
        };

        Ok(Self { client })
    }

    pub async fn init(&self) -> Result<()> {
        match &self.client {
            StorageClient::Local(storage) => storage.init().await,
            StorageClient::Minio(storage) => storage.init().await,
        }
    }

    pub async fn get_download_url(&self, object: &str) -> Result<String> {
        match &self.client {
            StorageClient::Local(storage) => storage.get_download_url(object).await,
            StorageClient::Minio(storage) => storage.get_download_url(object).await,
        }
    }

    pub async fn remove_object(&self, object: &str) -> Result<()> {
        match &self.client {
            StorageClient::Local(storage) => storage.remove_object(object).await,
            StorageClient::Minio(storage) => storage.remove_object(object).await,
        }
    }

    pub async fn remove_objects_all(&self) -> Result<()> {
        match &self.client {
            StorageClient::Local(storage) => storage.remove_objects_all().await,
            StorageClient::Minio(storage) => storage.remove_objects_all().await,
        }
    }

    pub async fn create_multipart_upload_id(&self, object: &str) -> Result<String> {
        match &self.client {
            StorageClient::Local(storage) => storage.create_multipart_upload_id(object).await,
            StorageClient::Minio(storage) => storage.create_multipart_upload_id(object).await,
        }
    }

    pub async fn multipart_upload(
        &self,
        object: &str,
        upload_id: &str,
        part_data: &[u8],
        part_number: u16,
    ) -> Result<Part> {
        match &self.client {
            StorageClient::Local(storage) => {
                storage
                    .multipart_upload(object, upload_id, part_data, part_number)
                    .await
            }
            StorageClient::Minio(storage) => {
                storage
                    .multipart_upload(object, upload_id, part_data, part_number)
                    .await
            }
        }
    }

    pub async fn complete_multipart_upload(
        &self,
        object: &str,
        upload_id: &str,
        parts: &Vec<Part>,
    ) -> Result<()> {
        match &self.client {
            StorageClient::Local(storage) => {
                storage
                    .complete_multipart_upload(object, upload_id, parts)
                    .await
            }
            StorageClient::Minio(storage) => {
                storage
                    .complete_multipart_upload(object, upload_id, parts)
                    .await
            }
        }
    }
}
