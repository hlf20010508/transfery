/*
:project: transfery
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

pub struct MySqlConfig {
    pub endpoint: String,
    pub username: String,
    pub password: String,
    pub name: String,
}

impl MySqlConfig {
    pub fn new(endpoint: &str, username: &str, password: &str, name: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            name: name.to_string(),
        }
    }
}

pub struct SqliteConfig {
    pub path: String,
}

impl SqliteConfig {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}

pub enum Config {
    MySql(MySqlConfig),
    Sqlite(SqliteConfig),
}
