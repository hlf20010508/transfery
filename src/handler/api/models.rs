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

use crate::client::database::models::message::{self, MessageItem};
use crate::crypto::Crypto;
use crate::env::Env;
use crate::error::Error;
use crate::error::ErrorType::{InternalServerError, UnauthorizedError};
use crate::error::Result;
use crate::utils::get_current_timestamp;

impl From<(i64, MessageItem)> for message::Model {
    fn from(
        (
            id,
            MessageItem {
                content,
                timestamp,
                is_private,
                file_name,
                is_complete,
                type_field,
                ..
            },
        ): (i64, MessageItem),
    ) -> Self {
        Self {
            id,
            content,
            timestamp,
            is_private,
            file_name,
            is_complete,
            type_field,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Account {
    pub username: String,
    pub password: String,
    #[serde(rename = "expirationTimestamp")]
    pub expiration_timestamp: i64,
}

impl Account {
    pub fn from(token: &str, crypto: &Crypto) -> Result<Self> {
        let account_json = crypto
            .decrypt(token)
            .map_err(|e| Error::context(UnauthorizedError, e, "failed to decrypt token"))?;

        let account = serde_json::from_str::<Account>(&account_json).map_err(|e| {
            Error::context(InternalServerError, e, "failed to parse account from token")
        })?;

        Ok(account)
    }

    pub fn is_valid(&self, env: &Env) -> bool {
        self.username == env.username
            && self.password == env.password
            && get_current_timestamp() < self.expiration_timestamp
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PushTextParams {
    pub content: String,
    pub token: String,
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
                        Error::context(
                            InternalServerError,
                            e,
                            "failed to parse query for PushTextParams",
                        )
                    })?;

                data
            }
            Method::POST => {
                let Json(data) = Json::<PushTextParams>::from_request(req, state)
                    .await
                    .map_err(|e| {
                        Error::context(
                            InternalServerError,
                            e,
                            "failed to parse json for PushTextParams",
                        )
                    })?;

                data
            }
            _ => {
                return Err(Error::new(
                    InternalServerError,
                    "unsupported method for PushTextParams",
                ))
            }
        };

        Ok(data)
    }
}

#[derive(Debug, Deserialize)]
pub struct LatestTextParams {
    pub token: String,
}
