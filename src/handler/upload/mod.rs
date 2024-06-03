/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod models;
#[cfg(test)]
mod tests;

use models::{
    CompleteUploadFormParams, FetchUploadIdJsonParams, FetchUploadIdResponse, UploadPartFormParams,
};

use axum::debug_handler;
use axum::extract::{Extension, Json};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::sync::Arc;

use crate::auth::AuthChecker;
use crate::client::{Database, Storage};
use crate::error::Result;
use crate::utils::rename;

pub static FETCH_UPLOAD_ID_PATH: &str = "/fetchUploadId";

#[debug_handler]
pub async fn fetch_upload_id(
    _: AuthChecker,
    Extension(storage): Extension<Arc<Storage>>,
    Json(params): Json<FetchUploadIdJsonParams>,
) -> Result<Response> {
    println!("received fetch upload id request");

    let content = params.clone().content;
    let timestamp = params.clone().timestamp;

    let file_name = rename(&content, timestamp);
    let upload_id = storage.create_multipart_upload_id(&file_name).await?;

    let result = FetchUploadIdResponse {
        file_name,
        upload_id,
    };

    println!("upload id pushed");

    // println!("{:#?}", result);

    Ok(axum::Json(result).into_response())
}

pub static UPLOAD_PART_PATH: &str = "/uploadPart";

#[debug_handler]
pub async fn upload_part(
    _: AuthChecker,
    Extension(storage): Extension<Arc<Storage>>,
    params: UploadPartFormParams,
) -> Result<Response> {
    let etag = storage
        .multipart_upload(
            &params.file_name,
            &params.upload_id,
            &params.file_part,
            params.part_number,
        )
        .await?
        .etag;

    Ok(etag.into_response())
}

pub static COMPLETE_UPLOAD_PATH: &str = "/completeUpload";

#[debug_handler]
pub async fn complete_upload(
    _: AuthChecker,
    Extension(storage): Extension<Arc<Storage>>,
    Extension(database): Extension<Arc<Database>>,
    Json(params): Json<CompleteUploadFormParams>,
) -> Result<Response> {
    println!("received complete upload request");

    let id = params.clone().id;
    let file_name = params.clone().file_name;
    let upload_id = params.clone().upload_id;
    let parts = params
        .clone()
        .parts
        .into_iter()
        .map(|part| part.into())
        .collect();

    storage
        .complete_multipart_upload(&file_name, &upload_id, &parts)
        .await?;

    database.update_complete(id).await?;

    Ok(StatusCode::OK.into_response())
}
