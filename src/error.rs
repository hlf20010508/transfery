/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use std::fmt::Display;
use actix_web::ResponseError;

#[derive(Debug)]
pub enum Error {
    UrlParseError(String),
    StorageClientError(String),
    StorageInitError(String),
    StorageObjectError(String),
    DatabaseClientError(String),
    SqlExecuteError(String),
    SqlQueryError(String),
    SqlGetValueError(String),
    PortParseError(String),
    SecretKeyGenError(String),
}

impl Error {
    pub fn context(&self, message: &str) -> Self {
        match self {
            Error::UrlParseError(e) => Error::UrlParseError(format!("Error in {}: {}", message, e)),
            Error::StorageClientError(e) => Error::StorageClientError(format!("Error in {}: {}", message, e)),
            Error::StorageInitError(e) => Error::StorageInitError(format!("Error in {}: {}", message, e)),
            Error::StorageObjectError(e) => Error::StorageObjectError(format!("Error in {}: {}", message, e)),
            Error::DatabaseClientError(e) => Error::DatabaseClientError(format!("Error in {}: {}", message, e)),
            Error::SqlExecuteError(e) => Error::SqlExecuteError(format!("Error in {}: {}", message, e)),
            Error::SqlQueryError(e) => Error::SqlQueryError(format!("Error in {}: {}", message, e)),
            Error::SqlGetValueError(e) => Error::SqlGetValueError(format!("Error in {}: {}", message, e)),
            Error::PortParseError(e) => Error::PortParseError(format!("Error in {}: {}", message, e)),
            Error::SecretKeyGenError(e) => Error::SecretKeyGenError(format!("Error in {}: {}", message, e)),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UrlParseError(e) => write!(f, "URL parse error: {}", e),
            Error::StorageClientError(e) => write!(f, "Storage client error: {}", e),
            Error::StorageInitError(e) => write!(f, "Storage initialization error: {}", e),
            Error::StorageObjectError(e) => write!(f, "Storage object error: {}", e),
            Error::DatabaseClientError(e) => write!(f, "Database client error: {}", e),
            Error::SqlExecuteError(e) => write!(f, "SQL execute error: {}", e),
            Error::SqlQueryError(e) => write!(f, "SQL query error: {}", e),
            Error::SqlGetValueError(e) => write!(f, "SQL get value error: {}", e),
            Error::PortParseError(e) => write!(f, "Port parse error: {}", e),
            Error::SecretKeyGenError(e) => write!(f, "Secret key generation error: {}", e),
        }
    }
}

impl ResponseError for Error {}

pub type Result<T> = std::result::Result<T, Error>;
