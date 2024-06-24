/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use serde::{Deserialize, Serialize};

use super::local::LocalStorage;
use super::minio::Minio;

#[derive(Clone)]
pub enum StorageClient {
    Local(LocalStorage),
    Minio(Minio),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    pub number: u16,
    pub etag: String,
}
