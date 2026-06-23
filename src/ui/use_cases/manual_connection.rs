use crate::adapters::metadata_adapter_for;
use crate::domain::{ConnectionConfig, DatabaseEngine, GeneratorConfig};
use crate::ui::state::{Catalog, CatalogMode};

pub(crate) struct ManualConnectionInput {
    pub(crate) engine: DatabaseEngine,
    pub(crate) host: String,
    pub(crate) port: String,
    pub(crate) database: String,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) generator_config: GeneratorConfig,
}

pub(crate) enum ManualConnectionOutcome {
    Connected {
        config: ConnectionConfig,
        catalog: Catalog,
        message: String,
    },
    ValidationError {
        message: String,
    },
    ConnectionError {
        message: String,
    },
}

pub(crate) fn connect(input: ManualConnectionInput) -> ManualConnectionOutcome {
    let host = input.host.trim().to_string();
    let database = input.database.trim().to_string();
    let username = input.username.trim().to_string();
    let password = input.password;

    if host.is_empty() {
        return ManualConnectionOutcome::ValidationError {
            message: "El campo Host es obligatorio.".to_string(),
        };
    }
    if database.is_empty() {
        return ManualConnectionOutcome::ValidationError {
            message: "El campo Database es obligatorio.".to_string(),
        };
    }
    if username.is_empty() {
        return ManualConnectionOutcome::ValidationError {
            message: "El campo Username es obligatorio.".to_string(),
        };
    }
    if password.is_empty() {
        return ManualConnectionOutcome::ValidationError {
            message: "El campo Password es obligatorio.".to_string(),
        };
    }

    let port = match input.port.trim() {
        "" => input.engine.default_port(),
        raw => match raw.parse::<u16>() {
            Ok(0) => {
                return ManualConnectionOutcome::ValidationError {
                    message: "El campo Port debe ser un numero mayor a 0.".to_string(),
                };
            }
            Ok(value) => value,
            Err(_) => {
                return ManualConnectionOutcome::ValidationError {
                    message: format!("El campo Port no es un numero valido: {raw}"),
                };
            }
        },
    };

    let config = ConnectionConfig {
        id: "manual".to_string(),
        name: format!("{} manual", input.engine),
        engine: input.engine,
        host: Some(host),
        port: Some(port),
        database: Some(database),
        username: Some(username),
        password: Some(password),
        connection_string: None,
        config_file_path: None,
        generator: input.generator_config,
    };

    let adapter = metadata_adapter_for(input.engine);

    match adapter.test_connection(&config) {
        Ok(()) => match adapter.list_objects(&config) {
            Ok(objects) => ManualConnectionOutcome::Connected {
                config,
                catalog: Catalog {
                    objects,
                    tables: Vec::new(),
                    mode: CatalogMode::Real,
                },
                message: format!(
                    "Conexion manual a {} establecida. Explora los objetos disponibles.",
                    input.engine
                ),
            },
            Err(error) => ManualConnectionOutcome::ConnectionError {
                message: format!(
                    "La conexion a {} funciono, pero no se pudieron listar objetos: {}",
                    input.engine, error
                ),
            },
        },
        Err(error) => ManualConnectionOutcome::ConnectionError {
            message: format!("No se pudo conectar a {}: {}", input.engine, error),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{ManualConnectionInput, ManualConnectionOutcome, connect};
    use crate::domain::{DatabaseEngine, GeneratorConfig};

    fn input(host: &str, port: &str, database: &str, username: &str, password: &str) -> ManualConnectionInput {
        ManualConnectionInput {
            engine: DatabaseEngine::PostgreSql,
            host: host.to_string(),
            port: port.to_string(),
            database: database.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            generator_config: GeneratorConfig::default(),
        }
    }

    #[test]
    fn rejects_empty_host() {
        let outcome = connect(input("", "5432", "db", "user", "pw"));
        assert!(matches!(outcome, ManualConnectionOutcome::ValidationError { message } if message.contains("Host")));
    }

    #[test]
    fn rejects_empty_database() {
        let outcome = connect(input("localhost", "5432", "", "user", "pw"));
        assert!(matches!(outcome, ManualConnectionOutcome::ValidationError { message } if message.contains("Database")));
    }

    #[test]
    fn rejects_empty_username() {
        let outcome = connect(input("localhost", "5432", "db", "", "pw"));
        assert!(matches!(outcome, ManualConnectionOutcome::ValidationError { message } if message.contains("Username")));
    }

    #[test]
    fn rejects_empty_password() {
        let outcome = connect(input("localhost", "5432", "db", "user", ""));
        assert!(matches!(outcome, ManualConnectionOutcome::ValidationError { message } if message.contains("Password")));
    }

    #[test]
    fn rejects_non_numeric_port() {
        let outcome = connect(input("localhost", "abc", "db", "user", "pw"));
        assert!(matches!(outcome, ManualConnectionOutcome::ValidationError { message } if message.contains("Port")));
    }

    #[test]
    fn rejects_zero_port() {
        let outcome = connect(input("localhost", "0", "db", "user", "pw"));
        assert!(matches!(outcome, ManualConnectionOutcome::ValidationError { message } if message.contains("Port")));
    }
}