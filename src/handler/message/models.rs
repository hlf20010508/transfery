/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use serde::{Deserialize, Serialize};
use socketioxide::socket::Sid;

use crate::client::database::models::message::{MessageItem, MessageItemType, Model};
use crate::error::Error;
use crate::error::ErrorType::InternalServerError;
use crate::error::Result;

impl From<(i64, NewItemParams)> for Model {
    fn from(
        (
            id,
            NewItemParams {
                content,
                timestamp,
                is_private,
                file_name,
                is_complete,
                type_field,
                ..
            },
        ): (i64, NewItemParams),
    ) -> Self {
        Self {
            id,
            content,
            timestamp,
            is_private,
            file_name,
            is_complete,
            type_field,
        }
    }
}

#[derive(Deserialize)]
pub struct PageQueryParams {
    pub size: i64,
}

#[derive(Deserialize)]
pub struct SyncQueryParams {
    #[serde(rename = "latestId")]
    pub latest_id: i64,
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
                        return Err(Error::new(
                            InternalServerError,
                            "missed field fileName for file type in MessageItem",
                        ));
                    }
                };

                let is_complete = match new_item.is_complete {
                    Some(is_complete) => is_complete,
                    None => {
                        return Err(Error::new(
                            InternalServerError,
                            "missed field isComplete for file type in MessageItem",
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
    pub id: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RemoveItemParams {
    pub id: i64,
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
