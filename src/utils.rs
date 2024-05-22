/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::extract::Extension;
use chrono::Utc;
use sanitize_filename::sanitize;
use std::sync::Arc;

pub fn get_current_timestamp() -> i64 {
    Utc::now().timestamp_millis()
}

pub fn rename(filename: &str, timestamp: i64) -> String {
    let parts: Vec<&str> = filename.split(".").collect();

    let timestamp_str = timestamp.to_string();
    let timestamp_second = &timestamp_str[..timestamp_str.len() - 3];

    let new_filename = if parts.len() > 1 {
        // file has extension
        format!("{}_{}.{}", parts[0], timestamp_second, parts[1])
    } else {
        // file doesn't have extension
        format!("{}_{}", parts[0], timestamp_second)
    };

    // prevent path issues
    sanitize(new_filename)
}

pub fn into_layer<T>(data: T) -> Extension<Arc<T>> {
    Extension(Arc::new(data))
}

#[cfg(test)]
pub mod tests {
    use axum::body::Body;
    use axum::response::Response;
    use http_body_util::BodyExt;

    use crate::error::Error::{DefaultError, ToStrError};
    use crate::error::Result;

    pub trait ResponseExt {
        async fn to_string(self) -> Result<String>;
    }

    impl ResponseExt for Response<Body> {
        async fn to_string(self) -> Result<String> {
            let result = String::from_utf8(
                self.into_body()
                    .collect()
                    .await
                    .map_err(|e| DefaultError(format!("failed to collect response body: {}", e)))?
                    .to_bytes()
                    .to_vec(),
            )
            .map_err(|e| ToStrError(format!("failed to convert response body to string: {}", e)))?;

            Ok(result)
        }
    }
}
