/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use minio::s3::args::GetPresignedObjectUrlArgs;

use super::Minio;
use crate::error::ErrorType::InternalServerError;
use crate::error::{Error, Result};

impl Minio {
    pub async fn get_download_url(&self, remote_path: &str) -> Result<String> {
        let args =
            GetPresignedObjectUrlArgs::new(&self.bucket, remote_path, http::method::Method::GET)
                .map_err(|e| {
                    Error::context(
                        InternalServerError,
                        e,
                        "failed to create get presigned object url args",
                    )
                })?;

        let response = self
            .client
            .get_presigned_object_url(&args)
            .await
            .map_err(|e| {
                Error::context(InternalServerError, e, "failed to get presigned object url")
            })?;

        Ok(response.url)
    }
}
