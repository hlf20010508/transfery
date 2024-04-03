/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use actix_web::{get, web, Error, HttpResponse, Result};
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

#[cfg(test)]
mod tests {
    use actix_web::dev::ServiceResponse;
    use actix_web::http::StatusCode;
    use actix_web::{test as atest, App};

    use super::*;

    use crate::client::storage::tests::{get_storage, init, reset, upload_data};
    use crate::error::Result;

    #[atest]
    async fn test_download_download_url() {
        async fn inner(storage: &Storage) -> Result<ServiceResponse> {
            let remote_path = "test_message_page.txt";
            init(&storage).await?;
            upload_data(&storage, remote_path).await?;

            let mut app = atest::init_service(
                App::new()
                    .service(download_url)
                    .app_data(web::Data::new(storage.clone())),
            )
            .await;

            let req = atest::TestRequest::get()
                .uri(&format!("/downloadUrl?fileName={}", remote_path))
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
