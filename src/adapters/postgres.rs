use crate::core::error::AppError;
use crate::core::ports::{ConnectionProvider, MetadataExplorer, SqlExecutor};
use crate::domain::{ConnectionConfig, DatabaseObject, TableSchema};

#[derive(Debug, Default, Clone, Copy)]
pub struct PostgresAdapter;

impl ConnectionProvider for PostgresAdapter {
    fn test_connection(&self, _config: &ConnectionConfig) -> Result<(), AppError> {
        Err(AppError::NotImplemented("conexion PostgreSQL"))
    }
}

impl MetadataExplorer for PostgresAdapter {
    fn list_objects(&self, _config: &ConnectionConfig) -> Result<Vec<DatabaseObject>, AppError> {
        Err(AppError::NotImplemented("metadata PostgreSQL"))
    }

    fn get_table_schema(
        &self,
        _config: &ConnectionConfig,
        _schema: &str,
        _table: &str,
    ) -> Result<TableSchema, AppError> {
        Err(AppError::NotImplemented("lectura de schema PostgreSQL"))
    }
}

impl SqlExecutor for PostgresAdapter {
    fn execute_sql(&self, _config: &ConnectionConfig, _sql: &str) -> Result<(), AppError> {
        Err(AppError::NotImplemented("ejecucion SQL PostgreSQL"))
    }
}
