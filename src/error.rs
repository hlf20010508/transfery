/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    DefaultError(String),
    FileReadError(String),
    UrlParseError(String),
    StorageClientError(String),
    StorageInitError(String),
    StorageObjectError(String),
    DatabaseClientError(String),
    SqlExecuteError(String),
    SqlQueryError(String),
    SqlGetValueError(String),
    PortParseError(String),
    ToStrError(String),
    ToJsonError(String),
    Base64DecodeError(String),
    CryptoError(String),
    CryptoLoadKeyError(String),
    CryptoEncryptError(String),
    CryptoDecryptError(String),
    CryptoNonceError(String),
    CryptoKeyGenError(String),
    FromRequestError(String),
    FieldParseError(String),
    SocketEmitError(String),
    UnauthorizedError(String),
}

fn into_response(status_code: StatusCode, error_string: String) -> Response {
    (status_code, error_string).into_response()
}

fn internal_server_error_response(error_string: String) -> Response {
    into_response(StatusCode::INTERNAL_SERVER_ERROR, error_string)
}

fn unauthorized_response(error_string: String) -> Response {
    into_response(StatusCode::UNAUTHORIZED, error_string)
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::DefaultError(e) => internal_server_error_response(e),
            Error::FileReadError(e) => internal_server_error_response(e),
            Error::UrlParseError(e) => internal_server_error_response(e),
            Error::StorageClientError(e) => internal_server_error_response(e),
            Error::StorageInitError(e) => internal_server_error_response(e),
            Error::StorageObjectError(e) => internal_server_error_response(e),
            Error::DatabaseClientError(e) => internal_server_error_response(e),
            Error::SqlExecuteError(e) => internal_server_error_response(e),
            Error::SqlQueryError(e) => internal_server_error_response(e),
            Error::SqlGetValueError(e) => internal_server_error_response(e),
            Error::PortParseError(e) => internal_server_error_response(e),
            Error::ToStrError(e) => internal_server_error_response(e),
            Error::ToJsonError(e) => internal_server_error_response(e),
            Error::Base64DecodeError(e) => internal_server_error_response(e),
            Error::CryptoError(e) => internal_server_error_response(e),
            Error::CryptoLoadKeyError(e) => internal_server_error_response(e),
            Error::CryptoEncryptError(e) => internal_server_error_response(e),
            Error::CryptoDecryptError(e) => internal_server_error_response(e),
            Error::CryptoNonceError(e) => internal_server_error_response(e),
            Error::CryptoKeyGenError(e) => internal_server_error_response(e),
            Error::FromRequestError(e) => internal_server_error_response(e),
            Error::FieldParseError(e) => internal_server_error_response(e),
            Error::SocketEmitError(e) => internal_server_error_response(e),
            Error::UnauthorizedError(e) => unauthorized_response(e),
        }
    }
}

fn add_context(message: &str, error: &str) -> String {
    format!("Error in {}: {}", message, error)
}

impl Error {
    pub fn context(self, message: &str) -> Self {
        match self {
            Error::DefaultError(e) => Error::DefaultError(add_context(message, &e)),
            Error::FileReadError(e) => Error::FileReadError(add_context(message, &e)),
            Error::UrlParseError(e) => Error::UrlParseError(add_context(message, &e)),
            Error::StorageClientError(e) => Error::StorageClientError(add_context(message, &e)),
            Error::StorageInitError(e) => Error::StorageInitError(add_context(message, &e)),
            Error::StorageObjectError(e) => Error::StorageObjectError(add_context(message, &e)),
            Error::DatabaseClientError(e) => Error::DatabaseClientError(add_context(message, &e)),
            Error::SqlExecuteError(e) => Error::SqlExecuteError(add_context(message, &e)),
            Error::SqlQueryError(e) => Error::SqlQueryError(add_context(message, &e)),
            Error::SqlGetValueError(e) => Error::SqlGetValueError(add_context(message, &e)),
            Error::PortParseError(e) => Error::PortParseError(add_context(message, &e)),
            Error::ToStrError(e) => Error::ToStrError(add_context(message, &e)),
            Error::ToJsonError(e) => Error::ToJsonError(add_context(message, &e)),
            Error::Base64DecodeError(e) => Error::Base64DecodeError(add_context(message, &e)),
            Error::CryptoError(e) => Error::CryptoError(add_context(message, &e)),
            Error::CryptoLoadKeyError(e) => Error::CryptoLoadKeyError(add_context(message, &e)),
            Error::CryptoEncryptError(e) => Error::CryptoEncryptError(add_context(message, &e)),
            Error::CryptoDecryptError(e) => Error::CryptoDecryptError(add_context(message, &e)),
            Error::CryptoNonceError(e) => Error::CryptoNonceError(add_context(message, &e)),
            Error::CryptoKeyGenError(e) => Error::CryptoKeyGenError(add_context(message, &e)),
            Error::FromRequestError(e) => Error::FromRequestError(add_context(message, &e)),
            Error::FieldParseError(e) => Error::FromRequestError(add_context(message, &e)),
            Error::SocketEmitError(e) => Error::SocketEmitError(add_context(message, &e)),
            Error::UnauthorizedError(e) => Error::UnauthorizedError(add_context(message, &e)),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DefaultError(e) => write!(f, "Default error: {}", e),
            Error::FileReadError(e) => write!(f, "File read error: {}", e),
            Error::UrlParseError(e) => write!(f, "URL parse error: {}", e),
            Error::StorageClientError(e) => write!(f, "Storage client error: {}", e),
            Error::StorageInitError(e) => write!(f, "Storage initialization error: {}", e),
            Error::StorageObjectError(e) => write!(f, "Storage object error: {}", e),
            Error::DatabaseClientError(e) => write!(f, "Database client error: {}", e),
            Error::SqlExecuteError(e) => write!(f, "SQL execute error: {}", e),
            Error::SqlQueryError(e) => write!(f, "SQL query error: {}", e),
            Error::SqlGetValueError(e) => write!(f, "SQL get value error: {}", e),
            Error::PortParseError(e) => write!(f, "Port parse error: {}", e),
            Error::ToStrError(e) => write!(f, "To &str error: {}", e),
            Error::ToJsonError(e) => write!(f, "To json error: {}", e),
            Error::Base64DecodeError(e) => write!(f, "Base64 decode error: {}", e),
            Error::CryptoError(e) => write!(f, "Crypto error: {}", e),
            Error::CryptoLoadKeyError(e) => write!(f, "Crypto load key error: {}", e),
            Error::CryptoEncryptError(e) => write!(f, "Crypto encode error: {}", e),
            Error::CryptoDecryptError(e) => write!(f, "Crypto decode error: {}", e),
            Error::CryptoNonceError(e) => write!(f, "Crypto nonce error: {}", e),
            Error::CryptoKeyGenError(e) => write!(f, "Crypto key gen error: {}", e),
            Error::FromRequestError(e) => write!(f, "From Request error: {}", e),
            Error::FieldParseError(e) => write!(f, "Field parse error: {}", e),
            Error::SocketEmitError(e) => write!(f, "Socket emit error: {}", e),
            Error::UnauthorizedError(e) => write!(f, "Unauthorized error: {}", e),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
