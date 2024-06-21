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
pub enum ErrorType {
    InternalServerError,
    UnauthorizedError,
}

#[derive(Debug)]
pub struct Error {
    error_type: ErrorType,
    message: String,
}

impl Error {
    pub fn new<T>(error_type: ErrorType, message: T) -> Self
    where
        T: Display,
    {
        Error {
            error_type,
            message: message.to_string(),
        }
    }

    pub fn context<T, U>(error_type: ErrorType, error: T, message: U) -> Self
    where
        T: Display,
        U: Display,
    {
        Error {
            error_type,
            message: format!("{message}: {error}"),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.error_type {
            ErrorType::InternalServerError => write!(f, "Internal server error: {}", self.message),
            ErrorType::UnauthorizedError => write!(f, "Unauthorized error: {}", self.message),
        }
    }
}

fn into_response(status_code: StatusCode, error_string: String) -> Response {
    (status_code, error_string).into_response()
}

fn internal_server_error_response(error_string: String) -> Response {
    tracing::debug!("Internal server error: {}", error_string);
    into_response(StatusCode::INTERNAL_SERVER_ERROR, error_string)
}

fn unauthorized_response(error_string: String) -> Response {
    tracing::debug!("Unauthorized error: {}", error_string);
    into_response(StatusCode::UNAUTHORIZED, error_string)
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self.error_type {
            ErrorType::InternalServerError => internal_server_error_response(self.message),
            ErrorType::UnauthorizedError => unauthorized_response(self.message),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
pub mod tests {
    use super::*;

    pub trait ServerExt {
        fn serialize_error<E>(e: E) -> Self
        where
            E: Display;

        fn deserialize_error<E>(e: E) -> Self
        where
            E: Display;

        fn req_build_error<E>(e: E) -> Self
        where
            E: Display;

        fn req_send_error<E>(e: E) -> Self
        where
            E: Display;

        fn tcp_listener_create_error<E>(e: E) -> Self
        where
            E: Display;

        fn tcp_get_address_error<E>(e: E) -> Self
        where
            E: Display;

        fn socketio_connect_error<E>(e: E) -> Self
        where
            E: Display;
    }

    impl ServerExt for Error {
        fn serialize_error<E>(e: E) -> Self
        where
            E: Display,
        {
            Error::context(
                ErrorType::InternalServerError,
                e,
                "failed to serialize value",
            )
        }

        fn deserialize_error<E>(e: E) -> Self
        where
            E: Display,
        {
            Error::context(
                ErrorType::InternalServerError,
                e,
                "failed to deserialize value",
            )
        }

        fn req_build_error<E>(e: E) -> Self
        where
            E: Display,
        {
            Error::context(ErrorType::InternalServerError, e, "failed to build request")
        }

        fn req_send_error<E>(e: E) -> Self
        where
            E: Display,
        {
            Error::context(ErrorType::InternalServerError, e, "failed to send request")
        }

        fn tcp_listener_create_error<E>(e: E) -> Self
        where
            E: Display,
        {
            Error::context(
                ErrorType::InternalServerError,
                e,
                "failed to create tcp listener",
            )
        }

        fn tcp_get_address_error<E>(e: E) -> Self
        where
            E: Display,
        {
            Error::context(
                ErrorType::InternalServerError,
                e,
                "failed to get tcp address",
            )
        }

        fn socketio_connect_error<E>(e: E) -> Self
        where
            E: Display,
        {
            Error::context(
                ErrorType::InternalServerError,
                e,
                "failed to connect to socketio server",
            )
        }
    }
}
