pub mod clipboard;
pub mod config;
pub mod postgres;
pub mod process;
pub mod sqlserver;

use crate::core::error::AppError;
use crate::domain::DatabaseEngine;

/// Resolve el adapter de base de datos adecuado para un motor.
///
/// Centraliza la seleccion de adapter para que la UI y los casos de uso
/// no conozcan el adapter concreto, evitando que se filtre `SqlServerAdapter`
/// cuando el motor es `PostgreSql`.
pub fn metadata_adapter_for(engine: DatabaseEngine) -> Box<dyn MetadataAdapter> {
    match engine {
        DatabaseEngine::PostgreSql => Box::new(postgres::PostgresAdapter),
        DatabaseEngine::SqlServer => Box::new(sqlserver::SqlServerAdapter),
    }
}

/// Trait unificado que agrupa los puertos de conexion, introspeccion y ejecucion
/// que la UI necesita de un adapter de base de datos.
pub trait MetadataAdapter:
    crate::core::ports::ConnectionProvider
    + crate::core::ports::MetadataExplorer
    + crate::core::ports::SqlExecutor
{
}

impl MetadataAdapter for postgres::PostgresAdapter {}
impl MetadataAdapter for sqlserver::SqlServerAdapter {}

/// Atajo para errores de motor no soportado en casos de uso que reciban un engine
/// todavia no resuelto a adapter concreto.
pub fn unsupported(engine: DatabaseEngine) -> AppError {
    AppError::UnsupportedDatabaseEngine(engine.to_string())
}