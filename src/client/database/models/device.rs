/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use sea_orm::entity::prelude::DeriveEntityModel;
use sea_orm::{
    ActiveModelBehavior, DerivePrimaryKey, DeriveRelation, EntityTrait, EnumIter, PrimaryKeyTrait,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, DeriveEntityModel, Serialize, PartialEq)]
#[sea_orm(table_name = "device")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub fingerprint: String,
    pub browser: String,
    #[sea_orm(rename = "lastUseTimestamp")]
    #[serde(rename = "lastUseTimestamp")]
    pub last_use_timestamp: i64,
    #[sea_orm(rename = "expirationTimestamp")]
    #[serde(rename = "expirationTimestamp")]
    pub expiration_timestamp: i64,
}

#[derive(Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DeviceItem {
    pub fingerprint: String,
    pub browser: String,
    #[serde(rename = "lastUseTimestamp")]
    pub last_use_timestamp: i64,
    #[serde(rename = "expirationTimestamp")]
    pub expiration_timestamp: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DeviceUpdateItem {
    pub fingerprint: String,
    pub browser: Option<String>,
    #[serde(rename = "lastUseTimestamp")]
    pub last_use_timestamp: Option<i64>,
    #[serde(rename = "expirationTimestamp")]
    pub expiration_timestamp: Option<i64>,
}
