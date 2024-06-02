/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use minio::s3::args::ListObjectsV2Args;
use minio::s3::types::Item;

use super::Storage;

use crate::error::Error::StorageObjectError;
use crate::error::Result;

impl Storage {
    pub async fn list_objects(&self) -> Result<Vec<Item>> {
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
}
