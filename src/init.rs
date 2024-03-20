/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::env::{MINIO_BUCKET, MINIO_ENDPOINT, MINIO_PASSWORD, MINIO_USERNAME};
use crate::storage::Storage;

pub async fn init() {
    println!("Initializing minio...");
    init_minio().await;
    println!("Minio initialized.");

    println!("Initializing mysql...");
    init_mysql().await;
    println!("Mysql initialized.");

    println!("All initialization completed.");
}

async fn init_minio() {
    let storage = Storage::new(
        MINIO_ENDPOINT.as_ref().unwrap(),
        MINIO_USERNAME.as_ref().unwrap(),
        MINIO_PASSWORD.as_ref().unwrap(),
        MINIO_BUCKET.as_ref().unwrap(),
    )
    .unwrap();

    storage.init().await;
}

async fn init_mysql() {}
