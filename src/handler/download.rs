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
    use dotenv::dotenv;
    use minio::s3::args::PutObjectArgs;
    use std::env;
    use std::io::Cursor;

    use super::*;

    use crate::error::Error::StorageObjectError;
    use crate::error::Result;

    fn get_storage() -> Storage {
        dotenv().ok();

        let endpoint = env::var("MINIO_ENDPOINT").unwrap();
        let username = env::var("MINIO_USERNAME").unwrap();
        let password = env::var("MINIO_PASSWORD").unwrap();
        let bucket = env::var("MINIO_BUCKET").unwrap();

        let storage = Storage::new(&endpoint, &username, &password, &bucket).unwrap();

        storage
    }

    async fn init(storage: &Storage) -> Result<()> {
        storage.init().await
    }

    async fn reset(storage: &Storage) {
        storage.remove_bucket().await.unwrap();
    }

    fn fake_data() -> Vec<u8> {
        let data = Vec::from("hello world!");

        let repeat_times: usize = 1024 * 1024;

        let data = data
            .iter()
            .cycle()
            .take(data.len() * repeat_times)
            .cloned()
            .collect();

        data
    }

    async fn upload_data(storage: &Storage, remote_path: &str) -> Result<()> {
        let mut data = Cursor::new(fake_data());
        let size = data.clone().into_inner().len();

        let mut args =
            PutObjectArgs::new(&storage.bucket, remote_path, &mut data, Some(size), None).map_err(
                |e| StorageObjectError(format!("Storage create put object args failed: {}", e)),
            )?;

        storage
            .client
            .put_object(&mut args)
            .await
            .map_err(|e| StorageObjectError(format!("Storage put object failed: {}", e)))?;

        Ok(())
    }

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
