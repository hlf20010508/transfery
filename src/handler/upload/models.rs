/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum::async_trait;
use axum::extract::{FromRequest, Multipart, Request};
use serde::{Deserialize, Serialize};

use crate::error::Error::{self, FieldParseError, FromRequestError};
use crate::error::Result;

#[derive(Deserialize, Serialize, Clone)]
pub struct Part {
    pub number: u16,
    pub etag: String,
}

impl Into<minio::s3::types::Part> for Part {
    fn into(self) -> minio::s3::types::Part {
        minio::s3::types::Part {
            number: self.number,
            etag: self.etag,
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct FetchUploadIdJsonParams {
    pub content: String,
    pub timestamp: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FetchUploadIdResponse {
    #[serde(rename = "uploadId")]
    pub upload_id: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
}

#[derive(Deserialize, Debug)]
pub struct UploadPartFormParams {
    #[serde(rename = "filePart")]
    pub file_part: Vec<u8>,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "uploadId")]
    pub upload_id: String,
    #[serde(rename = "partNumber")]
    pub part_number: u16, // at least 1
}

#[async_trait]
impl<S> FromRequest<S> for UploadPartFormParams
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request(req: Request, state: &S) -> Result<Self> {
        let mut multipart = Multipart::from_request(req, state)
            .await
            .map_err(|e| FromRequestError(format!("failed to parse multipart form: {}", e)))?;

        let mut file_name = String::new();
        let mut upload_id = String::new();
        let mut part_number = u16::default();
        let mut file_part = Vec::<u8>::new();

        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|e| FieldParseError(format!("failed to parse multipart field: {}", e)))?
        {
            let name = match field.name() {
                Some(name) => name.to_string(),
                None => continue,
            };

            let data = field
                .bytes()
                .await
                .map_err(|e| FieldParseError(format!("failed to read field bytes: {}", e)))?;

            match name.as_str() {
                "fileName" => {
                    file_name = String::from_utf8(data.to_vec()).map_err(|e| {
                        FieldParseError(format!("failed to parse field fileName: {}", e))
                    })?;
                }
                "uploadId" => {
                    upload_id = String::from_utf8(data.to_vec()).map_err(|e| {
                        FieldParseError(format!("failed to parse field uploadId: {}", e))
                    })?;
                }
                "partNumber" => {
                    let part_number_str = String::from_utf8(data.to_vec()).map_err(|e| {
                        FieldParseError(format!("failed to parse field partNumber: {}", e))
                    })?;
                    part_number = part_number_str.parse::<u16>().map_err(|e| {
                        FieldParseError(format!("failed to parse field partNumber to u16: {}", e))
                    })?;
                }
                "filePart" => {
                    file_part.extend_from_slice(&data);
                }
                _ => {}
            }
        }

        Ok(UploadPartFormParams {
            file_name,
            upload_id,
            part_number,
            file_part,
        })
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct CompleteUploadFormParams {
    pub id: i64,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "uploadId")]
    pub upload_id: String,
    pub parts: Vec<Part>,
}
