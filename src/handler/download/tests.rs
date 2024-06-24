/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use tower::ServiceExt;

use super::{download, download_url, DOWNLOAD_PATH, DOWNLOAD_URL_PATH, _DOWNLOAD_PATH_ROOT};
use crate::client::storage::tests::{get_storage, init, reset, upload_data};
use crate::client::storage::Storage;
use crate::env::tests::STType;
use crate::error::tests::ServerExt;
use crate::error::{Error, Result};
use crate::utils::into_layer;
use crate::utils::tests::sleep_async;

#[tokio::test]
async fn test_download_download_url() {
    async fn inner(storage: &Storage) -> Result<Response> {
        let remote_path = "test.txt";

        init(&storage).await?;
        upload_data(&storage, remote_path).await?;

        let router = Router::new()
            .route(DOWNLOAD_URL_PATH, get(download_url))
            .layer(into_layer(storage.clone()));

        let req = Request::builder()
            .method(Method::GET)
            .uri(&format!("{}?fileName={}", DOWNLOAD_URL_PATH, remote_path))
            .body(Body::empty())
            .map_err(|e| Error::req_build_error(e))?;

        let res = router
            .oneshot(req)
            .await
            .map_err(|e| Error::req_send_error(e))?;

        Ok(res)
    }

    let storage = get_storage(STType::LocalStorage).await;

    let result = inner(&storage).await;

    reset(&storage).await;

    assert_eq!(result.unwrap().status(), StatusCode::OK);

    sleep_async(1).await;
}

#[tokio::test]
async fn test_download_download() {
    async fn inner(storage: &Storage) -> Result<Response> {
        let file_name = "test.txt";

        init(&storage).await?;
        upload_data(&storage, file_name).await?;

        let router = Router::new()
            .route(DOWNLOAD_PATH, get(download))
            .layer(into_layer(storage.clone()));

        let req = Request::builder()
            .method(Method::GET)
            .uri(&format!("{}/{}", _DOWNLOAD_PATH_ROOT, file_name))
            .body(Body::empty())
            .map_err(|e| Error::req_build_error(e))?;

        let res = router
            .oneshot(req)
            .await
            .map_err(|e| Error::req_send_error(e))?;

        Ok(res)
    }

    let storage = get_storage(STType::LocalStorage).await;

    let result = inner(&storage).await;

    reset(&storage).await;

    assert_eq!(result.unwrap().status(), StatusCode::OK);

    sleep_async(1).await;
}
