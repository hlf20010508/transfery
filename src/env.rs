/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use pico_args::Arguments;
use std::fmt::Display;
use std::str::FromStr;

use crate::error::Error;
use crate::error::ErrorType::InternalServerError;
use crate::error::Result;

fn get_arg_value<T>(arg_name: &'static str) -> Result<T>
where
    T: FromStr,
    T::Err: Display,
{
    let mut args = Arguments::from_env();
    let value: T = args
        .value_from_str(arg_name)
        .map_err(|e| Error::context(InternalServerError, e, "failed to get arg"))?;

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

pub fn args_contains(arg_name: &'static str) -> bool {
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
            Err(Error::new(
                InternalServerError,
                "EnvMode must be one of pro or dev",
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
        } else {
            Ok(Self::Sqlite(SqliteEnv::new()?))
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
            path: "./db.sqlite".to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub enum StorageEnv {
    Minio(MinioEnv),
    LocalStorage(LocalStorageEnv),
}

impl StorageEnv {
    fn new() -> Result<Self> {
        if args_contains("--minio") {
            Ok(Self::Minio(MinioEnv::new()?))
        } else {
            Ok(Self::LocalStorage(LocalStorageEnv::new()?))
        }
    }
}

#[derive(Debug, Clone)]
pub struct MinioEnv {
    pub endpoint: String,
    pub username: String,
    pub password: String,
    pub bucket: String,
}

impl MinioEnv {
    fn new() -> Result<Self> {
        let endpoint = get_arg_value::<String>("--minio-endpoint")?;
        let username = get_arg_value::<String>("--minio-username")?;
        let password = get_arg_value::<String>("--minio-password")?;
        let bucket = get_arg_value::<String>("--minio-bucket")?;

        Ok(Self {
            endpoint,
            username,
            password,
            bucket,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LocalStorageEnv {
    pub path: String,
}

impl LocalStorageEnv {
    fn new() -> Result<Self> {
        Ok(Self {
            path: "./uploaded".to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Env {
    pub mode: EnvMode,
    pub port: u16,
    pub item_per_page: u64,
    pub socketio_ping_interval: u64,
    pub username: String,
    pub password: String,
    pub storage: StorageEnv,
    pub database: DatabaseEnv,
}

impl Env {
    pub fn new() -> Self {
        let mode = get_arg_value_option("--mode", EnvMode::Pro);
        let port = get_arg_value_option("--port", 8080);
        let item_per_page = get_arg_value_option("--item-per-page", 15);
        let socketio_ping_interval = get_arg_value_option("--socketio-ping-interval", 25);
        let username = get_arg_value::<String>("--username").unwrap();
        let password = get_arg_value::<String>("--password").unwrap();
        let storage = StorageEnv::new().unwrap();
        let database = DatabaseEnv::new().unwrap();

        Self {
            mode,
            port,
            item_per_page,
            socketio_ping_interval,
            username,
            password,
            storage,
            database,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use dotenv::dotenv;
    use std::env;
    use strum::EnumIter;

    #[derive(EnumIter)]
    pub enum DBType {
        MySql,
        Sqlite,
    }

    #[derive(EnumIter)]
    pub enum STType {
        Minio,
        LocalStorage,
    }

    fn get_env_value<T>(key: &str) -> Result<T>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        let value = env::var(key)
            .map_err(|e| Error::context(InternalServerError, e, "failed to get env"))?;

        value
            .parse::<T>()
            .map_err(|e| Error::context(InternalServerError, e, "failed to parse env"))
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

    impl StorageEnv {
        fn new_minio() -> Result<Self> {
            Ok(Self::Minio(MinioEnv::new_test()?))
        }

        fn new_local_storage() -> Result<Self> {
            Ok(Self::LocalStorage(LocalStorageEnv::new_test()?))
        }
    }

    impl MinioEnv {
        fn new_test() -> Result<Self> {
            let endpoint = get_env_value("MINIO_ENDPOINT")?;
            let username = get_env_value("MINIO_USERNAME")?;
            let password = get_env_value("MINIO_PASSWORD")?;
            let bucket = get_env_value("MINIO_BUCKET")?;

            Ok(Self {
                endpoint,
                username,
                password,
                bucket,
            })
        }
    }

    impl LocalStorageEnv {
        fn new_test() -> Result<Self> {
            Ok(Self {
                path: "./dev.uploaded".to_string(),
            })
        }
    }

    pub fn get_env(db_type: DBType, st_type: STType) -> Env {
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
        let socketio_ping_interval = env::var("SOCKETIO_PING_INTERVAL")
            .unwrap_or("25".to_string())
            .parse()
            .unwrap();
        let username = env::var("USERNAME").unwrap();
        let password = env::var("PASSWORD").unwrap();
        let storage = match st_type {
            STType::Minio => StorageEnv::new_minio().unwrap(),
            STType::LocalStorage => StorageEnv::new_local_storage().unwrap(),
        };
        let database = match db_type {
            DBType::MySql => DatabaseEnv::new_mysql().unwrap(),
            DBType::Sqlite => DatabaseEnv::new_sqlite().unwrap(),
        };

        Env {
            mode,
            port,
            item_per_page,
            socketio_ping_interval,
            username,
            password,
            storage,
            database,
        }
    }
}
