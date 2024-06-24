/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::response::Response;
use tokio::fs;
use tokio_util::io::ReaderStream;

use super::utils::LocalStorageUtils;
use super::LocalStorage;
use crate::error::ErrorType::InternalServerError;
use crate::error::{Error, Result};

impl LocalStorage {
    pub async fn get_download_response(&self, file_name: &str) -> Result<Response> {
        let file_path = self.get_path(file_name);

        if !file_path.exists() {
            return Err(Error::new(
                InternalServerError,
                format!("file {} not found", file_name),
            ));
        }

        let file = fs::File::open(&file_path).await.map_err(|e| {
            Error::context(
                InternalServerError,
                e,
                format!("failed to open file {}", file_name),
            )
        })?;

        let stream = ReaderStream::new(file);

        let body = Body::from_stream(stream);

        let response = Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .header(
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", file_name),
            )
            .body(body)
            .map_err(|e| {
                Error::context(
                    InternalServerError,
                    e,
                    "failed to build response for download file",
                )
            })?;

        Ok(response)
    }
}
