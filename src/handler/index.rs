/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::response::Html;
use tokio::fs::read_to_string;

use crate::error::Error;
use crate::error::ErrorType::InternalServerError;
use crate::error::Result;

pub static INDEX_PATH: &str = "/";

pub async fn index() -> Result<Html<String>> {
    let html = read_to_string("./index.html")
        .await
        .map_err(|e| Error::context(InternalServerError, e, "failed to read index.html"))?;

    Ok(Html(html))
}
