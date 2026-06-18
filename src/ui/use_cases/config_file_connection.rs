use crate::adapters::config::ConfigLoader;
use crate::adapters::sqlserver::SqlServerAdapter;
use crate::core::ports::{ConnectionProvider, MetadataExplorer};
use crate::domain::{ConnectionConfig, DatabaseEngine};
use crate::ui::mocks::catalog_fn;
use crate::ui::state::{Catalog, CatalogMode};

pub(crate) enum ConfigFileConnectionOutcome {
    CreatedTemplate {
        path: String,
        message: String,
    },
    Loaded {
        path: String,
        config: ConnectionConfig,
        engine: DatabaseEngine,
        catalog: Catalog,
        message: String,
    },
    Error {
        path: Option<String>,
        message: String,
    },
}

pub(crate) fn load_or_create() -> ConfigFileConnectionOutcome {
    let loader = ConfigLoader;

    match loader.ensure_default_file(None) {
        Ok(result) if result.created => {
            let path = result.path.display().to_string();
            ConfigFileConnectionOutcome::CreatedTemplate {
                message: format!(
                    "Se genero `{}`. Editalo manualmente y presiona Enter otra vez para cargarlo.",
                    path
                ),
                path,
            }
        }
        Ok(result) => {
            let path = result.path.display().to_string();

            match loader.load_from_json_file(result.path.as_path()) {
                Ok(config) => activate_connection(config, path),
                Err(error) => ConfigFileConnectionOutcome::Error {
                    path: Some(path.clone()),
                    message: format!(
                        "No se pudo cargar `{}`: {}. Edita el archivo y presiona Enter otra vez.",
                        path, error
                    ),
                },
            }
        }
        Err(error) => ConfigFileConnectionOutcome::Error {
            path: None,
            message: format!(
                "No fue posible preparar el archivo de configuracion: {}",
                error
            ),
        },
    }
}

fn activate_connection(config: ConnectionConfig, path: String) -> ConfigFileConnectionOutcome {
    match config.engine {
        DatabaseEngine::SqlServer => {
            let adapter = SqlServerAdapter;
            match adapter.test_connection(&config) {
                Ok(()) => match adapter.list_objects(&config) {
                    Ok(objects) => ConfigFileConnectionOutcome::Loaded {
                        path: path.clone(),
                        engine: config.engine,
                        config,
                        catalog: Catalog {
                            objects,
                            tables: Vec::new(),
                            mode: CatalogMode::Real,
                        },
                        message: format!(
                            "Configuracion cargada desde `{}`. Conexion real a SQL Server establecida.",
                            path
                        ),
                    },
                    Err(error) => ConfigFileConnectionOutcome::Error {
                        path: Some(path),
                        message: format!(
                            "La conexion a SQL Server funciono, pero no se pudieron listar objetos: {}",
                            error
                        ),
                    },
                },
                Err(error) => ConfigFileConnectionOutcome::Error {
                    path: Some(path.clone()),
                    message: format!("No se pudo conectar a SQL Server con `{}`: {}", path, error),
                },
            }
        }
        DatabaseEngine::PostgreSql => ConfigFileConnectionOutcome::Loaded {
            path: path.clone(),
            engine: config.engine,
            config,
            catalog: catalog_fn(DatabaseEngine::PostgreSql),
            message: format!(
                "Configuracion cargada desde `{}`. PostgreSQL aun usa catalogo mock mientras conectamos su adapter real.",
                path
            ),
        },
    }
}
