use crate::core::error::AppError;
use crate::domain::{
    ConnectionConfig, DatabaseObject, GeneratedCommand, ProcessResult, TableSchema,
};

pub trait ConnectionProvider {
    fn test_connection(&self, config: &ConnectionConfig) -> Result<(), AppError>;
}

pub trait MetadataExplorer {
    fn list_objects(&self, config: &ConnectionConfig) -> Result<Vec<DatabaseObject>, AppError>;
    fn get_table_schema(
        &self,
        config: &ConnectionConfig,
        schema: &str,
        table: &str,
    ) -> Result<TableSchema, AppError>;
}

pub trait SqlExecutor {
    fn execute_sql(&self, config: &ConnectionConfig, sql: &str) -> Result<(), AppError>;
}

pub trait ProcessRunner {
    fn run(&self, command: &GeneratedCommand) -> Result<ProcessResult, AppError>;
}

pub trait Clipboard {
    fn copy(&self, value: &str) -> Result<(), AppError>;
}
