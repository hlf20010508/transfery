/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use serde::{Deserialize, Serialize};
use socketioxide::socket::Sid;

use crate::client::database::{MessageItem, MessageItemType};
use crate::error::Error::FieldParseError;
use crate::error::Result;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct NewItemData {
    pub id: u64,
    pub content: String,
    pub timestamp: i64,
    #[serde(rename = "isPrivate")]
    pub is_private: bool,
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    #[serde(rename = "isComplete")]
    pub is_complete: Option<bool>,
    #[serde(rename = "type")]
    pub type_field: MessageItemType,
}

impl From<(u64, NewItemParams)> for NewItemData {
    fn from((id, params): (u64, NewItemParams)) -> Self {
        Self {
            id,
            content: params.content,
            timestamp: params.timestamp,
            is_private: params.is_private,
            file_name: params.file_name,
            is_complete: params.is_complete,
            type_field: params.type_field,
        }
    }
}

#[derive(Deserialize)]
pub struct PageQueryParams {
    pub size: u32,
}

#[derive(Deserialize)]
pub struct SyncQueryParams {
    #[serde(rename = "latestId")]
    pub latest_id: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NewItemParams {
    pub content: String,
    pub timestamp: i64,
    #[serde(rename = "isPrivate")]
    pub is_private: bool,
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    #[serde(rename = "isComplete")]
    pub is_complete: Option<bool>,
    #[serde(rename = "type")]
    pub type_field: MessageItemType,
    pub sid: Sid,
}

impl From<&NewItemParams> for Result<MessageItem> {
    fn from(new_item: &NewItemParams) -> Self {
        match new_item.type_field {
            MessageItemType::Text => Ok(MessageItem::new_text(
                &new_item.content,
                new_item.timestamp,
                new_item.is_private,
            )),
            MessageItemType::File => {
                let file_name = match new_item.file_name.clone() {
                    Some(file_name) => file_name,
                    None => {
                        return Err(FieldParseError(
                            "MessageItem field fileName missed for file type".to_string(),
                        ));
                    }
                };

                let is_complete = match new_item.is_complete {
                    Some(is_complete) => is_complete,
                    None => {
                        return Err(FieldParseError(
                            "MessageItem field isComplete missed for file type".to_string(),
                        ));
                    }
                };

                Ok(MessageItem::new_file(
                    &new_item.content,
                    new_item.timestamp,
                    new_item.is_private,
                    &file_name,
                    is_complete,
                ))
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct NewItemResponse {
    pub id: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RemoveItemParams {
    pub id: u64,
    #[serde(rename = "type")]
    pub type_field: MessageItemType,
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    pub sid: Sid,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RemoveAllParams {
    pub sid: Sid,
}
