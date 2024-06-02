/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use sqlx::mysql::MySql;
use sqlx::Executor;

use super::{Database, DeviceItem};

use crate::env::MYSQL_TABLE_DEVICE;
use crate::error::Error::{SqlExecuteError, SqlQueryError};
use crate::error::Result;

impl Database {
    pub async fn insert_device(&self, device_item: DeviceItem) -> Result<()> {
        let fingerprint_exists = {
            let sql = format!(
                "select count(*) from {} where fingerprint = \"{}\"",
                MYSQL_TABLE_DEVICE, device_item.fingerprint
            );

            let (count,) = sqlx::query_as::<MySql, (i64,)>(&sql)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| {
                    SqlQueryError(format!("MySql query fingerprint count failed: {}", e))
                })?;

            count > 0
        };

        if fingerprint_exists {
            self.update_device(device_item.clone()).await?;
        } else {
            let sql = format!(
                "insert into `{}` (
                    fingerprint,
                    browser,
                    lastUseTimestamp,
                    expirationTimestamp
                )
                values (?, ?, ?, ?)",
                MYSQL_TABLE_DEVICE
            );

            let query = sqlx::query(&sql)
                .bind(device_item.fingerprint)
                .bind(device_item.browser)
                .bind(device_item.last_use_timestamp)
                .bind(device_item.expiration_timestamp);

            self.pool
                .execute(query)
                .await
                .map_err(|e| SqlExecuteError(format!("MySql insert device failed: {}", e)))?;
        }

        Ok(())
    }

    pub async fn update_device(
        &self,
        DeviceItem {
            fingerprint,
            browser,
            last_use_timestamp,
            expiration_timestamp,
        }: DeviceItem,
    ) -> Result<()> {
        let mut sql = format!("update `{}` set", MYSQL_TABLE_DEVICE);
        let mut params = Vec::<String>::new();

        if let Some(browser) = browser {
            sql = format!("{} browser = ?,", sql);
            params.push(browser);
        }

        if let Some(last_use_timestamp) = last_use_timestamp {
            sql = format!("{} lastUseTimestamp = ?,", sql);
            params.push(last_use_timestamp.to_string());
        }

        if let Some(expiration_timestamp) = expiration_timestamp {
            sql = format!("{} expirationTimestamp = ?,", sql);
            params.push(expiration_timestamp.to_string());
        }

        sql = sql.trim_end_matches(",").to_string();
        sql = format!("{} where fingerprint = ?", sql);

        params.push(fingerprint);

        let mut query = sqlx::query(&sql);

        for param in params.iter() {
            query = query.bind(param);
        }

        self.pool
            .execute(query)
            .await
            .map_err(|e| SqlExecuteError(format!("MySql update device failed: {}", e)))?;

        Ok(())
    }

    pub async fn query_device_items(&self) -> Result<Vec<DeviceItem>> {
        let sql = format!("select * from `{}`", MYSQL_TABLE_DEVICE);

        let query = sqlx::query::<MySql>(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| SqlQueryError(format!("MySql query device failed: {}", e)))?;

        let result = query
            .into_iter()
            .map(|row| DeviceItem::from(row))
            .collect::<Vec<DeviceItem>>();

        Ok(result)
    }

    pub async fn remove_device(&self, fingerprint: &str) -> Result<()> {
        let sql = format!("delete from `{}` where fingerprint = ?", MYSQL_TABLE_DEVICE);
        let query = sqlx::query(&sql).bind(fingerprint);

        self.pool
            .execute(query)
            .await
            .map_err(|e| SqlExecuteError(format!("MySql remove device failed: {}", e)))?;

        Ok(())
    }
}
