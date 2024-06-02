/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use serde::{Deserialize, Serialize};
use sqlx::mysql::{MySqlRow, MySqlValueRef};
use sqlx::Row;
use sqlx::{Decode, Encode, MySql, Type};
use std::fmt::Display;

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

impl Display for MessageItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl<'r> Decode<'r, MySql> for MessageItemType {
    fn decode(value: MySqlValueRef<'r>) -> std::result::Result<Self, sqlx::error::BoxDynError> {
        let value = <&str as Decode<MySql>>::decode(value)?;
        match value {
            "text" => Ok(Self::Text),
            "file" => Ok(Self::File),
            _ => Err(Box::<dyn std::error::Error + Send + Sync>::from(format!(
                "Invalid message item type: {}",
                value
            ))),
        }
    }
}

impl<'r> Encode<'r, MySql> for MessageItemType {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::database::HasArguments<'r>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        <&str as Encode<MySql>>::encode(&self.to_string(), buf)
    }
}

impl Type<MySql> for MessageItemType {
    fn type_info() -> <MySql as sqlx::Database>::TypeInfo {
        <&str as Type<MySql>>::type_info()
    }
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct MessageItem {
    pub id: Option<i64>,
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
            id: None,
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
            id: None,
            content: content.to_string(),
            timestamp,
            is_private,
            type_field: MessageItemType::File,
            file_name: Some(file_name.to_string()),
            is_complete: Some(is_complete),
        }
    }
}

impl From<MySqlRow> for MessageItem {
    fn from(row: MySqlRow) -> Self {
        let id = row
            .try_get::<Option<i64>, &str>("id")
            .expect("MySql failed to get id for MessageItem");

        let content = row
            .try_get::<String, &str>("content")
            .expect("MySql failed to get content for MessageItem");

        let timestamp = row
            .try_get::<i64, &str>("timestamp")
            .expect("MySql failed to get timestamp for MessageItem");

        let is_private = row
            .try_get::<bool, &str>("isPrivate")
            .expect("MySql failed to get isPrivate for MessageItem");

        let type_field = row
            .try_get::<MessageItemType, &str>("type")
            .expect("MySql failed to get type for MessageItem");

        let file_name = row
            .try_get::<Option<String>, &str>("fileName")
            .expect("MySql failed to get fileName for MessageItem");

        let is_complete = row
            .try_get::<Option<bool>, &str>("isComplete")
            .expect("MySql failed to get isComplete for MessageItem");

        Self {
            id,
            content,
            timestamp,
            is_private,
            type_field,
            file_name,
            is_complete,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DeviceItem {
    pub fingerprint: String,
    pub browser: Option<String>,
    #[serde(rename = "lastUseTimestamp")]
    pub last_use_timestamp: Option<i64>,
    #[serde(rename = "expirationTimestamp")]
    pub expiration_timestamp: Option<i64>,
}

impl From<MySqlRow> for DeviceItem {
    fn from(row: MySqlRow) -> Self {
        let fingerprint = row
            .try_get::<String, &str>("fingerprint")
            .expect("MySql failed to get fingerprint for DeviceItem");

        let browser = row
            .try_get::<Option<String>, &str>("browser")
            .expect("MySql failed to get browser for DeviceItem");

        let last_use_timestamp = row
            .try_get::<Option<i64>, &str>("lastUseTimestamp")
            .expect("MySql failed to get lastUseTimestamp for DeviceItem");

        let expiration_timestamp = row
            .try_get::<Option<i64>, &str>("expirationTimestamp")
            .expect("MySql failed to get expirationTimestamp for DeviceItem");

        Self {
            fingerprint,
            browser,
            last_use_timestamp,
            expiration_timestamp,
        }
    }
}
