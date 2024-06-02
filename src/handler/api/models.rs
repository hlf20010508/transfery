/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::extract::{FromRequest, Query, Request};
use axum::http::Method;
use axum::{async_trait, Json};
use serde::{Deserialize, Serialize};

use crate::crypto::Crypto;
use crate::env::Env;
use crate::error::Error::{self, FieldParseError, FromRequestError, UnauthorizedError};
use crate::error::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct Token(pub String);

impl Token {
    pub fn to_string(self) -> String {
        self.0
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Account {
    pub username: String,
    pub password: String,
}

impl Account {
    pub fn from(token: Token, crypto: &Crypto) -> Result<Self> {
        let account_json = crypto
            .decrypt(&token.to_string())
            .map_err(|e| UnauthorizedError(format!("failed to decrypt token: {}", e)))?;

        let account = serde_json::from_str::<Account>(&account_json)
            .map_err(|e| FieldParseError(format!("failed to parse account from token: {}", e)))?;

        Ok(account)
    }

    pub fn is_valid(&self, env: &Env) -> bool {
        self.username == env.username && self.password == env.password
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PushTextParams {
    pub content: String,
    pub token: Token,
}

#[async_trait]
impl<S> FromRequest<S> for PushTextParams
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request(req: Request, state: &S) -> Result<Self> {
        let method = req.method().clone();

        let data = match method {
            Method::GET => {
                let Query(data) = Query::<PushTextParams>::from_request(req, state)
                    .await
                    .map_err(|e| {
                        FromRequestError(format!("failed to parse query for PushTextParams: {}", e))
                    })?;

                data
            }
            Method::POST => {
                let Json(data) = Json::<PushTextParams>::from_request(req, state)
                    .await
                    .map_err(|e| {
                        FromRequestError(format!("failed to parse json for PushTextParams: {}", e))
                    })?;

                data
            }
            _ => {
                return Err(FromRequestError(
                    "unsupported method for PushTextParams".to_string(),
                ))
            }
        };

        Ok(data)
    }
}

#[derive(Debug, Deserialize)]
pub struct LatestTextParams {
    pub token: Token,
}