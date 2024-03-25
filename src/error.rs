/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use std::fmt::Display;

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

pub type Result<T> = std::result::Result<T, Error>;
