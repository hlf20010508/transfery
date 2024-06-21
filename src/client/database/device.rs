/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};

use super::models::device::{self, DeviceItem, DeviceUpdateItem};
use super::Database;
use crate::error::ErrorType::InternalServerError;
use crate::error::{Error, Result};

impl Database {
    pub async fn insert_device(&self, device_item: DeviceItem) -> Result<()> {
        if self.is_fingerprint_exist(&device_item.fingerprint).await? {
            self.update_device(DeviceUpdateItem {
                fingerprint: device_item.fingerprint.clone(),
                browser: Some(device_item.browser),
                last_use_timestamp: Some(device_item.last_use_timestamp),
                expiration_timestamp: Some(device_item.expiration_timestamp),
            })
            .await?;
        } else {
            let insert_item = device::ActiveModel {
                fingerprint: Set(device_item.fingerprint),
                browser: Set(device_item.browser),
                last_use_timestamp: Set(device_item.last_use_timestamp),
                expiration_timestamp: Set(device_item.expiration_timestamp),
                ..Default::default()
            };

            device::Entity::insert(insert_item)
                .exec(&self.connection)
                .await
                .map_err(|e| Error::context(InternalServerError, e, "failed to insert device"))?;
        }

        Ok(())
    }

    pub async fn is_fingerprint_exist(&self, fingerprint: &str) -> Result<bool> {
        let count = device::Entity::find()
            .filter(device::Column::Fingerprint.eq(fingerprint))
            .count(&self.connection)
            .await
            .map_err(|e| Error::context(InternalServerError, e, "failed to count fingerprint"))?;

        Ok(count > 0)
    }

    pub async fn update_device(
        &self,
        DeviceUpdateItem {
            fingerprint,
            browser,
            last_use_timestamp,
            expiration_timestamp,
        }: DeviceUpdateItem,
    ) -> Result<()> {
        let query = {
            let mut query =
                device::Entity::update_many().filter(device::Column::Fingerprint.eq(fingerprint));

            if let Some(browser) = browser {
                query = query.col_expr(device::Column::Browser, Expr::value(browser));
            }

            if let Some(last_use_timestamp) = last_use_timestamp {
                query = query.col_expr(
                    device::Column::LastUseTimestamp,
                    Expr::value(last_use_timestamp),
                );
            }

            if let Some(expiration_timestamp) = expiration_timestamp {
                query = query.col_expr(
                    device::Column::ExpirationTimestamp,
                    Expr::value(expiration_timestamp),
                );
            }

            query
        };

        query
            .exec(&self.connection)
            .await
            .map_err(|e| Error::context(InternalServerError, e, "failed to update device"))?;

        Ok(())
    }

    pub async fn query_device_items(&self) -> Result<Vec<device::Model>> {
        let items = device::Entity::find()
            .all(&self.connection)
            .await
            .map_err(|e| Error::context(InternalServerError, e, "failed to query device items"))?;

        Ok(items)
    }

    pub async fn remove_device(&self, fingerprint: &str) -> Result<()> {
        device::Entity::delete_many()
            .filter(device::Column::Fingerprint.eq(fingerprint))
            .exec(&self.connection)
            .await
            .map_err(|e| Error::context(InternalServerError, e, "failed to remove device"))?;

        Ok(())
    }
}
