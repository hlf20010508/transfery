/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use tokio::fs;

use super::utils::LocalStorageUtils;
use super::LocalStorage;
use crate::error::ErrorType::InternalServerError;
use crate::error::{Error, Result};

impl LocalStorage {
    pub async fn remove_object(&self, file_name: &str) -> Result<()> {
        fs::remove_file(self.get_path(file_name))
            .await
            .map_err(|e| {
                Error::context(
                    InternalServerError,
                    e,
                    "failed to remove object in local storage",
                )
            })?;

        Ok(())
    }

    pub async fn remove_objects_all(&self) -> Result<()> {
        self.remove_dir().await?;
        self.create_dir().await?;

        Ok(())
    }
}
