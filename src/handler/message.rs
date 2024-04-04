use actix_web::{get, web, HttpResponse, Result};
use serde::Deserialize;

use crate::auth::AuthState;
use crate::client::Database;
use crate::env::ITEM_PER_PAGE;

#[derive(Deserialize)]
struct PageQueryParams {
    size: u32,
}

#[derive(Deserialize)]
struct SyncQueryParams {
    #[serde(rename = "latestId")]
    latest_id: u32,
}

#[get("/page")]
async fn page(
    database: web::Data<Database>,
    params: web::Query<PageQueryParams>,
    auth_state: AuthState,
) -> Result<HttpResponse> {
    println!("received new page request");

    let start = params.size;

    let result = database
        .query_message_items(start, ITEM_PER_PAGE.clone(), auth_state.is_authorized())
        .await?;

    println!("new page pushed");

    // println!("{:#?}", result);

    Ok(HttpResponse::Ok().json(result))
}

#[get("/sync")]
async fn sync(
    database: web::Data<Database>,
    params: web::Query<SyncQueryParams>,
    auth_state: AuthState,
) -> Result<HttpResponse> {
    println!("received sync request");

    let latest_id = params.latest_id;

    let result = database
        .query_message_items_after_id(latest_id, auth_state.is_authorized())
        .await?;

    println!("synced: {:#?}", result);

    Ok(HttpResponse::Ok().json(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::http::StatusCode;
    use actix_web::{test as atest, web, App};

    use crate::auth::tests::gen_auth;
    use crate::client::database::tests::{get_database, reset};
    use crate::client::database::MessageItem;
    use crate::client::Database;
    use crate::crypto::tests::get_crypto;
    use crate::utils::get_current_timestamp;

    async fn fake_message_item(database: &Database) {
        let item = MessageItem::new_text("fake item for message", get_current_timestamp(), false);

        database.create_table_message_if_not_exists().await.unwrap();
        database.insert_message_item(item).await.unwrap();
    }

    #[atest]
    async fn test_message_page() {
        let database = get_database().await;
        let crypto = get_crypto();

        fake_message_item(&database).await;

        let mut app = atest::init_service(
            App::new()
                .service(page)
                .app_data(web::Data::new(database.clone()))
                .app_data(web::Data::new(crypto.clone())),
        )
        .await;

        let authorization = gen_auth(&crypto);

        let req = atest::TestRequest::get()
            .uri("/page?size=0")
            .insert_header(("Authorization", authorization))
            .to_request();

        let resp = atest::call_service(&mut app, req).await;
        reset(&database).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[atest]
    async fn test_message_sync() {
        let database = get_database().await;
        let crypto = get_crypto();

        fake_message_item(&database).await;

        let mut app = atest::init_service(
            App::new()
                .service(sync)
                .app_data(web::Data::new(database.clone()))
                .app_data(web::Data::new(crypto.clone())),
        )
        .await;

        let authorization = gen_auth(&crypto);

        let req = atest::TestRequest::get()
            .uri("/sync?latestId=0")
            .insert_header(("Authorization", authorization))
            .to_request();

        let resp = atest::call_service(&mut app, req).await;
        reset(&database).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
