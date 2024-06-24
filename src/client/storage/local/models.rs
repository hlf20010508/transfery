/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use std::path::PathBuf;

use super::super::models::Part;

pub trait PathExt {
    fn to_string(&self) -> String;
}

impl PathExt for PathBuf {
    fn to_string(&self) -> String {
        self.to_str().unwrap().to_string()
    }
}

pub struct TaskInfo {
    pub file_name: String,
    pub parts: Vec<Part>,
    pub expiration_timestamp: i64,
}
