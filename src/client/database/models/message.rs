/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use sea_orm::entity::prelude::DeriveEntityModel;
use sea_orm::sea_query::{ArrayType, ValueType, ValueTypeErr};
use sea_orm::{
    ActiveModelBehavior, ColIdx, ColumnType, DbErr, DerivePrimaryKey, DeriveRelation, EntityTrait,
    EnumIter, PrimaryKeyTrait, QueryResult, TryGetError, TryGetable, Value,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, DeriveEntityModel, Serialize, Deserialize, PartialEq)]
#[sea_orm(table_name = "message")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub content: String,
    pub timestamp: i64,
    #[sea_orm(column_name = "isPrivate")]
    #[serde(rename = "isPrivate")]
    pub is_private: bool,
    #[sea_orm(column_name = "type")]
    #[serde(rename = "type")]
    pub type_field: MessageItemType,
    #[sea_orm(column_name = "fileName")]
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    #[sea_orm(column_name = "isComplete")]
    #[serde(rename = "isComplete")]
    pub is_complete: Option<bool>,
}

#[derive(Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum MessageItemType {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "file")]
    File,
}

impl MessageItemType {
    fn to_string(&self) -> String {
        match self {
            Self::Text => self.to_str().to_string(),
            Self::File => self.to_str().to_string(),
        }
    }

    fn to_str(&self) -> &str {
        match self {
            Self::Text => "text",
            Self::File => "file",
        }
    }
}

impl Into<sea_orm::Value> for MessageItemType {
    fn into(self) -> sea_orm::Value {
        match self {
            Self::Text => sea_orm::Value::String(Some(Box::new(self.to_string()))),
            Self::File => sea_orm::Value::String(Some(Box::new(self.to_string()))),
        }
    }
}

impl TryGetable for MessageItemType {
    fn try_get_by<I: ColIdx>(res: &QueryResult, index: I) -> Result<Self, TryGetError> {
        let value: String = res.try_get_by(index)?;

        match value.as_str() {
            "text" => Ok(MessageItemType::Text),
            "file" => Ok(MessageItemType::File),
            _ => Err(TryGetError::DbErr(DbErr::Type(format!(
                "message item type value should be one of text and file: {}",
                value
            )))),
        }
    }
}

impl ValueType for MessageItemType {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::String(Some(value)) => match value.as_str() {
                "text" => Ok(MessageItemType::Text),
                "file" => Ok(MessageItemType::File),
                _ => Err(ValueTypeErr),
            },
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        "MessageItemType".to_string()
    }

    fn array_type() -> ArrayType {
        ArrayType::String
    }

    fn column_type() -> ColumnType {
        ColumnType::String(None)
    }
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct MessageItem {
    pub content: String,
    pub timestamp: i64,
    #[serde(rename = "isPrivate")]
    pub is_private: bool,
    #[serde(rename = "type")]
    pub type_field: MessageItemType,
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    #[serde(rename = "isComplete")]
    pub is_complete: Option<bool>,
}

impl MessageItem {
    pub fn new_text(content: &str, timestamp: i64, is_private: bool) -> Self {
        Self {
            content: content.to_string(),
            timestamp,
            is_private,
            type_field: MessageItemType::Text,
            file_name: None,
            is_complete: None,
        }
    }

    pub fn new_file(
        content: &str,
        timestamp: i64,
        is_private: bool,
        file_name: &str,
        is_complete: bool,
    ) -> Self {
        Self {
            content: content.to_string(),
            timestamp,
            is_private,
            type_field: MessageItemType::File,
            file_name: Some(file_name.to_string()),
            is_complete: Some(is_complete),
        }
    }
}
