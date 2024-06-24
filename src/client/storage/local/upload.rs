/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use std::path::PathBuf;

use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

use super::super::models::Part;
use super::models::TaskInfo;
use super::utils::LocalStorageUtils;
use super::LocalStorage;
use crate::error::ErrorType::InternalServerError;
use crate::error::{Error, Result};
use crate::utils::get_current_timestamp;

impl LocalStorage {
    pub async fn create_multipart_upload_id(&self, file_name: &str) -> Result<String> {
        let upload_id = Uuid::new_v4().to_string();
        let expiration_timestamp = get_current_timestamp() + 1000 * 24 * 3600; // 1 day

        self.tasks.lock().await.insert(
            upload_id.clone(),
            TaskInfo {
                file_name: file_name.to_string(),
                parts: Vec::new(),
                expiration_timestamp,
            },
        );

        fs::create_dir_all(self.get_parts_dir(file_name, &upload_id))
            .await
            .map_err(|e| {
                Error::context(InternalServerError, e, "failed to create parts directory")
            })?;

        Ok(upload_id)
    }

    pub async fn multipart_upload(
        &self,
        file_name: &str,
        upload_id: &str,
        part_data: &[u8],
        part_number: u16,
    ) -> Result<Part> {
        let part_path = self.get_part_path(file_name, upload_id, part_number);

        File::create(part_path)
            .await
            .map_err(|e| Error::context(InternalServerError, e, "failed t o create part file"))?
            .write_all(part_data)
            .await
            .map_err(|e| Error::context(InternalServerError, e, "failed to write part file"))?;

        let etag = format!("{:x}", md5::compute(part_data));

        let part = Part {
            number: part_number,
            etag,
        };

        self.tasks
            .lock()
            .await
            .get_mut(upload_id)
            .unwrap()
            .parts
            .push(part.clone());

        Ok(part)
    }

    pub async fn complete_multipart_upload(
        &self,
        file_name: &str,
        upload_id: &str,
        parts: &Vec<Part>,
    ) -> Result<()> {
        let file_path = self.get_path(file_name);

        let mut final_file = File::create(file_path)
            .await
            .map_err(|e| Error::context(InternalServerError, e, "failed to create final file"))?;

        for part in parts {
            let part_path = self.get_part_path(file_name, upload_id, part.number);

            let mut part_file = File::open(&part_path)
                .await
                .map_err(|e| Error::context(InternalServerError, e, "failed to open part file"))?;

            let mut buffer = Vec::new();

            part_file.read_to_end(&mut buffer).await.map_err(|e| {
                Error::context(InternalServerError, e, "failed to write part to buffer")
            })?;

            final_file.write_all(&buffer).await.map_err(|e| {
                Error::context(InternalServerError, e, "failed to write part to final file")
            })?;

            fs::remove_file(part_path).await.map_err(|e| {
                Error::context(InternalServerError, e, "failed to remove part file")
            })?;
        }

        fs::remove_dir_all(self.get_parts_dir(file_name, upload_id))
            .await
            .map_err(|e| {
                Error::context(InternalServerError, e, "failed to remove parts directory")
            })?;

        final_file
            .flush()
            .await
            .map_err(|e| Error::context(InternalServerError, e, "failed to flush final file"))?;

        self.tasks.lock().await.remove(upload_id);

        Ok(())
    }

    pub fn get_parts_dir(&self, file_name: &str, upload_id: &str) -> PathBuf {
        self.get_path(&format!("__PARTS__{}_{}", file_name, upload_id))
    }

    fn get_part_path(&self, file_name: &str, upload_id: &str, part_number: u16) -> PathBuf {
        self.get_parts_dir(file_name, upload_id)
            .join(&format!("__PART__{}_{}", file_name, part_number))
    }
}
