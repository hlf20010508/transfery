/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;

use super::LocalStorage;
use crate::env::LocalStorageEnv;
use crate::error::ErrorType::InternalServerError;
use crate::error::{Error, Result};
use crate::utils::get_current_timestamp;

impl LocalStorage {
    pub fn new(LocalStorageEnv { path }: &LocalStorageEnv) -> Self {
        let path = PathBuf::from(path);

        let storage = Self {
            path,
            tasks: Arc::new(Mutex::new(HashMap::new())),
        };

        let storage_clone = storage.clone();

        tokio::spawn(async move {
            Self::task_watcher(storage_clone).await.unwrap();
        });

        storage
    }

    pub async fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.path).await.map_err(|e| {
            Error::context(
                InternalServerError,
                e,
                "failed to create local storage directory",
            )
        })
    }

    async fn task_watcher(storage: LocalStorage) -> Result<()> {
        let interval = 5 * 60; // run every 5 minutes

        loop {
            let now = get_current_timestamp();

            let mut expired_uploads = Vec::new();

            {
                let mut tasks_guard = storage.tasks.lock().await;

                for (upload_id, task_info) in tasks_guard.iter() {
                    if task_info.expiration_timestamp <= now {
                        expired_uploads.push((task_info.file_name.clone(), upload_id.clone()));
                    }
                }

                for (file_name, upload_id) in &expired_uploads {
                    storage.cleanup_part_files(&file_name, &upload_id).await?;
                    tasks_guard.remove(upload_id);
                }
            }
            // release lock before sleep

            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
        }
    }

    async fn cleanup_part_files(&self, file_name: &str, upload_id: &str) -> Result<()> {
        fs::remove_dir_all(self.get_parts_dir(file_name, upload_id))
            .await
            .map_err(|e| {
                Error::context(InternalServerError, e, "failed to remove parts directory")
            })?;

        Ok(())
    }
}
