/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::client;
use crate::env::Env;

pub async fn init(env: &Env) {
    println!("Initializing minio...");
    init_minio(env).await;
    println!("Minio initialized.");

    println!("Initializing mysql...");
    init_mysql(env).await;
    println!("MySql initialized.");

    println!("All initialization completed.");
}

async fn init_minio(env: &Env) {
    let storage = client::get_storage(env);

    storage.init().await.unwrap();
}

async fn init_mysql(env: &Env) {
    let database = client::get_database(env).await;

    database.init().await.unwrap();
}
