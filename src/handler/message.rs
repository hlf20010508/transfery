use actix_web::{get, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};

use crate::auth::AuthState;
use crate::client::database::MessageItem;
use crate::client::Database;
use crate::env::ITEM_PER_PAGE;

#[derive(Deserialize)]
struct PageQueryParams {
    size: u32,
}

#[derive(Serialize)]
struct PageResponseParams {
    messages: Vec<MessageItem>,
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

    Ok(HttpResponse::Ok().json(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::http::StatusCode;
    use actix_web::{test as atest, web, App};
    use dotenv::dotenv;
    use std::env;

    use crate::auth::Authorization;
    use crate::client::Database;
    use crate::crypto::Crypto;
    use crate::utils::get_current_timestamp;

    fn gen_auth(crypto: &Crypto) -> String {
        let fingerprint = "fingerprint for test";
        let certificate = crypto.encrypt(fingerprint).unwrap();

        let auth = Authorization {
            fingerprint: fingerprint.to_string(),
            certificate: Some(certificate),
        };

        serde_json::to_string(&auth).unwrap()
    }

    fn get_crypto() -> Crypto {
        let secret_key = Crypto::gen_secret_key().unwrap();
        Crypto::new(&secret_key).unwrap()
    }

    async fn get_database() -> Database {
        dotenv().ok();

        let endpoint = env::var("MYSQL_ENDPOINT").unwrap();
        let username = env::var("MYSQL_USERNAME").unwrap();
        let password = env::var("MYSQL_PASSWORD").unwrap();
        let name = env::var("MYSQL_DATABASE").unwrap();

        let database = Database::new(&endpoint, &username, &password, &name)
            .await
            .unwrap();

        database
    }

    async fn fake_message_item(database: &Database) {
        let item = MessageItem::new_text("fake item for message", get_current_timestamp(), false);

        database.create_table_message_if_not_exists().await.unwrap();
        database.insert_message_item(item).await.unwrap();
    }

    async fn reset(database: &Database) {
        database.drop_database_if_exists().await.unwrap();
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
}
