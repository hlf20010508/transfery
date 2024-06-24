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
use axum::extract::{Extension, Query};
use axum::response::Response;
use models::DownloadUrlQueryParams;
use std::sync::Arc;

use crate::client::Storage;
use crate::error::Result;

pub static DOWNLOAD_PATH: &str = "/download";

#[debug_handler]
pub async fn download(
    Extension(storage): Extension<Arc<Storage>>,
    Query(params): Query<DownloadUrlQueryParams>,
) -> Result<Response> {
    tracing::info!("received download request");

    let file_name = params.file_name.clone();

    let response = storage.get_download_response(&file_name).await?;

    tracing::info!("download response pushed");
    tracing::debug!("download response: {:#?}", response);

    Ok(response)
}
