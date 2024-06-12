/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

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
