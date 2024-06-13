/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use sqlx::mysql::MySql;
use sqlx::Executor;

use super::{Database, NewTokenItem, TokenItem};

use crate::env::MYSQL_TABLE_TOKEN;
use crate::error::Error::{SqlExecuteError, SqlQueryError};
use crate::error::Result;
use crate::utils::get_current_timestamp;

impl Database {
    pub async fn insert_token(
        &self,
        NewTokenItem {
            token,
            name,
            expiration_timestamp,
        }: NewTokenItem,
    ) -> Result<()> {
        let sql = format!(
            "insert into `{}` (
                token,
                name,
                lastUseTimestamp,
                expirationTimestamp
            )
            values (?, ?, ?, ?)",
            MYSQL_TABLE_TOKEN
        );

        let query = sqlx::query(&sql)
            .bind(token)
            .bind(name)
            .bind(get_current_timestamp())
            .bind(expiration_timestamp);

        self.pool
            .execute(query)
            .await
            .map_err(|e| SqlExecuteError(format!("MySql insert token failed: {}", e)))?;

        Ok(())
    }

    pub async fn query_token_items(&self) -> Result<Vec<TokenItem>> {
        let sql = format!("select * from `{}`", MYSQL_TABLE_TOKEN);

        let query = sqlx::query::<MySql>(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| SqlQueryError(format!("MySql query token failed: {}", e)))?;

        let result = query
            .into_iter()
            .map(|row| TokenItem::from(row))
            .collect::<Vec<TokenItem>>();

        Ok(result)
    }

    pub async fn remove_token(&self, token: String) -> Result<()> {
        let sql = format!("delete from `{}` where token = ?", MYSQL_TABLE_TOKEN);

        let query = sqlx::query(&sql).bind(token);

        self.pool
            .execute(query)
            .await
            .map_err(|e| SqlExecuteError(format!("MySql remove token failed: {}", e)))?;

        Ok(())
    }

    pub async fn update_token(&self, token: &str, last_use_timestamp: i64) -> Result<()> {
        let sql = format!(
            "update `{}` set lastUseTimestamp = ? where token = ?",
            MYSQL_TABLE_TOKEN
        );

        let query = sqlx::query(&sql).bind(last_use_timestamp).bind(token);

        self.pool
            .execute(query)
            .await
            .map_err(|e| SqlExecuteError(format!("MySql update token failed: {}", e)))?;

        Ok(())
    }
}
