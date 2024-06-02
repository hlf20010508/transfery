/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/
use minio::s3::args::{CompleteMultipartUploadArgs, CreateMultipartUploadArgs, UploadPartArgs};
use minio::s3::types::Part;

use super::Storage;

use crate::error::Error::StorageObjectError;
use crate::error::Result;

impl Storage {
    pub async fn create_multipart_upload_id(&self, remote_path: &str) -> Result<String> {
        let args = CreateMultipartUploadArgs::new(&self.bucket, remote_path).map_err(|e| {
            StorageObjectError(format!(
                "Storage create multipart upload args failed: {}",
                e
            ))
        })?;

        let multipart_upload_response =
            self.client
                .create_multipart_upload(&args)
                .await
                .map_err(|e| {
                    StorageObjectError(format!(
                        "Storage get multipart upload response failed: {}",
                        e
                    ))
                })?;

        let upload_id = multipart_upload_response.upload_id;

        Ok(upload_id)
    }

    pub async fn multipart_upload(
        &self,
        remote_path: &str,
        upload_id: &str,
        part_data: &[u8],
        part_number: u16,
    ) -> Result<Part> {
        let args =
            UploadPartArgs::new(&self.bucket, remote_path, upload_id, part_number, part_data)
                .map_err(|e| {
                    StorageObjectError(format!("Storage create upload part args failed: {}", e))
                })?;

        let response = self
            .client
            .upload_part(&args)
            .await
            .map_err(|e| StorageObjectError(format!("Storage upload part failed: {}", e)))?;

        let etag = response.etag;

        Ok(Part {
            number: part_number,
            etag,
        })
    }

    pub async fn complete_multipart_upload(
        &self,
        remote_path: &str,
        upload_id: &str,
        parts: &Vec<Part>,
    ) -> Result<()> {
        let args = CompleteMultipartUploadArgs::new(&self.bucket, remote_path, upload_id, parts)
            .map_err(|e| {
                StorageObjectError(format!(
                    "Storage create complete multipart upload args failed: {}",
                    e
                ))
            })?;

        self.client
            .complete_multipart_upload(&args)
            .await
            .map_err(|e| {
                StorageObjectError(format!("Storage complete multipart upload failed: {}", e))
            })?;

        Ok(())
    }
}
