use std::future::Future;

use tiberius::{AuthMethod, Client, Config};
use tokio::net::TcpStream;
use tokio::runtime::{Builder, Runtime};
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

use crate::core::error::AppError;
use crate::core::ports::{ConnectionProvider, MetadataExplorer, SqlExecutor};
use crate::domain::{
    ColumnSchema, ConnectionConfig, DatabaseEngine, DatabaseObject, DatabaseObjectType, TableSchema,
};

type SqlServerClient = Client<Compat<TcpStream>>;

#[derive(Debug, Default, Clone, Copy)]
pub struct SqlServerAdapter;

impl ConnectionProvider for SqlServerAdapter {
    fn test_connection(&self, config: &ConnectionConfig) -> Result<(), AppError> {
        self.with_runtime(async {
            let mut client = connect(config).await?;
            client
                .simple_query("SELECT 1 AS healthcheck")
                .await
                .map_err(|error| {
                    AppError::Database(format!("no fue posible validar la conexion: {error}"))
                })?
                .into_row()
                .await
                .map_err(|error| {
                    AppError::Database(format!(
                        "la prueba de conexion no devolvio resultado valido: {error}"
                    ))
                })?;
            Ok(())
        })
    }
}

impl MetadataExplorer for SqlServerAdapter {
    fn list_objects(&self, config: &ConnectionConfig) -> Result<Vec<DatabaseObject>, AppError> {
        self.with_runtime(async {
            let mut client = connect(config).await?;
            let rows = client
                .simple_query(LIST_OBJECTS_SQL)
                .await
                .map_err(|error| {
                    AppError::Database(format!("no fue posible listar objetos: {error}"))
                })?
                .into_first_result()
                .await
                .map_err(|error| {
                    AppError::Database(format!(
                        "no fue posible leer resultados de objetos: {error}"
                    ))
                })?;

            rows.into_iter().map(map_object_row).collect()
        })
    }

    fn get_table_schema(
        &self,
        config: &ConnectionConfig,
        schema: &str,
        table: &str,
    ) -> Result<TableSchema, AppError> {
        self.with_runtime(async {
            let mut client = connect(config).await?;
            let rows = client
                .query(TABLE_SCHEMA_SQL, &[&schema, &table])
                .await
                .map_err(|error| {
                    AppError::Database(format!(
                        "no fue posible leer la estructura de la tabla {schema}.{table}: {error}"
                    ))
                })?
                .into_first_result()
                .await
                .map_err(|error| {
                    AppError::Database(format!(
                        "no fue posible materializar el schema de la tabla {schema}.{table}: {error}"
                    ))
                })?;

            if rows.is_empty() {
                return Err(AppError::Database(format!(
                    "no se encontraron columnas para la tabla {schema}.{table}"
                )));
            }

            let mut columns = Vec::with_capacity(rows.len());
            let mut primary_keys = Vec::new();

            for row in rows {
                let column = map_column_row(row)?;
                if column.is_primary_key {
                    primary_keys.push(column.name.clone());
                }
                columns.push(column);
            }

            Ok(TableSchema::new(schema, table, columns, primary_keys))
        })
    }
}

impl SqlExecutor for SqlServerAdapter {
    fn execute_sql(&self, config: &ConnectionConfig, sql: &str) -> Result<(), AppError> {
        self.with_runtime(async {
            let mut client = connect(config).await?;
            client.execute(sql, &[]).await.map_err(|error| {
                AppError::Database(format!(
                    "no fue posible ejecutar SQL en SQL Server: {error}"
                ))
            })?;
            Ok(())
        })
    }
}

impl SqlServerAdapter {
    fn with_runtime<T, F>(&self, future: F) -> Result<T, AppError>
    where
        F: Future<Output = Result<T, AppError>>,
    {
        runtime()?.block_on(future)
    }
}

async fn connect(config: &ConnectionConfig) -> Result<SqlServerClient, AppError> {
    validate_sql_server_config(config)?;

    let tiberius_config = build_config(config)?;
    let address = tiberius_config.get_addr();
    let tcp = TcpStream::connect(address.as_str())
        .await
        .map_err(|error| {
            AppError::Database(format!(
                "no fue posible abrir el socket TCP a SQL Server: {error}"
            ))
        })?;

    tcp.set_nodelay(true).map_err(|error| {
        AppError::Database(format!("no fue posible configurar TCP_NODELAY: {error}"))
    })?;

    Client::connect(tiberius_config, tcp.compat_write())
        .await
        .map_err(|error| {
            AppError::Database(format!(
                "no fue posible completar el login en SQL Server: {error}"
            ))
        })
}

fn runtime() -> Result<Runtime, AppError> {
    Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| {
            AppError::Process(format!(
                "no fue posible crear el runtime async para SQL Server: {error}"
            ))
        })
}

fn validate_sql_server_config(config: &ConnectionConfig) -> Result<(), AppError> {
    if config.engine != DatabaseEngine::SqlServer {
        return Err(AppError::UnsupportedDatabaseEngine(format!(
            "se esperaba sqlserver y se recibio {}",
            config.engine
        )));
    }

    if config.connection_string.is_some() {
        return Ok(());
    }

    require_field(&config.host, "host")?;
    require_field(&config.database, "database")?;
    require_field(&config.username, "username")?;
    require_field(&config.password, "password")?;

    Ok(())
}

fn build_config(config: &ConnectionConfig) -> Result<Config, AppError> {
    if let Some(connection_string) = &config.connection_string {
        return Config::from_ado_string(connection_string).map_err(|error| {
            AppError::InvalidConfiguration(format!(
                "la connection string de SQL Server no es valida: {error}"
            ))
        });
    }

    let mut sql_config = Config::new();
    sql_config.host(config.host.as_deref().unwrap_or("localhost"));
    sql_config.port(config.port.unwrap_or(1433));
    sql_config.database(config.database.as_deref().unwrap_or("master"));
    sql_config.authentication(AuthMethod::sql_server(
        config.username.as_deref().unwrap_or_default(),
        config.password.as_deref().unwrap_or_default(),
    ));

    // Primera iteracion enfocada en ambientes de desarrollo. Lo volveremos configurable
    // antes de abrir la conexion a entornos mas estrictos.
    sql_config.trust_cert();
    sql_config.application_name("api-from-db-cli");

    Ok(sql_config)
}

fn require_field(value: &Option<String>, field: &str) -> Result<(), AppError> {
    match value.as_deref().map(str::trim) {
        Some("") | None => Err(AppError::MissingRequiredField(field.to_string())),
        Some(_) => Ok(()),
    }
}

fn map_object_row(row: tiberius::Row) -> Result<DatabaseObject, AppError> {
    let schema = required_string(&row, "schema_name")?;
    let name = required_string(&row, "object_name")?;
    let object_type = match required_string(&row, "object_type")?.as_str() {
        "table" => DatabaseObjectType::Table,
        "stored_procedure" => DatabaseObjectType::StoredProcedure,
        unexpected => {
            return Err(AppError::Database(format!(
                "tipo de objeto SQL Server no reconocido: {unexpected}"
            )));
        }
    };

    Ok(DatabaseObject {
        name,
        schema: Some(schema),
        object_type,
        engine: DatabaseEngine::SqlServer,
    })
}

fn map_column_row(row: tiberius::Row) -> Result<ColumnSchema, AppError> {
    let name = required_string(&row, "COLUMN_NAME")?;
    let base_type = required_string(&row, "DATA_TYPE")?;
    let nullable = required_string(&row, "IS_NULLABLE")?.eq_ignore_ascii_case("YES");
    let ordinal_position = required_i32(&row, "ORDINAL_POSITION")? as usize;
    let is_primary_key = optional_i32(&row, "IS_PRIMARY_KEY").unwrap_or(0) == 1;

    let rendered_db_type = render_sql_server_db_type(
        &base_type,
        optional_i32(&row, "CHARACTER_MAXIMUM_LENGTH"),
        optional_i32(&row, "NUMERIC_PRECISION"),
        optional_i32(&row, "NUMERIC_SCALE"),
        optional_i32(&row, "DATETIME_PRECISION"),
    );

    Ok(ColumnSchema {
        name,
        db_type: rendered_db_type,
        normalized_type: crate::domain::NormalizedType::Unknown,
        nullable,
        ordinal_position,
        is_primary_key,
        is_deleted_at_candidate: false,
    })
}

fn required_string(row: &tiberius::Row, column: &str) -> Result<String, AppError> {
    row.get::<&str, _>(column)
        .map(|value| value.to_string())
        .ok_or_else(|| {
            AppError::Database(format!(
                "la columna requerida `{column}` no estuvo presente en el resultado"
            ))
        })
}

fn required_i32(row: &tiberius::Row, column: &str) -> Result<i32, AppError> {
    optional_i32(row, column).ok_or_else(|| {
        AppError::Database(format!(
            "la columna numerica requerida `{column}` no estuvo presente o no pudo convertirse"
        ))
    })
}

fn optional_i32(row: &tiberius::Row, column: &str) -> Option<i32> {
    row.try_get::<i32, _>(column)
        .ok()
        .flatten()
        .or_else(|| row.try_get::<i16, _>(column).ok().flatten().map(i32::from))
        .or_else(|| row.try_get::<u8, _>(column).ok().flatten().map(i32::from))
}

fn render_sql_server_db_type(
    base_type: &str,
    char_max_length: Option<i32>,
    numeric_precision: Option<i32>,
    numeric_scale: Option<i32>,
    datetime_precision: Option<i32>,
) -> String {
    let base = base_type.to_ascii_lowercase();

    match base.as_str() {
        "varchar" | "nvarchar" | "char" | "nchar" => match char_max_length {
            Some(-1) => format!("{base}(max)"),
            Some(length) => format!("{base}({length})"),
            None => base,
        },
        "decimal" | "numeric" => match (numeric_precision, numeric_scale) {
            (Some(precision), Some(scale)) => format!("{base}({precision},{scale})"),
            _ => base,
        },
        "datetime2" | "datetimeoffset" | "time" => match datetime_precision {
            Some(precision) => format!("{base}({precision})"),
            None => base,
        },
        _ => base,
    }
}

const LIST_OBJECTS_SQL: &str = r#"
SELECT
    t.TABLE_SCHEMA AS schema_name,
    t.TABLE_NAME AS object_name,
    'table' AS object_type,
    1 AS sort_order
FROM INFORMATION_SCHEMA.TABLES t
WHERE t.TABLE_TYPE = 'BASE TABLE'
  AND t.TABLE_SCHEMA NOT IN ('sys', 'INFORMATION_SCHEMA')
UNION ALL
SELECT
    r.ROUTINE_SCHEMA AS schema_name,
    r.ROUTINE_NAME AS object_name,
    'stored_procedure' AS object_type,
    2 AS sort_order
FROM INFORMATION_SCHEMA.ROUTINES r
WHERE r.ROUTINE_TYPE = 'PROCEDURE'
  AND r.ROUTINE_SCHEMA NOT IN ('sys', 'INFORMATION_SCHEMA')
ORDER BY schema_name, sort_order, object_name;
"#;

const TABLE_SCHEMA_SQL: &str = r#"
SELECT
    c.COLUMN_NAME,
    c.DATA_TYPE,
    c.IS_NULLABLE,
    c.ORDINAL_POSITION,
    c.CHARACTER_MAXIMUM_LENGTH,
    c.NUMERIC_PRECISION,
    c.NUMERIC_SCALE,
    c.DATETIME_PRECISION,
    CASE WHEN EXISTS (
        SELECT 1
        FROM INFORMATION_SCHEMA.TABLE_CONSTRAINTS tc
        INNER JOIN INFORMATION_SCHEMA.KEY_COLUMN_USAGE kcu
            ON tc.CONSTRAINT_NAME = kcu.CONSTRAINT_NAME
           AND tc.TABLE_SCHEMA = kcu.TABLE_SCHEMA
           AND tc.TABLE_NAME = kcu.TABLE_NAME
        WHERE tc.CONSTRAINT_TYPE = 'PRIMARY KEY'
          AND kcu.TABLE_SCHEMA = c.TABLE_SCHEMA
          AND kcu.TABLE_NAME = c.TABLE_NAME
          AND kcu.COLUMN_NAME = c.COLUMN_NAME
    ) THEN 1 ELSE 0 END AS IS_PRIMARY_KEY
FROM INFORMATION_SCHEMA.COLUMNS c
WHERE c.TABLE_SCHEMA = @P1
  AND c.TABLE_NAME = @P2
ORDER BY c.ORDINAL_POSITION;
"#;

#[cfg(test)]
mod tests {
    use super::{render_sql_server_db_type, validate_sql_server_config};
    use crate::core::error::AppError;
    use crate::domain::{ConnectionConfig, DatabaseEngine, GeneratorConfig};

    #[test]
    fn renders_varchar_with_length() {
        assert_eq!(
            render_sql_server_db_type("varchar", Some(50), None, None, None),
            "varchar(50)"
        );
    }

    #[test]
    fn renders_decimal_with_precision_and_scale() {
        assert_eq!(
            render_sql_server_db_type("decimal", None, Some(10), Some(2), None),
            "decimal(10,2)"
        );
    }

    #[test]
    fn rejects_incomplete_manual_sql_server_config() {
        let config = ConnectionConfig {
            id: "1".to_string(),
            name: "sqlserver".to_string(),
            engine: DatabaseEngine::SqlServer,
            host: Some("localhost".to_string()),
            port: Some(1433),
            database: None,
            username: Some("sa".to_string()),
            password: Some("secret".to_string()),
            connection_string: None,
            config_file_path: None,
            generator: GeneratorConfig::default(),
        };

        let result = validate_sql_server_config(&config);
        assert!(
            matches!(result, Err(AppError::MissingRequiredField(field)) if field == "database")
        );
    }
}
