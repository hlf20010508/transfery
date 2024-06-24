/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

pub mod download;
pub mod init;
pub mod models;
pub mod remove;
#[cfg(test)]
pub mod tests;
pub mod upload;
pub mod utils;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use models::TaskInfo;

#[derive(Clone)]
pub struct LocalStorage {
    path: PathBuf,
    tasks: Arc<Mutex<HashMap<String, TaskInfo>>>,
}
