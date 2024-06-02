/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use minio::s3::args::{BucketArgs, ObjectVersionArgs, RemoveObjectsArgs};
use minio::s3::types::{DeleteObject, Item};

use super::Storage;

use crate::error::Error::StorageObjectError;
use crate::error::Result;

impl Storage {
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

    pub async fn _remove_bucket(&self) -> Result<()> {
        self.remove_objects_all().await?;

        let args = BucketArgs::new(&self.bucket)
            .map_err(|e| StorageObjectError(format!("Storage create bucket args failed: {}", e)))?;

        self.client
            .remove_bucket(&args)
            .await
            .map_err(|e| StorageObjectError(format!("Storage remove bucket failed: {}", e)))?;

        Ok(())
    }
}
