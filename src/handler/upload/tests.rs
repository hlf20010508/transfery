/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::body::Body;
use axum::extract::Request;
use axum::http::{header, Method, StatusCode};
use axum::response::Response;
use axum::routing::post;
use axum::Router;
use tower::ServiceExt;

use super::models::{
    CompleteUploadFormParams, FetchUploadIdJsonParams, FetchUploadIdResponse, Part,
    UploadPartFormParams,
};
use super::{
    complete_upload, fetch_upload_id, upload_part, COMPLETE_UPLOAD_PATH, FETCH_UPLOAD_ID_PATH,
    UPLOAD_PART_PATH,
};

use crate::auth::tests::gen_auth;
use crate::client::database::models::message::MessageItem;
use crate::client::database::tests::{get_database, reset as reset_database};
use crate::client::storage::tests::{get_storage, init, reset as reset_storage};
use crate::client::{Database, Storage};
use crate::crypto::tests::get_crypto;
use crate::env::tests::DBType;
use crate::error::tests::ServerExt;
use crate::error::Error;
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

        let body = serde_json::to_string(&data).map_err(|e| Error::serialize_error(e))?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(FETCH_UPLOAD_ID_PATH)
            .header("Authorization", auth)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .map_err(|e| Error::req_build_error(e))?;

        let res = router
            .oneshot(req)
            .await
            .map_err(|e| Error::req_send_error(e))?;

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

        let body = serde_json::to_string(&data).map_err(|e| Error::serialize_error(e))?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(FETCH_UPLOAD_ID_PATH)
            .header("Authorization", auth.clone())
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .map_err(|e| Error::req_build_error(e))?;

        let res = router
            .clone()
            .oneshot(req)
            .await
            .map_err(|e| Error::req_send_error(e))?;

        let res_content = res.to_string().await?;
        let res_data: FetchUploadIdResponse =
            serde_json::from_str(&res_content).map_err(|e| Error::deserialize_error(e))?;

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
            .map_err(|e| Error::req_build_error(e))?;

        let res = router
            .clone()
            .oneshot(req)
            .await
            .map_err(|e| Error::req_send_error(e))?;

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

        let body = serde_json::to_string(&data).map_err(|e| Error::serialize_error(e))?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(FETCH_UPLOAD_ID_PATH)
            .header("Authorization", auth.clone())
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .map_err(|e| Error::req_build_error(e))?;

        let res = router
            .clone()
            .oneshot(req)
            .await
            .map_err(|e| Error::req_send_error(e))?;

        let res_content = res.to_string().await?;
        let res_data: FetchUploadIdResponse =
            serde_json::from_str(&res_content).map_err(|e| Error::deserialize_error(e))?;

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
            .map_err(|e| Error::req_build_error(e))?;

        let res = router
            .clone()
            .oneshot(req)
            .await
            .map_err(|e| Error::req_send_error(e))?;

        let etag = res.to_string().await?;

        let data = CompleteUploadFormParams {
            id,
            file_name: file_name.clone(),
            upload_id: upload_id.clone(),
            parts: vec![Part { number: 1, etag }],
        };

        let body = serde_json::to_string(&data).map_err(|e| Error::serialize_error(e))?;

        let req = Request::builder()
            .method(Method::POST)
            .uri(COMPLETE_UPLOAD_PATH)
            .header("Authorization", auth.clone())
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .map_err(|e| Error::req_build_error(e))?;

        let res = router
            .clone()
            .oneshot(req)
            .await
            .map_err(|e| Error::req_send_error(e))?;

        Ok(res)
    }

    let storage = get_storage();
    let database = get_database(DBType::Sqlite).await;

    let result = inner(&storage, &database).await;
    reset_storage(&storage).await;
    reset_database(database).await;
    assert_eq!(result.unwrap().status(), StatusCode::OK);

    sleep_async(1).await;
}
