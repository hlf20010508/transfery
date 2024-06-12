/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod models;
#[cfg(test)]
mod tests;

use models::DownloadUrlQueryParams;

use axum::debug_handler;
use axum::extract::{Extension, Query};
use std::sync::Arc;

use crate::client::Storage;
use crate::error::{Error, Result};

pub static DOWNLOAD_URL_PATH: &str = "/downloadUrl";

#[debug_handler]
pub async fn download_url(
    Extension(storage): Extension<Arc<Storage>>,
    Query(params): Query<DownloadUrlQueryParams>,
) -> Result<String> {
    tracing::info!("received download url request");

    let file_name = params.file_name.clone();

    let url = storage
        .get_download_url(&file_name)
        .await
        .map_err(|e| Error::from(e.context("download url")))?;

    tracing::info!("download url pushed");
    tracing::debug!("download url: {}", url);

    Ok(url)
}
