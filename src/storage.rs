/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use minio::s3::args::{BucketExistsArgs, MakeBucketArgs};
use minio::s3::client::Client;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;

use crate::error;

pub struct Storage {
    client: Client,
    bucket: String,
}

impl Storage {
    pub fn new(
        endpoint: &str,
        username: &str,
        password: &str,
        bucket: &str,
    ) -> Result<Self, error::Error> {
        let base_url = endpoint
            .parse::<BaseUrl>()
            .map_err(|_| error::Error::UrlParseError("Minio endpoint parse failed.".to_string()))?;

        let static_provider = StaticProvider::new(username, password, None);

        let client = Client::new(base_url, Some(Box::new(static_provider)), None, None)
            .map_err(|e| error::Error::MinioClientError(e.to_string()))?;

        Ok(Self {
            client,
            bucket: bucket.to_string(),
        })
    }

    pub async fn init(&self) {
        self.create_buffer_if_not_exists().await;
    }

    async fn create_buffer_if_not_exists(&self) {
        let exists = self
            .client
            .bucket_exists(
                &BucketExistsArgs::new(&self.bucket)
                    .expect("Minio bucket name invalid when checking existence."),
            )
            .await
            .expect("Minio checking bucket existence await failed.");

        if !exists {
            self.client
                .make_bucket(
                    &MakeBucketArgs::new(&self.bucket)
                        .expect("Minio bucket name invalid when making."),
                )
                .await
                .expect("Minio making bucket await failed.");
        }
    }
}
