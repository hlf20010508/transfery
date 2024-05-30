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
pub struct Authorization {
    pub fingerprint: String,
    pub certificate: Option<String>,
}

#[async_trait]
impl<S> FromRequestParts<S> for Authorization
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(req: &mut Parts, _state: &S) -> Result<Self> {
        let authorization = req
            .headers
            .get("Authorization")
            .ok_or(UnauthorizedError(
                "Failed to parse Authorization".to_string(),
            ))?
            .to_str()
            .map_err(|e| {
                UnauthorizedError(format!("Failed to convert auth header to &str: {}", e))
            })?;

        let auth_json = serde_json::from_str::<Authorization>(authorization).map_err(|e| {
            UnauthorizedError(format!("Failed to convert auth header &str to json: {}", e))
        })?;

        Ok(auth_json)
    }
}

pub struct AuthState(pub bool);

#[async_trait]
impl<S> FromRequestParts<S> for AuthState
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self> {
        if let Ok(Authorization {
            fingerprint,
            certificate,
        }) = Authorization::from_request_parts(req, state).await
        {
            if let Some(certificate) = certificate {
                let crypto = req
                    .extensions
                    .get::<Arc<Crypto>>()
                    .ok_or(UnauthorizedError("Crypto data not found".to_string()))?;

                if let Ok(fingerprint_decrypted) = crypto.decrypt(&certificate) {
                    if fingerprint == fingerprint_decrypted {
                        return Ok(AuthState(true));
                    }
                };
            }
        }

        Ok(AuthState(false))
    }
}

pub struct AuthChecker;

#[async_trait]
impl<S> FromRequestParts<S> for AuthChecker
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self> {
        let AuthState(is_authorized) = AuthState::from_request_parts(req, state).await?;

        if is_authorized {
            Ok(Self)
        } else {
            Err(UnauthorizedError("Unauthorized".to_string()))
        }
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

    async fn authorization_handler(_authorization: Authorization) -> Response {
        StatusCode::OK.into_response()
    }

    async fn auth_state_handler(AuthState(is_authorized): AuthState) -> Response {
        if is_authorized {
            StatusCode::OK.into_response()
        } else {
            StatusCode::UNAUTHORIZED.into_response()
        }
    }

    async fn auth_checker_handler(_: AuthChecker) -> Response {
        StatusCode::OK.into_response()
    }

    #[tokio::test]
    async fn test_auth_authorization_from_request_parts() {
        let crypto = get_crypto();

        let router = Router::new()
            .route("/", get(authorization_handler))
            .layer(into_layer(crypto.clone()));

        let authorization = gen_auth(&crypto);

        let req = Request::builder()
            .method(Method::GET)
            .uri("/")
            .header("Authorization", authorization)
            .body(Body::empty())
            .unwrap();

        let res = router.oneshot(req).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);

        sleep_async(1).await;
    }

    #[tokio::test]
    async fn test_auth_auth_state_from_request_parts() {
        let crypto = get_crypto();

        let router = Router::new()
            .route("/", get(auth_state_handler))
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

    #[tokio::test]
    async fn test_auth_auth_checker_from_request_parts() {
        let crypto = get_crypto();

        let router = Router::new()
            .route("/", get(auth_checker_handler))
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
