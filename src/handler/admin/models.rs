/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use socketioxide::socket::Sid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthParams {
    pub username: String,
    pub password: String,
    #[serde(rename = "rememberMe")]
    pub remember_me: bool,
    pub fingerprint: String,
    pub browser: String,
    pub sid: Sid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceSignOutParams {
    pub fingerprint: String,
    pub sid: Sid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTokenParams {
    pub name: String,
    #[serde(rename = "expirationTimestamp")]
    pub expiration_timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveTokenParams {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenRaw {
    pub username: String,
    pub password: String,
    pub nonce: String,
}

impl TokenRaw {
    pub fn new(username: &str, password: &str) -> Self {
        let nonce = thread_rng()
            .sample_iter(Alphanumeric)
            .take(8)
            .map(char::from)
            .collect::<String>();

        Self {
            username: username.to_string(),
            password: password.to_string(),
            nonce,
        }
    }
}
