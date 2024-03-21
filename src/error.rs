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
    DatabaseClientError(String),
    SqlExecuteError(String),
    SqlQueryError(String),
    PortParseError(String),
}
