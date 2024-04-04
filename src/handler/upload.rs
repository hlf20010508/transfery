/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use actix_web::{post, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};

use crate::auth::AuthState;
use crate::client::Storage;
use crate::utils::rename;

#[derive(Deserialize, Serialize, Clone)]
struct FetchUploadIdJsonParams {
    content: String,
    timestamp: i64,
}

#[derive(Serialize, Debug)]
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
}
