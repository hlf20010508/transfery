/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use pico_args::Arguments;
use std::fmt::Display;
use std::str::FromStr;

fn get_arg_value<T>(arg_name: &'static str) -> T
where
    T: FromStr,
    T::Err: Display,
{
    let mut args = Arguments::from_env();
    let value: T = args.value_from_str(arg_name).unwrap();
    value
}

fn get_arg_value_option<T>(arg_name: &'static str, default: T) -> T
where
    T: FromStr,
    T::Err: Display,
{
    let mut args = Arguments::from_env();
    match args.value_from_str(arg_name) {
        Ok(value) => return value,
        Err(_) => return default,
    };
}

pub struct Env {
    pub port: u16,
    pub item_per_page: u8,
    pub username: String,
    pub password: String,
    pub minio_endpoint: String,
    pub minio_username: String,
    pub minio_password: String,
    pub minio_bucket: String,
    pub mysql_endpoint: String,
    pub mysql_username: String,
    pub mysql_password: String,
    pub mysql_database: String,
}

impl Env {
    pub fn new() -> Self {
        let port = get_arg_value_option("--port", 8080);
        let item_per_page = get_arg_value_option("--item-per-page", 15);
        let username = get_arg_value::<String>("--username");
        let password = get_arg_value::<String>("--password");
        let minio_endpoint = get_arg_value::<String>("--minio-endpoint");
        let minio_username = get_arg_value::<String>("--minio-username");
        let minio_password = get_arg_value::<String>("--minio-password");
        let minio_bucket = get_arg_value::<String>("--minio-bucket");
        let mysql_endpoint = get_arg_value::<String>("--mysql-endpoint");
        let mysql_username = get_arg_value::<String>("--mysql-username");
        let mysql_password = get_arg_value::<String>("--mysql-password");
        let mysql_database = get_arg_value::<String>("--mysql-database");

        Self {
            port,
            item_per_page,
            username,
            password,
            minio_endpoint,
            minio_username,
            minio_password,
            minio_bucket,
            mysql_endpoint,
            mysql_username,
            mysql_password,
            mysql_database,
        }
    }
}

pub static MYSQL_TABLE_MESSAGE: &str = "message";
pub static MYSQL_TABLE_AUTH: &str = "auth";
pub static MYSQL_TABLE_DEVICE: &str = "device";

#[cfg(test)]
pub mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::env;

    pub fn get_env() -> Env {
        dotenv().ok();

        let port = 8080;
        let item_per_page = 15;
        let username = env::var("USERNAME").unwrap();
        let password = env::var("PASSWORD").unwrap();
        let minio_endpoint = env::var("MINIO_ENDPOINT").unwrap();
        let minio_username = env::var("MINIO_USERNAME").unwrap();
        let minio_password = env::var("MINIO_PASSWORD").unwrap();
        let minio_bucket = env::var("MINIO_BUCKET").unwrap();
        let mysql_endpoint = env::var("MYSQL_ENDPOINT").unwrap();
        let mysql_username = env::var("MYSQL_USERNAME").unwrap();
        let mysql_password = env::var("MYSQL_PASSWORD").unwrap();
        let mysql_database = env::var("MYSQL_DATABASE").unwrap();

        Env {
            port,
            item_per_page,
            username,
            password,
            minio_endpoint,
            minio_username,
            minio_password,
            minio_bucket,
            mysql_endpoint,
            mysql_username,
            mysql_password,
            mysql_database,
        }
    }
}
