/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::crypto::Crypto;
use crate::error::Error::{self, UnauthorizedError};
use crate::error::Result;

#[derive(Deserialize, Serialize)]
struct Authorization {
    fingerprint: String,
    certificate: Option<String>,
}

pub struct AuthState(pub bool);

#[async_trait]
impl<S> FromRequestParts<S> for AuthState
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(req: &mut Parts, _state: &S) -> Result<Self> {
        if let Some(auth) = req.headers.get("Authorization") {
            let auth_str = match auth.to_str() {
                Ok(str) => str,
                Err(e) => {
                    return Err(UnauthorizedError(format!(
                        "Failed to convert auth header to &str: {}",
                        e
                    )));
                }
            };

            let auth_json = match serde_json::from_str::<Authorization>(auth_str) {
                Ok(json) => json,
                Err(e) => {
                    return Err(UnauthorizedError(format!(
                        "Failed to convert auth header &str to json: {}",
                        e
                    )));
                }
            };

            let Authorization {
                fingerprint,
                certificate,
            } = auth_json;

            if let Some(certificate) = certificate {
                let crypto = match req.extensions.get::<Arc<Crypto>>() {
                    Some(ext) => ext,
                    None => {
                        return Err(UnauthorizedError(format!(
                            "Crypto data not found in from_request"
                        )));
                    }
                };

                if let Ok(fingerprint_decrypted) = crypto.decrypt(&certificate) {
                    if fingerprint == fingerprint_decrypted {
                        return Ok(AuthState(true));
                    }
                };
            }
        };

        Ok(AuthState(false))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use axum::body::Body;
    use axum::http::{Method, Request, StatusCode};
    use axum::response::{IntoResponse, Response};
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    use crate::crypto::tests::get_crypto;
    use crate::utils::into_layer;
    use crate::utils::tests::sleep_async;

    pub fn gen_auth(crypto: &Crypto) -> String {
        let fingerprint = "fingerprint for test";
        let certificate = crypto.encrypt(fingerprint).unwrap();

        let auth = Authorization {
            fingerprint: fingerprint.to_string(),
            certificate: Some(certificate),
        };

        serde_json::to_string(&auth).unwrap()
    }

    async fn index(AuthState(is_authorized): AuthState) -> Response {
        if is_authorized {
            StatusCode::OK.into_response()
        } else {
            StatusCode::UNAUTHORIZED.into_response()
        }
    }

    #[tokio::test]
    async fn test_auth_from_request() {
        let crypto = get_crypto();

        let router = Router::new()
            .route("/", get(index))
            .layer(into_layer(crypto.clone()));

        let authorization = gen_auth(&crypto);

        let req = Request::builder()
            .method(Method::GET)
            .uri("/")
            .header("Authorization", authorization)
            .body(Body::empty())
            .unwrap();

        let res = router.clone().oneshot(req).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);

        let req = Request::builder()
            .method(Method::GET)
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let res = router.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        sleep_async(1).await;
    }
}
