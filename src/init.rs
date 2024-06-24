/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::client;
use crate::env::Env;

pub async fn init(env: &Env) {
    println!("Initializing storage...");
    init_storage(env).await;
    println!("Storage initialized.");

    println!("Initializing database...");
    init_database(env).await;
    println!("Database initialized.");

    println!("All initialization completed.");
}

async fn init_storage(env: &Env) {
    let storage = client::get_storage(env).await;

    storage.init().await.unwrap();
}

async fn init_database(env: &Env) {
    let database = client::get_database(env).await;

    database.init().await.unwrap();
}
