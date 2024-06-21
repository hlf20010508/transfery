/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};

use super::models::token::{self, TokenNewItem};
use super::Database;
use crate::error::ErrorType::InternalServerError;
use crate::error::{Error, Result};
use crate::utils::get_current_timestamp;

impl Database {
    pub async fn insert_token(
        &self,
        TokenNewItem {
            token,
            name,
            expiration_timestamp,
        }: TokenNewItem,
    ) -> Result<()> {
        let insert_item = token::ActiveModel {
            token: Set(token),
            name: Set(name),
            last_use_timestamp: Set(get_current_timestamp()),
            expiration_timestamp: Set(expiration_timestamp),
            ..Default::default()
        };

        token::Entity::insert(insert_item)
            .exec(&self.connection)
            .await
            .map_err(|e| Error::context(InternalServerError, e, "failed to insert token"))?;

        Ok(())
    }

    pub async fn query_token_items(&self) -> Result<Vec<token::Model>> {
        let token_items = token::Entity::find()
            .all(&self.connection)
            .await
            .map_err(|e| Error::context(InternalServerError, e, "failed to query token items"))?;

        Ok(token_items)
    }

    pub async fn remove_token(&self, token: String) -> Result<()> {
        token::Entity::delete_many()
            .filter(token::Column::Token.eq(token))
            .exec(&self.connection)
            .await
            .map_err(|e| Error::context(InternalServerError, e, "failed to remove token"))?;

        Ok(())
    }

    pub async fn update_token(&self, token: &str, last_use_timestamp: i64) -> Result<()> {
        token::Entity::update_many()
            .filter(token::Column::Token.eq(token))
            .col_expr(
                token::Column::LastUseTimestamp,
                Expr::value(last_use_timestamp),
            )
            .exec(&self.connection)
            .await
            .map_err(|e| Error::context(InternalServerError, e, "failed to update token"))?;

        Ok(())
    }
}
