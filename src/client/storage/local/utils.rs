/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use std::path::PathBuf;
use tokio::fs;

use super::LocalStorage;
use crate::error::ErrorType::InternalServerError;
use crate::error::{Error, Result};

pub trait LocalStorageUtils {
    fn get_path(&self, name: &str) -> PathBuf;

    async fn create_dir(&self) -> Result<()>;

    async fn remove_dir(&self) -> Result<()>;
}

impl LocalStorageUtils for LocalStorage {
    fn get_path(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }

    async fn create_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.path).await.map_err(|e| {
            Error::context(
                InternalServerError,
                e,
                "failed to create local storage directory",
            )
        })?;

        Ok(())
    }

    async fn remove_dir(&self) -> Result<()> {
        fs::remove_dir_all(&self.path).await.map_err(|e| {
            Error::context(
                InternalServerError,
                e,
                "failed to remove local storage directory",
            )
        })?;

        Ok(())
    }
}
