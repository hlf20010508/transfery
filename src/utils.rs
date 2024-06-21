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
    let timestamp_str = timestamp.to_string();
    let timestamp_second = &timestamp_str[..timestamp_str.len() - 3];

    let last_dot_pos = filename.rfind('.'); // the last dot position

    let new_filename = if let Some(pos) = last_dot_pos {
        // the file has extension
        let name = &filename[..pos];
        let extension = &filename[pos + 1..];
        format!("{}_{}.{}", name, timestamp_second, extension)
    } else {
        format!("{}_{}", filename, timestamp_second)
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

    use crate::error::Error;
    use crate::error::ErrorType::InternalServerError;
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
                    .map_err(|e| {
                        Error::context(InternalServerError, e, "failed to collect response body")
                    })?
                    .to_bytes()
                    .to_vec(),
            )
            .map_err(|e| {
                Error::context(
                    InternalServerError,
                    e,
                    "failed to convert response body to string",
                )
            })?;

            Ok(result)
        }
    }

    pub fn sleep(secs: i64) {
        std::thread::sleep(std::time::Duration::from_secs(secs as u64));
    }

    pub async fn sleep_async(secs: i64) {
        tokio::time::sleep(std::time::Duration::from_secs(secs as u64)).await;
    }
}
