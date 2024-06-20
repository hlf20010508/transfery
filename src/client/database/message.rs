/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set};

use super::models::message::{self, MessageItem};
use super::Database;
use crate::error::Error::{SqlExecuteError, SqlQueryError};
use crate::error::Result;

impl Database {
    pub async fn query_message_items(
        &self,
        start: i64,
        number: i64,
        access_private: bool,
    ) -> Result<Vec<message::Model>> {
        let query = {
            let mut query = message::Entity::find()
                .order_by_desc(message::Column::Timestamp)
                .order_by_desc(message::Column::Id)
                .limit(Some(number as u64))
                .offset(Some(start as u64));

            if !access_private {
                query = query.filter(message::Column::IsPrivate.eq(false));
            }

            query
        };

        let items = query
            .all(&self.connection)
            .await
            .map_err(|e| SqlQueryError(format!("failed to query message items: {}", e)))?;

        Ok(items)
    }

    pub async fn query_message_items_after_id(
        &self,
        id: i64,
        access_private: bool,
    ) -> Result<Vec<message::Model>> {
        let query = {
            let mut query = message::Entity::find().filter(message::Column::Id.gt(id));

            if !access_private {
                query = query.filter(message::Column::IsPrivate.eq(false));
            }

            query
        };

        let items = query
            .all(&self.connection)
            .await
            .map_err(|e| SqlQueryError(format!("failed to query message items after id: {}", e)))?;

        Ok(items)
    }

    pub async fn query_message_latest(&self) -> Result<Option<message::Model>> {
        let message = message::Entity::find()
            .order_by_desc(message::Column::Timestamp)
            .order_by_desc(message::Column::Id)
            .filter(message::Column::IsPrivate.eq(true))
            .one(&self.connection)
            .await
            .map_err(|e| SqlQueryError(format!("failed to query message latest: {}", e)))?;

        Ok(message)
    }

    pub async fn insert_message_item(&self, item: MessageItem) -> Result<i64> {
        let insert_item = message::ActiveModel {
            content: Set(item.content),
            timestamp: Set(item.timestamp),
            is_private: Set(item.is_private),
            type_field: Set(item.type_field),
            file_name: Set(item.file_name),
            is_complete: Set(item.is_complete),
            ..Default::default()
        };

        let id = message::Entity::insert(insert_item)
            .exec(&self.connection)
            .await
            .map_err(|e| SqlExecuteError(format!("failed to insert message item: {}", e)))?
            .last_insert_id;

        Ok(id)
    }

    pub async fn remove_message_item(&self, id: i64) -> Result<()> {
        message::Entity::delete_by_id(id)
            .exec(&self.connection)
            .await
            .map_err(|e| SqlExecuteError(format!("failed to remove message item: {}", e)))?;

        Ok(())
    }

    pub async fn remove_message_all(&self) -> Result<()> {
        message::Entity::delete_many()
            .exec(&self.connection)
            .await
            .map_err(|e| SqlExecuteError(format!("failed to remove message all: {}", e)))?;

        Ok(())
    }

    pub async fn update_complete(&self, id: i64) -> Result<()> {
        message::Entity::update_many()
            .filter(message::Column::Id.eq(id))
            .col_expr(message::Column::IsComplete, Expr::value(true))
            .exec(&self.connection)
            .await
            .map_err(|e| SqlExecuteError(format!("failed to update message complete: {}", e)))?;

        Ok(())
    }
}
