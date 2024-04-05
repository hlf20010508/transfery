/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use actix_multipart::Multipart;
use actix_web::{post, web, Error, FromRequest, HttpResponse, Result};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use crate::auth::AuthState;
use crate::client::{Database, Storage};
use crate::utils::rename;

#[derive(Deserialize, Serialize, Clone)]
struct FetchUploadIdJsonParams {
    content: String,
    timestamp: i64,
}

#[derive(Deserialize, Serialize, Debug)]
struct FetchUploadIdResponse {
    #[serde(rename = "uploadId")]
    upload_id: String,
    #[serde(rename = "fileName")]
    file_name: String,
}

#[post("/fetchUploadId")]
pub async fn fetch_upload_id(
    storage: web::Data<Storage>,
    params: web::Json<FetchUploadIdJsonParams>,
    auth_state: AuthState,
) -> Result<HttpResponse> {
    if !auth_state.is_authorized() {
        return Ok(HttpResponse::Unauthorized().finish());
    }

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

    Ok(HttpResponse::Ok().json(result))
}

#[derive(Deserialize, Debug)]
struct UploadPartFormParams {
    #[serde(rename = "filePart")]
    file_part: Vec<u8>,
    #[serde(rename = "fileName")]
    file_name: String,
    #[serde(rename = "uploadId")]
    upload_id: String,
    #[serde(rename = "partNumber")]
    part_number: u16, // at least 1
}

impl FromRequest for UploadPartFormParams {
    type Error = Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self, Error>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let mut payload = Multipart::new(req.headers(), payload.take());

        let fut = async move {
            let mut file_name = String::new();
            let mut upload_id = String::new();
            let mut part_number = u16::default();
            let mut file_part = Vec::<u8>::new();

            while let Some(Ok(mut field)) = payload.next().await {
                let disposition = field.content_disposition();
                let field_name = disposition.get_name().unwrap_or_default();

                match field_name {
                    "fileName" => {
                        let mut data = Vec::<u8>::new();
                        while let Some(chunk) = field.next().await {
                            data.extend_from_slice(&chunk?);
                        }
                        file_name = String::from_utf8(data.to_vec()).unwrap_or_default();
                    }
                    "uploadId" => {
                        let mut data = Vec::<u8>::new();
                        while let Some(chunk) = field.next().await {
                            data.extend_from_slice(&chunk?);
                        }
                        upload_id = String::from_utf8(data.to_vec()).unwrap_or_default();
                    }
                    "partNumber" => {
                        let mut data = Vec::<u8>::new();
                        while let Some(chunk) = field.next().await {
                            data.extend_from_slice(&chunk?);
                        }
                        let part_number_str = String::from_utf8(data.to_vec()).unwrap_or_default();
                        part_number = part_number_str.parse::<u16>().unwrap_or(0);
                    }
                    "filePart" => {
                        let mut data = Vec::<u8>::new();
                        while let Some(chunk) = field.next().await {
                            data.extend_from_slice(&chunk?);
                        }
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
        };

        Box::pin(fut)
    }
}

#[post("/uploadPart")]
pub async fn upload_part(
    storage: web::Data<Storage>,
    params: UploadPartFormParams,
    auth_state: AuthState,
) -> Result<HttpResponse> {
    if !auth_state.is_authorized() {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    let etag = storage
        .multipart_upload(
            &params.file_name,
            &params.upload_id,
            &params.file_part,
            params.part_number,
        )
        .await?
        .etag;

    Ok(HttpResponse::Ok().body(etag))
}

#[derive(Deserialize, Serialize, Clone)]
struct Part {
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
struct CompleteUploadFormParams {
    id: i64,
    #[serde(rename = "fileName")]
    file_name: String,
    #[serde(rename = "uploadId")]
    upload_id: String,
    parts: Vec<Part>,
}

#[post("/completeUpload")]
pub async fn complete_upload(
    storage: web::Data<Storage>,
    database: web::Data<Database>,
    params: web::Json<CompleteUploadFormParams>,
    auth_state: AuthState,
) -> Result<HttpResponse> {
    if !auth_state.is_authorized() {
        return Ok(HttpResponse::Unauthorized().finish());
    }

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

    Ok(HttpResponse::Ok().finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::dev::ServiceResponse;
    use actix_web::http::StatusCode;
    use actix_web::{test as atest, App};

    use crate::auth::tests::gen_auth;
    use crate::client::database::tests::{get_database, reset as reset_database};
    use crate::client::database::MessageItem;
    use crate::client::storage::tests::{get_storage, init, reset as reset_storage};
    use crate::client::{Database, Storage};
    use crate::crypto::tests::get_crypto;
    use crate::error::Error::ToStrError;
    use crate::error::Result;
    use crate::utils::get_current_timestamp;

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

    #[atest]
    async fn test_upload_fetch_upload_id() {
        async fn inner(storage: &Storage) -> Result<ServiceResponse> {
            let content = "test_upload_fetch_upload_id.txt";
            init(&storage).await?;

            let crypto = get_crypto();
            let auth = gen_auth(&crypto);

            let mut app = atest::init_service(
                App::new()
                    .service(fetch_upload_id)
                    .app_data(web::Data::new(storage.clone()))
                    .app_data(web::Data::new(crypto.clone())),
            )
            .await;

            let req = atest::TestRequest::post()
                .uri("/fetchUploadId")
                .set_json(FetchUploadIdJsonParams {
                    content: content.to_string(),
                    timestamp: get_current_timestamp(),
                })
                .insert_header(("Authorization", auth))
                .to_request();

            let resp = atest::call_service(&mut app, req).await;

            Ok(resp)
        }

        let storage = get_storage();

        let result = inner(&storage).await;
        reset_storage(&storage).await;
        assert_eq!(result.unwrap().status(), StatusCode::OK);
    }

    #[atest]
    async fn test_upload_upload_part() {
        async fn inner(storage: &Storage) -> Result<ServiceResponse> {
            let content = "test_upload_upload_part.txt";
            init(&storage).await?;

            let crypto = get_crypto();
            let auth = gen_auth(&crypto);

            let mut app = atest::init_service(
                App::new()
                    .service(fetch_upload_id)
                    .service(upload_part)
                    .app_data(web::Data::new(storage.clone()))
                    .app_data(web::Data::new(crypto.clone())),
            )
            .await;

            let req = atest::TestRequest::post()
                .uri("/fetchUploadId")
                .set_json(FetchUploadIdJsonParams {
                    content: content.to_string(),
                    timestamp: get_current_timestamp(),
                })
                .insert_header(("Authorization", auth.clone()))
                .to_request();

            let resp = atest::call_service(&mut app, req).await;
            let resp: FetchUploadIdResponse = atest::read_body_json(resp).await;

            let FetchUploadIdResponse {
                upload_id,
                file_name,
            } = resp;

            let data = UploadPartFormParams {
                file_name,
                upload_id,
                part_number: 1,
                file_part: content.as_bytes().to_vec(),
            };

            let payload = data.gen_payload();

            let req = atest::TestRequest::post()
                .uri("/uploadPart")
                .set_payload(payload)
                .insert_header(UploadPartFormParams::gen_header())
                .insert_header(("Authorization", auth.clone()))
                .to_request();

            let resp = atest::call_service(&mut app, req).await;

            Ok(resp)
        }

        let storage = get_storage();

        let result = inner(&storage).await;
        reset_storage(&storage).await;
        assert_eq!(result.unwrap().status(), StatusCode::OK);
    }

    #[atest]
    async fn test_upload_complete_upload() {
        async fn inner(storage: &Storage, database: &Database) -> Result<ServiceResponse> {
            let content = "test_upload_complete_upload.txt";

            init(&storage).await?;

            let crypto = get_crypto();
            let auth = gen_auth(&crypto);

            let mut app = atest::init_service(
                App::new()
                    .service(fetch_upload_id)
                    .service(upload_part)
                    .service(complete_upload)
                    .app_data(web::Data::new(storage.clone()))
                    .app_data(web::Data::new(database.clone()))
                    .app_data(web::Data::new(crypto.clone())),
            )
            .await;

            let req = atest::TestRequest::post()
                .uri("/fetchUploadId")
                .set_json(FetchUploadIdJsonParams {
                    content: content.to_string(),
                    timestamp: get_current_timestamp(),
                })
                .insert_header(("Authorization", auth.clone()))
                .to_request();

            let resp = atest::call_service(&mut app, req).await;
            let resp: FetchUploadIdResponse = atest::read_body_json(resp).await;

            let FetchUploadIdResponse {
                upload_id,
                file_name,
            } = resp;

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

            let req = atest::TestRequest::post()
                .uri("/uploadPart")
                .set_payload(payload)
                .insert_header(UploadPartFormParams::gen_header())
                .insert_header(("Authorization", auth.clone()))
                .to_request();

            let resp = atest::call_service(&mut app, req).await;
            let etag_byte = atest::read_body(resp)
                .await
                .into_iter()
                .collect::<Vec<u8>>();
            let etag = String::from_utf8(etag_byte).map_err(|e| {
                ToStrError(format!("Failed to convert etag Vec<u8> to String: {}", e))
            })?;

            let req = atest::TestRequest::post()
                .uri("/completeUpload")
                .set_json(CompleteUploadFormParams {
                    id: id as i64,
                    file_name: file_name.clone(),
                    upload_id: upload_id.clone(),
                    parts: vec![Part { number: 1, etag }],
                })
                .insert_header(("Authorization", auth.clone()))
                .to_request();

            let resp = atest::call_service(&mut app, req).await;

            Ok(resp)
        }

        let storage = get_storage();
        let database = get_database().await;

        let result = inner(&storage, &database).await;
        reset_storage(&storage).await;
        reset_database(&database).await;
        assert_eq!(result.unwrap().status(), StatusCode::OK);
    }
}
