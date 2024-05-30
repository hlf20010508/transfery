/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::extract::{Extension, FromRequest, Json, Multipart, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{async_trait, debug_handler};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::auth::AuthChecker;
use crate::client::{Database, Storage};
use crate::error::Error::{self, FieldParseError, FromRequestError};
use crate::error::Result;
use crate::utils::rename;

#[derive(Deserialize, Serialize, Clone)]
pub struct FetchUploadIdJsonParams {
    content: String,
    timestamp: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FetchUploadIdResponse {
    #[serde(rename = "uploadId")]
    upload_id: String,
    #[serde(rename = "fileName")]
    file_name: String,
}

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

#[derive(Deserialize, Debug)]
pub struct UploadPartFormParams {
    #[serde(rename = "filePart")]
    file_part: Vec<u8>,
    #[serde(rename = "fileName")]
    file_name: String,
    #[serde(rename = "uploadId")]
    upload_id: String,
    #[serde(rename = "partNumber")]
    part_number: u16, // at least 1
}

#[async_trait]
impl<S> FromRequest<S> for UploadPartFormParams
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request(req: Request, state: &S) -> Result<Self> {
        let mut multipart = Multipart::from_request(req, state)
            .await
            .map_err(|e| FromRequestError(format!("failed to parse multipart form: {}", e)))?;

        let mut file_name = String::new();
        let mut upload_id = String::new();
        let mut part_number = u16::default();
        let mut file_part = Vec::<u8>::new();

        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|e| FieldParseError(format!("failed to parse multipart field: {}", e)))?
        {
            let name = match field.name() {
                Some(name) => name.to_string(),
                None => continue,
            };

            let data = field
                .bytes()
                .await
                .map_err(|e| FieldParseError(format!("failed to read field bytes: {}", e)))?;

            match name.as_str() {
                "fileName" => {
                    file_name = String::from_utf8(data.to_vec()).map_err(|e| {
                        FieldParseError(format!("failed to parse field fileName: {}", e))
                    })?;
                }
                "uploadId" => {
                    upload_id = String::from_utf8(data.to_vec()).map_err(|e| {
                        FieldParseError(format!("failed to parse field uploadId: {}", e))
                    })?;
                }
                "partNumber" => {
                    let part_number_str = String::from_utf8(data.to_vec()).map_err(|e| {
                        FieldParseError(format!("failed to parse field partNumber: {}", e))
                    })?;
                    part_number = part_number_str.parse::<u16>().map_err(|e| {
                        FieldParseError(format!("failed to parse field partNumber to u16: {}", e))
                    })?;
                }
                "filePart" => {
                    file_part.extend_from_slice(&data);
                }
                _ => {}
            }
        }

        Ok(UploadPartFormParams {
            file_name,
            upload_id,
            part_number,
            file_part,
        })
    }
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

#[derive(Deserialize, Serialize, Clone)]
pub struct Part {
    number: u16,
    etag: String,
}

impl Into<minio::s3::types::Part> for Part {
    fn into(self) -> minio::s3::types::Part {
        minio::s3::types::Part {
            number: self.number,
            etag: self.etag,
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct CompleteUploadFormParams {
    id: i64,
    #[serde(rename = "fileName")]
    file_name: String,
    #[serde(rename = "uploadId")]
    upload_id: String,
    parts: Vec<Part>,
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

#[cfg(test)]
mod tests {
    use super::*;

    use axum::body::Body;
    use axum::extract::Request;
    use axum::http::{header, Method};
    use axum::response::Response;
    use axum::routing::post;
    use axum::Router;
    use tower::ServiceExt;

    use crate::auth::tests::gen_auth;
    use crate::client::database::tests::{get_database, reset as reset_database};
    use crate::client::database::MessageItem;
    use crate::client::storage::tests::{get_storage, init, reset as reset_storage};
    use crate::client::{Database, Storage};
    use crate::crypto::tests::get_crypto;
    use crate::error::Error::{DefaultError, ToStrError};
    use crate::error::Result;
    use crate::utils::tests::sleep_async;
    use crate::utils::tests::ResponseExt;
    use crate::utils::{get_current_timestamp, into_layer};

    const BOUNDARY: &str = "------------------------boundary";

    impl UploadPartFormParams {
        fn gen_payload(&self) -> Vec<u8> {
            let mut body = format!(
                "--{boundary}\r\n\
                Content-Disposition: form-data; name=\"fileName\"\r\n\r\n\
                {file_name}\r\n\
                --{boundary}\r\n\
                Content-Disposition: form-data; name=\"uploadId\"\r\n\r\n\
                {upload_id}\r\n\
                --{boundary}\r\n\
                Content-Disposition: form-data; name=\"partNumber\"\r\n\r\n\
                {part_number}\r\n\
                --{boundary}\r\n\
                Content-Disposition: form-data; name=\"filePart\"; filename=\"{file_name}\"\r\n\
                Content-Type: application/octet-stream\r\n\r\n\
                ",
                boundary = BOUNDARY,
                file_name = self.file_name,
                upload_id = self.upload_id,
                part_number = self.part_number,
            )
            .as_bytes()
            .to_vec();

            body.extend_from_slice(&self.file_part);
            body.extend_from_slice(format!("\r\n--{}--\r\n", BOUNDARY).as_bytes());

            body
        }

        fn gen_header<'a>() -> (&'a str, String) {
            (
                "Content-Type",
                format!("multipart/form-data; boundary={}", BOUNDARY),
            )
        }
    }

    #[tokio::test]
    async fn test_upload_fetch_upload_id() {
        async fn inner(storage: &Storage) -> Result<Response> {
            let content = "test_upload_fetch_upload_id.txt";
            init(&storage).await?;

            let crypto = get_crypto();
            let auth = gen_auth(&crypto);

            let router = Router::new()
                .route(FETCH_UPLOAD_ID_PATH, post(fetch_upload_id))
                .layer(into_layer(storage.clone()))
                .layer(into_layer(crypto.clone()));

            let data = FetchUploadIdJsonParams {
                content: content.to_string(),
                timestamp: get_current_timestamp(),
            };

            let body = serde_json::to_string(&data)
                .map_err(|e| ToStrError(format!("failed to build request: {}", e)))?;

            let req = Request::builder()
                .method(Method::POST)
                .uri(FETCH_UPLOAD_ID_PATH)
                .header("Authorization", auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .map_err(|e| DefaultError(format!("failed to build request: {}", e)))?;

            let res = router
                .oneshot(req)
                .await
                .map_err(|e| DefaultError(format!("failed to make request: {}", e)))?;

            Ok(res)
        }

        let storage = get_storage();

        let result = inner(&storage).await;
        reset_storage(&storage).await;
        assert_eq!(result.unwrap().status(), StatusCode::OK);

        sleep_async(1).await;
    }

    #[tokio::test]
    async fn test_upload_upload_part() {
        async fn inner(storage: &Storage) -> Result<Response> {
            let content = "test_upload_upload_part.txt";
            init(&storage).await?;

            let crypto = get_crypto();
            let auth = gen_auth(&crypto);

            let router = Router::new()
                .route(FETCH_UPLOAD_ID_PATH, post(fetch_upload_id))
                .route(UPLOAD_PART_PATH, post(upload_part))
                .layer(into_layer(storage.clone()))
                .layer(into_layer(crypto.clone()));

            let data = FetchUploadIdJsonParams {
                content: content.to_string(),
                timestamp: get_current_timestamp(),
            };

            let body = serde_json::to_string(&data)
                .map_err(|e| ToStrError(format!("failed to build request: {}", e)))?;

            let req = Request::builder()
                .method(Method::POST)
                .uri(FETCH_UPLOAD_ID_PATH)
                .header("Authorization", auth.clone())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .map_err(|e| DefaultError(format!("failed to build request: {}", e)))?;

            let res = router
                .clone()
                .oneshot(req)
                .await
                .map_err(|e| DefaultError(format!("failed to make request: {}", e)))?;

            let res_content = res.to_string().await?;
            let res_data: FetchUploadIdResponse = serde_json::from_str(&res_content)
                .map_err(|e| ToStrError(format!("failed to parse response: {}", e)))?;

            let FetchUploadIdResponse {
                upload_id,
                file_name,
            } = res_data;

            let data = UploadPartFormParams {
                file_name,
                upload_id,
                part_number: 1,
                file_part: content.as_bytes().to_vec(),
            };

            let payload = data.gen_payload();

            let (upload_header_key, upload_header_value) = UploadPartFormParams::gen_header();

            let req = Request::builder()
                .method(Method::POST)
                .uri(UPLOAD_PART_PATH)
                .header("Authorization", auth.clone())
                .header(upload_header_key, upload_header_value)
                .body(Body::from(payload))
                .map_err(|e| DefaultError(format!("failed to build request: {}", e)))?;

            let res = router
                .clone()
                .oneshot(req)
                .await
                .map_err(|e| DefaultError(format!("failed to make request: {}", e)))?;

            Ok(res)
        }

        let storage = get_storage();

        let result = inner(&storage).await;
        reset_storage(&storage).await;
        assert_eq!(result.unwrap().status(), StatusCode::OK);

        sleep_async(1).await;
    }

    #[tokio::test]
    async fn test_upload_complete_upload() {
        async fn inner(storage: &Storage, database: &Database) -> Result<Response> {
            let content = "test_upload_complete_upload.txt";

            init(&storage).await?;

            let crypto = get_crypto();
            let auth = gen_auth(&crypto);

            let router = Router::new()
                .route(FETCH_UPLOAD_ID_PATH, post(fetch_upload_id))
                .route(UPLOAD_PART_PATH, post(upload_part))
                .route(COMPLETE_UPLOAD_PATH, post(complete_upload))
                .layer(into_layer(storage.clone()))
                .layer(into_layer(database.clone()))
                .layer(into_layer(crypto.clone()));

            let data = FetchUploadIdJsonParams {
                content: content.to_string(),
                timestamp: get_current_timestamp(),
            };

            let body = serde_json::to_string(&data)
                .map_err(|e| ToStrError(format!("failed to build request: {}", e)))?;

            let req = Request::builder()
                .method(Method::POST)
                .uri(FETCH_UPLOAD_ID_PATH)
                .header("Authorization", auth.clone())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .map_err(|e| DefaultError(format!("failed to build request: {}", e)))?;

            let res = router
                .clone()
                .oneshot(req)
                .await
                .map_err(|e| DefaultError(format!("failed to make request: {}", e)))?;

            let res_content = res.to_string().await?;
            let res_data: FetchUploadIdResponse = serde_json::from_str(&res_content)
                .map_err(|e| ToStrError(format!("failed to parse response: {}", e)))?;

            let FetchUploadIdResponse {
                upload_id,
                file_name,
            } = res_data;

            let item =
                MessageItem::new_file(content, get_current_timestamp(), false, &file_name, false);

            database.create_table_message_if_not_exists().await?;
            let id = database.insert_message_item(item).await?;

            let data = UploadPartFormParams {
                file_name: file_name.clone(),
                upload_id: upload_id.clone(),
                part_number: 1,
                file_part: content.as_bytes().to_vec(),
            };

            let payload = data.gen_payload();

            let (upload_header_key, upload_header_value) = UploadPartFormParams::gen_header();

            let req = Request::builder()
                .method(Method::POST)
                .uri(UPLOAD_PART_PATH)
                .header("Authorization", auth.clone())
                .header(upload_header_key, upload_header_value)
                .body(Body::from(payload))
                .map_err(|e| DefaultError(format!("failed to build request: {}", e)))?;

            let res = router
                .clone()
                .oneshot(req)
                .await
                .map_err(|e| DefaultError(format!("failed to make request: {}", e)))?;

            let etag = res.to_string().await?;

            let data = CompleteUploadFormParams {
                id: id as i64,
                file_name: file_name.clone(),
                upload_id: upload_id.clone(),
                parts: vec![Part { number: 1, etag }],
            };

            let body = serde_json::to_string(&data)
                .map_err(|e| ToStrError(format!("failed to build request: {}", e)))?;

            let req = Request::builder()
                .method(Method::POST)
                .uri(COMPLETE_UPLOAD_PATH)
                .header("Authorization", auth.clone())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .map_err(|e| DefaultError(format!("failed to build request: {}", e)))?;

            let res = router
                .clone()
                .oneshot(req)
                .await
                .map_err(|e| DefaultError(format!("failed to make request: {}", e)))?;

            Ok(res)
        }

        let storage = get_storage();
        let database = get_database().await;

        let result = inner(&storage, &database).await;
        reset_storage(&storage).await;
        reset_database(database).await;
        assert_eq!(result.unwrap().status(), StatusCode::OK);

        sleep_async(1).await;
    }
}
