use crate::adapters::config::ConfigLoader;
use crate::adapters::metadata_adapter_for;
use crate::domain::{self, ConnectionConfig, DatabaseEngine};
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
    let engine = config.engine;
    let adapter = metadata_adapter_for(engine);

    match adapter.test_connection(&config) {
        Ok(()) => match adapter.list_objects(&config) {
            Ok(mut objects) => {
                domain::sort_objects(&mut objects);
                ConfigFileConnectionOutcome::Loaded {
                path: path.clone(),
                engine,
                config,
                catalog: Catalog {
                    objects,
                    tables: Vec::new(),
                    mode: CatalogMode::Real,
                },
                message: format!(
                    "Configuracion cargada desde `{}`. Conexion real a {} establecida.",
                    path, engine
                ),
            }},
            Err(error) => ConfigFileConnectionOutcome::Error {
                path: Some(path),
                message: format!(
                    "La conexion a {} funciono, pero no se pudieron listar objetos: {}",
                    engine, error
                ),
            },
        },
        Err(error) => ConfigFileConnectionOutcome::Error {
            path: Some(path.clone()),
            message: format!("No se pudo conectar a {} con `{}`: {}", engine, path, error),
        },
    }
}
