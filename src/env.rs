/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use pico_args::Arguments;
use std::fmt::Display;
use std::str::FromStr;

use crate::error::Error::{self, DefaultError};
use crate::error::Result;

fn get_arg_value<T>(arg_name: &'static str) -> Result<T>
where
    T: FromStr,
    T::Err: Display,
{
    let mut args = Arguments::from_env();
    let value: T = args
        .value_from_str(arg_name)
        .map_err(|e| DefaultError(format!("failed to get arg {}: {}", arg_name, e)))?;

    Ok(value)
}

fn get_arg_value_option<T>(arg_name: &'static str, default: T) -> T
where
    T: FromStr,
    T::Err: Display,
{
    let mut args = Arguments::from_env();
    match args.value_from_str(arg_name) {
        Ok(value) => return value,
        Err(_) => return default,
    };
}

fn args_contains(arg_name: &'static str) -> bool {
    let mut args = Arguments::from_env();
    args.contains(arg_name)
}

#[derive(Debug, Clone)]
pub enum EnvMode {
    Pro,
    Dev,
}

impl EnvMode {
    pub fn tracing_level(&self) -> String {
        match self {
            EnvMode::Pro => "info".to_string(),
            EnvMode::Dev => "debug".to_string(),
        }
    }
}

impl FromStr for EnvMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s == "pro" {
            Ok(Self::Pro)
        } else if s == "dev" {
            Ok(Self::Dev)
        } else {
            Err(DefaultError(
                "EnvMode must be one of 'pro' or 'dev'".to_string(),
            ))
        }
    }
}

impl Display for EnvMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pro => write!(f, "pro"),
            Self::Dev => write!(f, "dev"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum DatabaseEnv {
    MySql(MySqlEnv),
    Sqlite(SqliteEnv),
}

impl DatabaseEnv {
    fn new() -> Result<Self> {
        if args_contains("--mysql") {
            Ok(Self::MySql(MySqlEnv::new()?))
        } else if args_contains("--sqlite") {
            Ok(Self::Sqlite(SqliteEnv::new()?))
        } else {
            Err(DefaultError("no database specified".to_string()))
        }
    }
}

#[derive(Debug, Clone)]
pub struct MySqlEnv {
    pub endpoint: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

impl MySqlEnv {
    fn new() -> Result<Self> {
        let endpoint = get_arg_value::<String>("--mysql-endpoint")?;
        let username = get_arg_value::<String>("--mysql-username")?;
        let password = get_arg_value::<String>("--mysql-password")?;
        let database = get_arg_value::<String>("--mysql-database")?;

        Ok(Self {
            endpoint,
            username,
            password,
            database,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SqliteEnv {
    pub path: String,
}

impl SqliteEnv {
    fn new() -> Result<Self> {
        Ok(Self {
            path: "./data/db.sqlite".to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Env {
    pub mode: EnvMode,
    pub port: u16,
    pub item_per_page: i64,
    pub username: String,
    pub password: String,
    pub minio_endpoint: String,
    pub minio_username: String,
    pub minio_password: String,
    pub minio_bucket: String,
    pub database: DatabaseEnv,
}

impl Env {
    pub fn new() -> Self {
        let mode = get_arg_value_option("--mode", EnvMode::Pro);
        let port = get_arg_value_option("--port", 8080);
        let item_per_page = get_arg_value_option("--item-per-page", 15);
        let username = get_arg_value::<String>("--username").unwrap();
        let password = get_arg_value::<String>("--password").unwrap();
        let minio_endpoint = get_arg_value::<String>("--minio-endpoint").unwrap();
        let minio_username = get_arg_value::<String>("--minio-username").unwrap();
        let minio_password = get_arg_value::<String>("--minio-password").unwrap();
        let minio_bucket = get_arg_value::<String>("--minio-bucket").unwrap();
        let database = DatabaseEnv::new().unwrap();

        Self {
            mode,
            port,
            item_per_page,
            username,
            password,
            minio_endpoint,
            minio_username,
            minio_password,
            minio_bucket,
            database,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::env;

    pub enum DBType {
        MySql,
        Sqlite,
    }

    fn get_env_value<T>(key: &str) -> Result<T>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        let value =
            env::var(key).map_err(|e| DefaultError(format!("failed to get env {}: {}", key, e)))?;

        value
            .parse::<T>()
            .map_err(|e| DefaultError(format!("failed to parse env {}: {}", key, e)))
    }

    impl DatabaseEnv {
        fn new_mysql() -> Result<Self> {
            Ok(Self::MySql(MySqlEnv::new_test()?))
        }

        fn new_sqlite() -> Result<Self> {
            Ok(Self::Sqlite(SqliteEnv::new_test()?))
        }
    }

    impl MySqlEnv {
        fn new_test() -> Result<Self> {
            let endpoint = get_env_value("MYSQL_ENDPOINT")?;
            let username = get_env_value("MYSQL_USERNAME")?;
            let password = get_env_value("MYSQL_PASSWORD")?;
            let database = get_env_value("MYSQL_DATABASE")?;

            Ok(Self {
                endpoint,
                username,
                password,
                database,
            })
        }
    }

    impl SqliteEnv {
        fn new_test() -> Result<Self> {
            Ok(Self {
                path: "./dev.sqlite".to_string(),
            })
        }
    }

    pub fn get_env(db_type: DBType) -> Env {
        dotenv().ok();

        let mode: EnvMode = EnvMode::Dev;
        let port = env::var("PORT")
            .unwrap_or("8080".to_string())
            .parse()
            .unwrap();
        let item_per_page = env::var("ITEM_PER_PAGE")
            .unwrap_or("15".to_string())
            .parse()
            .unwrap();
        let username = env::var("USERNAME").unwrap();
        let password = env::var("PASSWORD").unwrap();
        let minio_endpoint = env::var("MINIO_ENDPOINT").unwrap();
        let minio_username = env::var("MINIO_USERNAME").unwrap();
        let minio_password = env::var("MINIO_PASSWORD").unwrap();
        let minio_bucket = env::var("MINIO_BUCKET").unwrap();
        let database = match db_type {
            DBType::MySql => DatabaseEnv::new_mysql().unwrap(),
            DBType::Sqlite => DatabaseEnv::new_sqlite().unwrap(),
        };

        Env {
            mode,
            port,
            item_per_page,
            username,
            password,
            minio_endpoint,
            minio_username,
            minio_password,
            minio_bucket,
            database,
        }
    }
}
