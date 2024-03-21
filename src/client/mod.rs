/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod database;
mod storage;

use crate::env::{
    MINIO_BUCKET, MINIO_ENDPOINT, MINIO_PASSWORD, MINIO_USERNAME, MYSQL_DATABASE, MYSQL_ENDPOINT,
    MYSQL_PASSWORD, MYSQL_USERNAME,
};
use storage::Storage;

pub fn get_storage() -> Storage {
    Storage::new(
        MINIO_ENDPOINT.as_ref().unwrap(),
        MINIO_USERNAME.as_ref().unwrap(),
        MINIO_PASSWORD.as_ref().unwrap(),
        MINIO_BUCKET.as_ref().unwrap(),
    )
    .unwrap()
}
