/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

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

pub type Result<T> = std::result::Result<T, Error>;
