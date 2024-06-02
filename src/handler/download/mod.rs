/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod models;
#[cfg(test)]
mod tests;

use models::{DownloadUrlQueryParams, DownloadUrlResponseParams};

use axum::extract::{Extension, Query};
use axum::Json;
use std::sync::Arc;

use crate::client::Storage;
use crate::error::{Error, Result};

pub static DOWNLOAD_URL_PATH: &str = "/downloadUrl";

pub async fn download_url(
    Extension(storage): Extension<Arc<Storage>>,
    Query(params): Query<DownloadUrlQueryParams>,
) -> Result<Json<DownloadUrlResponseParams>> {
    println!("received download url request");

    let file_name = params.file_name.clone();

    let url = storage
        .get_download_url(&file_name)
        .await
        .map_err(|e| Error::from(e.context("download url")))?;

    println!("download url pushed");

    Ok(Json(DownloadUrlResponseParams { url }))
}
