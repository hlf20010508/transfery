/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod download;
mod init;
mod list;
mod remove;
#[cfg(test)]
pub mod tests;
mod upload;

use minio::s3::client::Client;

#[derive(Debug, Clone)]
pub struct Minio {
    pub client: Client,
    pub bucket: String,
}
