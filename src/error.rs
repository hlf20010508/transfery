/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use actix_web::ResponseError;
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
    ToStrError(String),
    ToJsonError(String),
    Base64DecodeError(String),
    CryptoError(String),
    CryptoLoadKeyError(String),
    CryptoEncryptError(String),
    CryptoDecryptError(String),
    CryptoNonceError(String),
    CryptoKeyGenError(String),
}

impl Error {
    pub fn context(&self, message: &str) -> Self {
        match self {
            Error::UrlParseError(e) => Error::UrlParseError(format!("Error in {}: {}", message, e)),
            Error::StorageClientError(e) => {
                Error::StorageClientError(format!("Error in {}: {}", message, e))
            }
            Error::StorageInitError(e) => {
                Error::StorageInitError(format!("Error in {}: {}", message, e))
            }
            Error::StorageObjectError(e) => {
                Error::StorageObjectError(format!("Error in {}: {}", message, e))
            }
            Error::DatabaseClientError(e) => {
                Error::DatabaseClientError(format!("Error in {}: {}", message, e))
            }
            Error::SqlExecuteError(e) => {
                Error::SqlExecuteError(format!("Error in {}: {}", message, e))
            }
            Error::SqlQueryError(e) => Error::SqlQueryError(format!("Error in {}: {}", message, e)),
            Error::SqlGetValueError(e) => {
                Error::SqlGetValueError(format!("Error in {}: {}", message, e))
            }
            Error::PortParseError(e) => {
                Error::PortParseError(format!("Error in {}: {}", message, e))
            }
            Error::ToStrError(e) => Error::ToStrError(format!("Error in {}: {}", message, e)),
            Error::ToJsonError(e) => Error::ToJsonError(format!("Error in {}: {}", message, e)),
            Error::Base64DecodeError(e) => {
                Error::Base64DecodeError(format!("Error in {}: {}", message, e))
            }
            Error::CryptoError(e) => Error::CryptoError(format!("Error in {}: {}", message, e)),
            Error::CryptoLoadKeyError(e) => {
                Error::CryptoLoadKeyError(format!("Error in {}: {}", message, e))
            }
            Error::CryptoEncryptError(e) => {
                Error::CryptoEncryptError(format!("Error in {}: {}", message, e))
            }
            Error::CryptoDecryptError(e) => {
                Error::CryptoDecryptError(format!("Error in {}: {}", message, e))
            }
            Error::CryptoNonceError(e) => {
                Error::CryptoNonceError(format!("Error in {}: {}", message, e))
            }
            Error::CryptoKeyGenError(e) => {
                Error::CryptoKeyGenError(format!("Error in {}: {}", message, e))
            }
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
            Error::ToStrError(e) => write!(f, "To &str error: {}", e),
            Error::ToJsonError(e) => write!(f, "To json error: {}", e),
            Error::Base64DecodeError(e) => write!(f, "Base64 decode error: {}", e),
            Error::CryptoError(e) => write!(f, "Crypto error: {}", e),
            Error::CryptoLoadKeyError(e) => write!(f, "Crypto load key error: {}", e),
            Error::CryptoEncryptError(e) => write!(f, "Crypto encode error: {}", e),
            Error::CryptoDecryptError(e) => write!(f, "Crypto decode error: {}", e),
            Error::CryptoNonceError(e) => write!(f, "Crypto nonce error: {}", e),
            Error::CryptoKeyGenError(e) => write!(f, "Crypto key gen error: {}", e),
        }
    }
}

impl ResponseError for Error {}

pub type Result<T> = std::result::Result<T, Error>;
