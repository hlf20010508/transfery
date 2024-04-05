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
use crate::client::Storage;
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

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::dev::ServiceResponse;
    use actix_web::http::StatusCode;
    use actix_web::{test as atest, App};

    use crate::auth::tests::gen_auth;
    use crate::client::storage::tests::{get_storage, init, reset};
    use crate::client::Storage;
    use crate::crypto::tests::get_crypto;
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
        reset(&storage).await;
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
        reset(&storage).await;
        assert_eq!(result.unwrap().status(), StatusCode::OK);
    }
}
