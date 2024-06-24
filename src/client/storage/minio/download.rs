/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::response::Response;
use minio::s3::args::GetPresignedObjectUrlArgs;
use minio::s3::utils::urlencode;

use super::Minio;
use crate::error::ErrorType::InternalServerError;
use crate::error::{Error, Result};

impl Minio {
    pub async fn get_download_response(&self, remote_path: &str) -> Result<Response> {
        let encoded_remote_path = urlencode(remote_path);

        let args = GetPresignedObjectUrlArgs::new(
            &self.bucket,
            &encoded_remote_path,
            http::method::Method::GET,
        )
        .map_err(|e| {
            Error::context(
                InternalServerError,
                e,
                "failed to create get presigned object url args",
            )
        })?;

        let url = self
            .client
            .get_presigned_object_url(&args)
            .await
            .map_err(|e| {
                Error::context(InternalServerError, e, "failed to get presigned object url")
            })?
            .url;

        Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, url)
            .body(Body::empty())
            .map_err(|e| {
                Error::context(
                    InternalServerError,
                    e,
                    "failed to build response for redirect to presigned url",
                )
            })
    }
}
