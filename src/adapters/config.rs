use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::error::AppError;
use crate::domain::{ConnectionConfig, DatabaseEngine, GeneratorConfig};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnsureConfigFileResult {
    pub path: PathBuf,
    pub created: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ConfigLoader;

impl ConfigLoader {
    pub fn default_path(&self) -> Result<PathBuf, AppError> {
        let current_dir = env::current_dir().map_err(|error| {
            AppError::InvalidConfiguration(format!(
                "no fue posible obtener el directorio actual: {error}"
            ))
        })?;

        Ok(current_dir.join("api-from-db-cli.config.json"))
    }

    pub fn ensure_default_file(
        &self,
        path: Option<&Path>,
    ) -> Result<EnsureConfigFileResult, AppError> {
        let path = match path {
            Some(path) => path.to_path_buf(),
            None => self.default_path()?,
        };

        if path.exists() {
            return Ok(EnsureConfigFileResult {
                path,
                created: false,
            });
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                AppError::InvalidConfiguration(format!(
                    "no fue posible crear el directorio de configuracion: {error}"
                ))
            })?;
        }

        let template = ConfigTemplate::default();
        let raw = serde_json::to_string_pretty(&template).map_err(|error| {
            AppError::InvalidConfiguration(format!(
                "no fue posible serializar la plantilla de configuracion: {error}"
            ))
        })?;

        fs::write(&path, raw).map_err(|error| {
            AppError::InvalidConfiguration(format!(
                "no fue posible escribir el archivo de configuracion: {error}"
            ))
        })?;

        Ok(EnsureConfigFileResult {
            path,
            created: true,
        })
    }

    pub fn load_from_json_file(&self, path: &Path) -> Result<ConnectionConfig, AppError> {
        let raw = fs::read_to_string(path).map_err(|error| {
            AppError::InvalidConfiguration(format!(
                "no fue posible leer el archivo de configuracion: {error}"
            ))
        })?;

        let mut file: ConfigTemplate = serde_json::from_str(&raw).map_err(|error| {
            AppError::InvalidConfiguration(format!(
                "el archivo de configuracion no es JSON valido: {error}"
            ))
        })?;

        let engine = match file.engine.trim().to_ascii_lowercase().as_str() {
            "postgresql" => DatabaseEngine::PostgreSql,
            "sqlserver" => DatabaseEngine::SqlServer,
            other => return Err(AppError::UnsupportedDatabaseEngine(other.to_string())),
        };

        validate_template(&mut file)?;

        Ok(ConnectionConfig {
            id: file.id.unwrap_or_else(|| "from-file".to_string()),
            name: file.name.unwrap_or_else(|| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("config-file")
                    .to_string()
            }),
            engine,
            host: file.host,
            port: file.port,
            database: file.database,
            username: file.username,
            password: file.password,
            connection_string: file.connection_string,
            config_file_path: Some(path.display().to_string()),
            generator: GeneratorConfig {
                cmd: file.r#gen.cmd,
                flags: file.r#gen.flags,
            },
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfigTemplate {
    id: Option<String>,
    name: Option<String>,
    engine: String,
    host: Option<String>,
    port: Option<u16>,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
    #[serde(alias = "connectionString")]
    connection_string: Option<String>,
    #[serde(default)]
    #[serde(rename = "gen")]
    r#gen: GeneratorConfigTemplate,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeneratorConfigTemplate {
    cmd: String,
    #[serde(default)]
    flags: Vec<String>,
    flag: Option<String>,
}

impl Default for GeneratorConfigTemplate {
    fn default() -> Self {
        Self {
            cmd: "gen.exe".to_string(),
            flags: vec!["--mvc".to_string()],
            flag: None,
        }
    }
}

impl Default for ConfigTemplate {
    fn default() -> Self {
        Self {
            id: Some("local-sqlserver".to_string()),
            name: Some("SQL Server local".to_string()),
            engine: "sqlserver".to_string(),
            host: Some("localhost".to_string()),
            port: Some(1433),
            database: Some("master".to_string()),
            username: Some("sa".to_string()),
            password: Some("cambia_este_valor".to_string()),
            connection_string: None,
            r#gen: GeneratorConfigTemplate::default(),
        }
    }
}

fn validate_template(file: &mut ConfigTemplate) -> Result<(), AppError> {
    normalize_generator_flags(&mut file.r#gen);
    require_string(Some(file.r#gen.cmd.as_str()), "gen.cmd")?;

    if file
        .connection_string
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
    {
        return Ok(());
    }

    require_string(file.host.as_deref(), "host")?;
    require_port(file.port, "port")?;
    require_string(file.database.as_deref(), "database")?;
    require_string(file.username.as_deref(), "username")?;
    require_string(file.password.as_deref(), "password")?;

    Ok(())
}

fn normalize_generator_flags(generator: &mut GeneratorConfigTemplate) {
    if let Some(flag) = generator.flag.take() {
        if !flag.trim().is_empty() {
            generator.flags.push(flag);
        }
    }

    generator.flags = generator
        .flags
        .drain(..)
        .map(|flag| flag.trim().to_string())
        .filter(|flag| !flag.is_empty())
        .collect();
}

fn require_string(value: Option<&str>, field: &str) -> Result<(), AppError> {
    match value.map(str::trim) {
        Some("") | None => Err(AppError::MissingRequiredField(field.to_string())),
        Some(_) => Ok(()),
    }
}

fn require_port(value: Option<u16>, field: &str) -> Result<(), AppError> {
    match value {
        Some(0) | None => Err(AppError::MissingRequiredField(field.to_string())),
        Some(_) => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::ConfigLoader;
    use crate::core::error::AppError;
    use crate::domain::DatabaseEngine;

    #[test]
    fn creates_template_when_file_does_not_exist() {
        let loader = ConfigLoader;
        let path = temp_config_path("create");
        let result = loader.ensure_default_file(Some(path.as_path())).unwrap();

        assert!(result.created);
        assert!(path.exists());

        let content = fs::read_to_string(path.as_path()).unwrap();
        assert!(content.contains("sqlserver"));
        assert!(content.contains("\"gen\""));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn loads_valid_config_file() {
        let loader = ConfigLoader;
        let path = temp_config_path("load");
        loader.ensure_default_file(Some(path.as_path())).unwrap();

        let config = loader.load_from_json_file(path.as_path()).unwrap();
        assert_eq!(config.engine, DatabaseEngine::SqlServer);
        assert_eq!(config.host.as_deref(), Some("localhost"));
        assert_eq!(config.generator.cmd, "gen.exe");
        assert_eq!(config.generator.flags, vec!["--mvc".to_string()]);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn rejects_invalid_config_file() {
        let loader = ConfigLoader;
        let path = temp_config_path("invalid");
        fs::write(&path, "{\"engine\":\"sqlserver\",\"host\":\"localhost\"}").unwrap();

        let result = loader.load_from_json_file(path.as_path());
        assert!(matches!(result, Err(AppError::MissingRequiredField(field)) if field == "port"));

        let _ = fs::remove_file(path);
    }

    fn temp_config_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("api-from-db-cli-{label}-{nanos}.json"))
    }
}
