/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::extract::{Extension, Query};
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;

use crate::auth::AuthState;
use crate::client::database::MessageItem;
use crate::client::Database;
use crate::env::ITEM_PER_PAGE;
use crate::error::Result;

#[derive(Deserialize)]
pub struct PageQueryParams {
    size: u32,
}

#[derive(Deserialize)]
pub struct SyncQueryParams {
    #[serde(rename = "latestId")]
    latest_id: u32,
}

pub static PAGE_PATH: &str = "/page";

pub async fn page(
    Extension(database): Extension<Arc<Database>>,
    Query(params): Query<PageQueryParams>,
    AuthState(is_authorized): AuthState,
) -> Result<Json<Vec<MessageItem>>> {
    println!("received new page request");

    let start = params.size;

    let result = database
        .query_message_items(start, ITEM_PER_PAGE.clone(), is_authorized)
        .await?;

    println!("new page pushed");

    // println!("{:#?}", result);

    Ok(Json(result))
}

pub static SYNC_PATH: &str = "/sync";

pub async fn sync(
    Extension(database): Extension<Arc<Database>>,
    Query(params): Query<SyncQueryParams>,
    AuthState(is_authorized): AuthState,
) -> Result<Json<Vec<MessageItem>>> {
    println!("received sync request");

    let latest_id = params.latest_id;

    let result = database
        .query_message_items_after_id(latest_id, is_authorized)
        .await?;

    println!("synced: {:#?}", result);

    Ok(Json(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::response::Response;
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    use crate::auth::tests::gen_auth;
    use crate::client::database::tests::{get_database, reset};
    use crate::client::database::MessageItem;
    use crate::client::Database;
    use crate::crypto::tests::get_crypto;
    use crate::error::Error::DefaultError;
    use crate::utils::{get_current_timestamp, into_layer};

    async fn fake_message_item(database: &Database) {
        let item = MessageItem::new_text("fake item for message", get_current_timestamp(), false);

        database.create_table_message_if_not_exists().await.unwrap();
        database.insert_message_item(item).await.unwrap();
    }

    #[tokio::test]
    async fn test_message_page() {
        async fn inner(database: &Database) -> Result<Response> {
            let crypto = get_crypto();

            fake_message_item(&database).await;

            let router = Router::new()
                .route(PAGE_PATH, get(page))
                .layer(into_layer(database.clone()))
                .layer(into_layer(crypto.clone()));

            let authorization = gen_auth(&crypto);

            let req = Request::builder()
                .method("GET")
                .uri(format!("{}?size=0", PAGE_PATH))
                .header("Authorization", authorization)
                .body(Body::empty())
                .map_err(|e| DefaultError(format!("failed to build request: {}", e)))?;

            let res = router
                .oneshot(req)
                .await
                .map_err(|e| DefaultError(format!("failed to make request: {}", e)))?;

            Ok(res)
        }

        let database = get_database().await;
        let result = inner(&database).await;
        reset(&database).await;
        assert_eq!(result.unwrap().status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_message_sync() {
        async fn inner(database: &Database) -> Result<Response> {
            let crypto = get_crypto();

            fake_message_item(&database).await;

            let router = Router::new()
                .route(SYNC_PATH, get(sync))
                .layer(into_layer(database.clone()))
                .layer(into_layer(crypto.clone()));

            let authorization = gen_auth(&crypto);

            let req = Request::builder()
                .method("GET")
                .uri(format!("{}?latestId=0", SYNC_PATH))
                .header("Authorization", authorization)
                .body(Body::empty())
                .map_err(|e| DefaultError(format!("failed to build request: {}", e)))?;

            let res = router
                .oneshot(req)
                .await
                .map_err(|e| DefaultError(format!("failed to make request: {}", e)))?;

            Ok(res)
        }

        let database = get_database().await;
        let result = inner(&database).await;
        reset(&database).await;
        assert_eq!(result.unwrap().status(), StatusCode::OK);
    }
}
