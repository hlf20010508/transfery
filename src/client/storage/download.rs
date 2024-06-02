/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use minio::s3::args::GetPresignedObjectUrlArgs;

use super::Storage;

use crate::error::Error::StorageObjectError;
use crate::error::Result;

impl Storage {
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
