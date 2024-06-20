/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, EntityName, EntityTrait, PaginatorTrait,
    Schema, Set, Statement,
};
use tokio::fs;

use super::models::config::Config;
use super::Database;
use crate::client::database::models::{auth, device, message, token};
use crate::crypto::Crypto;
use crate::error::Error::{DatabaseClientError, DefaultError, SqlExecuteError, SqlQueryError};
use crate::error::Result;

impl Database {
    pub async fn new(config: Config) -> Result<Self> {
        match config {
            Config::MySql(config) => {
                let connection = sea_orm::Database::connect(format!(
                    "mysql://{}:{}@{}",
                    config.username, config.password, config.endpoint
                ))
                .await
                .map_err(|e| {
                    DatabaseClientError(format!("failed to connect to MySql partial: {}", e))
                })?;

                Database::create_database_if_not_exists(&connection, &config.name).await?;

                let connection = sea_orm::Database::connect(format!(
                    "mysql://{}:{}@{}/{}",
                    config.username, config.password, config.endpoint, config.name
                ))
                .await
                .map_err(|e| DatabaseClientError(format!("failed to connect to MySql: {}", e)))?;

                Ok(Self {
                    connection,
                    _name: config.name,
                })
            }
            Config::Sqlite(config) => {
                let connection =
                    sea_orm::Database::connect(format!("sqlite://{}?mode=rwc", config.path))
                        .await
                        .map_err(|e| {
                            DatabaseClientError(format!("failed to connect to Sqlite: {}", e))
                        })?;

                Ok(Self {
                    connection,
                    _name: config.path,
                })
            }
        }
    }

    pub async fn _close(self) -> Result<()> {
        self.connection.close().await.map_err(|e| {
            DatabaseClientError(format!("failed to close database connection: {}", e))
        })?;

        Ok(())
    }

    pub async fn create_database_if_not_exists(
        connection: &DatabaseConnection,
        name: &str,
    ) -> Result<()> {
        if connection.get_database_backend() == DbBackend::MySql {
            let sql = format!("create database if not exists `{}`", name);

            connection
                .execute(Statement::from_string(
                    connection.get_database_backend(),
                    sql,
                ))
                .await
                .map_err(|e| SqlExecuteError(format!("failed to create database: {}", e)))?;
        }

        Ok(())
    }

    pub async fn init(&self) -> Result<()> {
        self.create_table_message_if_not_exists().await?;
        self.create_table_auth_if_not_exists().await?;
        self.create_table_device_if_not_exists().await?;
        self.create_table_token_if_not_exists().await?;
        self.create_secret_key_if_not_exists().await?;

        Ok(())
    }

    async fn is_table_exists<E>(&self) -> bool
    where
        E: EntityTrait,
    {
        let result = E::find().all(&self.connection).await;

        if result.is_ok() {
            true
        } else {
            false
        }
    }

    async fn create_table_if_not_exists<E>(&self, entity: E) -> Result<()>
    where
        E: EntityTrait + EntityName,
    {
        if !self.is_table_exists::<E>().await {
            let backend = self.connection.get_database_backend();

            let table_create_statement = Schema::new(backend).create_table_from_entity(entity);

            self.connection
                .execute(backend.build(&table_create_statement))
                .await
                .map_err(|e| {
                    SqlExecuteError(format!(
                        "failed to create table {}: {}",
                        entity.table_name(),
                        e
                    ))
                })?;
        }

        Ok(())
    }

    pub async fn create_table_message_if_not_exists(&self) -> Result<()> {
        self.create_table_if_not_exists(message::Entity).await?;

        Ok(())
    }

    pub async fn create_table_auth_if_not_exists(&self) -> Result<()> {
        self.create_table_if_not_exists(auth::Entity).await?;

        Ok(())
    }

    pub async fn create_table_device_if_not_exists(&self) -> Result<()> {
        self.create_table_if_not_exists(device::Entity).await?;

        Ok(())
    }

    pub async fn create_table_token_if_not_exists(&self) -> Result<()> {
        self.create_table_if_not_exists(token::Entity).await?;

        Ok(())
    }

    pub async fn create_secret_key_if_not_exists(&self) -> Result<()> {
        if !self.is_secret_key_exist().await? {
            let secret_key = Crypto::gen_secret_key()?;

            let insert_item = auth::ActiveModel {
                secret_key: Set(secret_key.clone()),
                ..Default::default()
            };

            auth::Entity::insert(insert_item)
                .exec(&self.connection)
                .await
                .map_err(|e| SqlExecuteError(format!("failed to create secret key: {}", e)))?;
        }

        Ok(())
    }

    pub async fn is_secret_key_exist(&self) -> Result<bool> {
        let count = auth::Entity::find()
            .count(&self.connection)
            .await
            .map_err(|e| SqlQueryError(format!("failed to count secret key: {}", e)))?;

        Ok(count > 0)
    }

    pub async fn get_secret_key(&self) -> Result<String> {
        let auth::Model { secret_key, .. } = auth::Entity::find()
            .one(&self.connection)
            .await
            .map_err(|e| SqlQueryError(format!("failed to get secret key: {}", e)))?
            .ok_or_else(|| SqlQueryError("secret key not found".to_string()))?;

        Ok(secret_key)
    }

    pub async fn _drop_database_if_exists(self) -> Result<()> {
        match self.connection.get_database_backend() {
            DbBackend::MySql => {
                let sql = format!("drop database if exists `{}`", self._name);

                self.connection
                    .execute(Statement::from_string(
                        self.connection.get_database_backend(),
                        sql,
                    ))
                    .await
                    .map_err(|e| SqlExecuteError(format!("failed to drop database: {}", e)))?;

                self._close().await?;
            }
            DbBackend::Sqlite => {
                if fs::metadata(&self._name).await.is_ok() {
                    let path = self._name.clone();

                    self._close().await?;

                    fs::remove_file(&path).await.map_err(|e| {
                        DefaultError(format!("failed to remove sqlite file: {}", e))
                    })?;
                }
            }
            _ => {
                return Err(DefaultError("unsupported database backend".to_string()));
            }
        }

        Ok(())
    }
}
