/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use actix_web::Error;
use actix_web::{get, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};

use crate::client::Storage;

#[derive(Deserialize)]
struct DownloadUrlQueryParams {
    #[serde[rename = "fileName"]]
    file_name: String,
}

#[derive(Serialize)]
struct DownloadUrlResponseParams {
    url: String,
}

#[get("downloadUrl")]
pub async fn download_url(
    storage: web::Data<Storage>,
    params: web::Query<DownloadUrlQueryParams>,
) -> Result<HttpResponse> {
    println!("received download url request");

    let file_name = params.file_name.clone();

    let url = storage
        .get_download_url(&file_name)
        .await
        .map_err(|e| Error::from(e.context("download url")))?;

    println!("download url pushed");

    Ok(HttpResponse::Ok().json(DownloadUrlResponseParams { url }))
}
