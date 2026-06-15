use std::fs;

use crate::core::error::AppError;
use crate::domain::{ConnectionConfig, DatabaseEngine};

#[derive(Debug, Default, Clone, Copy)]
pub struct ConfigLoader;

impl ConfigLoader {
    pub fn load_from_json_stub(&self, path: &str) -> Result<ConnectionConfig, AppError> {
        let raw = fs::read_to_string(path)
            .map_err(|error| AppError::InvalidConfiguration(error.to_string()))?;

        let engine = if raw.contains("\"engine\":\"postgresql\"")
            || raw.contains("\"engine\": \"postgresql\"")
        {
            DatabaseEngine::PostgreSql
        } else if raw.contains("\"engine\":\"sqlserver\"")
            || raw.contains("\"engine\": \"sqlserver\"")
        {
            DatabaseEngine::SqlServer
        } else {
            return Err(AppError::UnsupportedDatabaseEngine(
                "no fue posible inferir el motor desde el archivo".to_string(),
            ));
        };

        Ok(ConnectionConfig {
            id: "from-file".to_string(),
            name: path.to_string(),
            engine,
            host: None,
            port: None,
            database: None,
            username: None,
            password: None,
            connection_string: None,
            config_file_path: Some(path.to_string()),
        })
    }
}
