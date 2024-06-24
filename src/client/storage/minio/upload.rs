/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/
use minio::s3::args::{CompleteMultipartUploadArgs, CreateMultipartUploadArgs, UploadPartArgs};

use super::super::models::Part;
use super::Minio;
use crate::error::ErrorType::InternalServerError;
use crate::error::{Error, Result};

impl Minio {
    pub async fn create_multipart_upload_id(&self, remote_path: &str) -> Result<String> {
        let args = CreateMultipartUploadArgs::new(&self.bucket, remote_path).map_err(|e| {
            Error::context(
                InternalServerError,
                e,
                "failed to create multipart upload args",
            )
        })?;

        let multipart_upload_response =
            self.client
                .create_multipart_upload(&args)
                .await
                .map_err(|e| {
                    Error::context(
                        InternalServerError,
                        e,
                        "failed to get multipart upload response",
                    )
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
                    Error::context(InternalServerError, e, "failed to create upload part args")
                })?;

        let response = self
            .client
            .upload_part(&args)
            .await
            .map_err(|e| Error::context(InternalServerError, e, "failed to upload part"))?;

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
        let parts = parts
            .iter()
            .map(|part| minio::s3::types::Part {
                number: part.number,
                etag: part.etag.clone(),
            })
            .collect::<Vec<minio::s3::types::Part>>();

        let args = CompleteMultipartUploadArgs::new(&self.bucket, remote_path, upload_id, &parts)
            .map_err(|e| {
            Error::context(
                InternalServerError,
                e,
                "failed to create complete multipart upload args",
            )
        })?;

        self.client
            .complete_multipart_upload(&args)
            .await
            .map_err(|e| {
                Error::context(
                    InternalServerError,
                    e,
                    "failed to complete multipart upload",
                )
            })?;

        Ok(())
    }
}
