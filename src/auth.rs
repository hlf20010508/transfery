/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use actix_web::FromRequest;
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};

use crate::crypto::Crypto;
use crate::error::Error::{CryptoError, ToJsonError, ToStrError};

#[derive(Deserialize, Serialize)]
pub struct Authorization {
    pub fingerprint: String,
    pub certificate: Option<String>,
}

pub struct AuthState(bool);

impl AuthState {
    pub fn is_authorized(&self) -> bool {
        self.0
    }
}

impl FromRequest for AuthState {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        if let Some(auth) = req.headers().get("Authorization") {
            let auth_str = match auth.to_str() {
                Ok(str) => str,
                Err(e) => {
                    let e = ToStrError(format!("Failed to convert auth header to &str: {}", e));
                    return ready(Err(Self::Error::from(e)));
                }
            };

            let auth_json = match serde_json::from_str::<Authorization>(auth_str) {
                Ok(json) => json,
                Err(e) => {
                    let e =
                        ToJsonError(format!("Failed to convert auth header &str to json: {}", e));
                    return ready(Err(Self::Error::from(e)));
                }
            };

            let Authorization {
                fingerprint,
                certificate,
            } = auth_json;

            if let Some(certificate) = certificate {
                let crypto = match req.app_data::<actix_web::web::Data<Crypto>>() {
                    Some(crypto) => crypto,
                    None => {
                        let e = CryptoError(format!("Crypto data not found in from_request"));
                        return ready(Err(Self::Error::from(e)));
                    }
                };

                if let Ok(fingerprint_decrypted) = crypto.decrypt(&certificate) {
                    if fingerprint == fingerprint_decrypted {
                        return ready(Ok(AuthState(true)));
                    }
                };
            }
        };

        return ready(Ok(AuthState(false)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::http::StatusCode;
    use actix_web::{get, test as atest, web, App, HttpResponse, Responder};

    #[get("/")]
    async fn index(auth_state: AuthState) -> impl Responder {
        if auth_state.is_authorized() {
            HttpResponse::Ok()
        } else {
            HttpResponse::Unauthorized()
        }
    }

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

    #[atest]
    async fn test_auth_from_request() {
        let crypto = get_crypto();

        let mut app = atest::init_service(
            App::new()
                .service(index)
                .app_data(web::Data::new(crypto.clone())),
        )
        .await;

        let authorization = gen_auth(&crypto);

        let req = atest::TestRequest::get()
            .uri("/")
            .insert_header(("Authorization", authorization))
            .to_request();

        let resp = atest::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let req = atest::TestRequest::get().uri("/").to_request();

        let resp = atest::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
