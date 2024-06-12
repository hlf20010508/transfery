/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use serde::Deserialize;

#[derive(Deserialize)]
pub struct DownloadUrlQueryParams {
    #[serde[rename = "fileName"]]
    pub file_name: String,
}
