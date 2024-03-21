/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use lazy_static::lazy_static;
use pico_args::{Arguments, Error};
use std::fmt::Display;
use std::str::FromStr;

fn get_arg_value<T>(arg_name: &'static str) -> Result<T, Error>
where
    T: FromStr,
    T::Err: Display,
{
    let mut args = Arguments::from_env();
    let value: T = args.value_from_str(arg_name)?;
    Ok(value)
}

lazy_static! {
    pub static ref MINIO_ENDPOINT: Result<String, Error> = get_arg_value("--minio-endpoint");
    pub static ref MINIO_USERNAME: Result<String, Error> = get_arg_value("--minio-username");
    pub static ref MINIO_PASSWORD: Result<String, Error> = get_arg_value("--minio-password");
    pub static ref MINIO_BUCKET: Result<String, Error> = get_arg_value("--minio-bucket");
    pub static ref MYSQL_ENDPOINT: Result<String, Error> = get_arg_value("--mysql-endpoint");
    pub static ref MYSQL_USERNAME: Result<String, Error> = get_arg_value("--mysql-username");
    pub static ref MYSQL_PASSWORD: Result<String, Error> = get_arg_value("--mysql-password");
    pub static ref MYSQL_DATABASE: Result<String, Error> = get_arg_value("--mysql-database");
}
