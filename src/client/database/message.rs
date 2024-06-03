/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use sqlx::mysql::MySql;
use sqlx::Executor;

use super::{Database, MessageItem};

use crate::env::MYSQL_TABLE_MESSAGE;
use crate::error::Error::{SqlExecuteError, SqlQueryError};
use crate::error::Result;

impl Database {
    pub async fn query_message_items(
        &self,
        start: u32,
        number: u8,
        access_private: bool,
    ) -> Result<Vec<MessageItem>> {
        let sql = {
            let mut sql = format!("select * from `{}` ", MYSQL_TABLE_MESSAGE);
            if !access_private {
                sql.push_str("where isPrivate = false ");
            }
            sql.push_str("order by timestamp desc, id desc limit ?, ?");

            sql
        };

        let query = sqlx::query::<MySql>(&sql)
            .bind(start)
            .bind(number)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| SqlQueryError(format!("MySql query message items failed: {}", e)))?;

        let result: Vec<MessageItem> = query
            .into_iter()
            .map(|row| MessageItem::from(row))
            .collect();

        Ok(result)
    }

    pub async fn query_message_items_after_id(
        &self,
        id: u32,
        access_private: bool,
    ) -> Result<Vec<MessageItem>> {
        let sql = {
            let mut sql = format!("select * from `{}` where id > ?", MYSQL_TABLE_MESSAGE);
            if !access_private {
                sql.push_str(" and isPrivate = false ");
            }

            sql
        };

        let query = sqlx::query::<MySql>(&sql)
            .bind(id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                SqlQueryError(format!("MySql query message items after id failed: {}", e))
            })?;

        let result: Vec<MessageItem> = query
            .into_iter()
            .map(|row| MessageItem::from(row))
            .collect();

        Ok(result)
    }

    pub async fn query_message_latest(&self) -> Option<MessageItem> {
        let sql = format!(
            "select * from `{}` where isPrivate = true order by timestamp desc, id desc limit 1",
            MYSQL_TABLE_MESSAGE
        );

        match sqlx::query::<MySql>(&sql).fetch_one(&self.pool).await {
            Ok(row) => Some(MessageItem::from(row)),
            Err(_) => None,
        }
    }

    pub async fn insert_message_item(&self, item: MessageItem) -> Result<u64> {
        let sql = format!(
            "insert into `{}` (
                content,
                timestamp,
                isPrivate,
                type,
                fileName,
                isComplete
            )
            values (?, ?, ?, ?, ?, ?)",
            MYSQL_TABLE_MESSAGE
        );

        let query = sqlx::query::<MySql>(&sql)
            .bind(item.content)
            .bind(item.timestamp)
            .bind(item.is_private)
            .bind(item.type_field)
            .bind(item.file_name)
            .bind(item.is_complete);

        let id = self
            .pool
            .execute(query)
            .await
            .map_err(|e| SqlExecuteError(format!("MySql insert message item failed: {}", e)))?
            .last_insert_id();

        Ok(id)
    }

    pub async fn remove_message_item(&self, id: i64) -> Result<()> {
        let sql = format!("delete from `{}` where id = ?", MYSQL_TABLE_MESSAGE);

        let query = sqlx::query(&sql).bind(id);

        self.pool
            .execute(query)
            .await
            .map_err(|e| SqlExecuteError(format!("MySql remove message item failed: {}", e)))?;

        Ok(())
    }

    pub async fn remove_message_all(&self) -> Result<()> {
        let sql = format!("delete from `{}`", MYSQL_TABLE_MESSAGE);

        let query = sqlx::query(&sql);

        self.pool
            .execute(query)
            .await
            .map_err(|e| SqlExecuteError(format!("MySql remove message all failed: {}", e)))?;

        Ok(())
    }

    pub async fn update_complete(&self, id: i64) -> Result<()> {
        let sql = format!(
            "update `{}` set isComplete = true where id = ?",
            MYSQL_TABLE_MESSAGE
        );

        let query = sqlx::query(&sql).bind(id);

        self.pool
            .execute(query)
            .await
            .map_err(|e| SqlExecuteError(format!("MySql update complete failed: {}", e)))?;

        Ok(())
    }
}
