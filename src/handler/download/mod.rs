/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod models;
#[cfg(test)]
mod tests;

use axum::debug_handler;
use axum::extract::{Extension, Path, Query};
use axum::response::Response;
use models::DownloadUrlQueryParams;
use std::sync::Arc;

use crate::client::storage::models::StorageClient;
use crate::client::Storage;
use crate::error::ErrorType::InternalServerError;
use crate::error::{Error, Result};

pub static DOWNLOAD_URL_PATH: &str = "/downloadUrl";

#[debug_handler]
pub async fn download_url(
    Extension(storage): Extension<Arc<Storage>>,
    Query(params): Query<DownloadUrlQueryParams>,
) -> Result<String> {
    tracing::info!("received download url request");

    let file_name = params.file_name.clone();

    let url = storage.get_download_url(&file_name).await?;

    tracing::info!("download url pushed");
    tracing::debug!("download url: {}", url);

    Ok(url)
}

pub static _DOWNLOAD_PATH_ROOT: &str = "/download";
pub static DOWNLOAD_PATH: &str = "/download/:file_name";

#[debug_handler]
pub async fn download(
    Extension(storage): Extension<Arc<Storage>>,
    Path(file_name): Path<String>,
) -> Result<Response> {
    match &storage.client {
        StorageClient::Local(storage) => storage.get_download_response(&file_name).await,
        _ => Err(Error::new(
            InternalServerError,
            "download handler only support local storage",
        )),
    }
}
