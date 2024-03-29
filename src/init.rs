/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::client;

pub async fn init() {
    println!("Initializing minio...");
    init_minio().await;
    println!("Minio initialized.");

    println!("Initializing mysql...");
    init_mysql().await;
    println!("MySql initialized.");

    println!("All initialization completed.");
}

async fn init_minio() {
    let storage = client::get_storage();

    storage.init().await.unwrap();
}

async fn init_mysql() {
    let database = client::get_database().await;

    database.init().await.unwrap();
}
