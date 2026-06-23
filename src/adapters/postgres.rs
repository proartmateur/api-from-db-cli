use std::future::Future;
use std::str::FromStr;

use tokio::runtime::{Builder, Runtime};
use tokio_postgres::config::Config as PgConfig;
use tokio_postgres::{Client, NoTls, Row};

use crate::core::error::AppError;
use crate::core::ports::{ConnectionProvider, MetadataExplorer, SqlExecutor};
use crate::domain::{
    ColumnSchema, ConnectionConfig, DatabaseEngine, DatabaseObject, DatabaseObjectType, TableSchema,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct PostgresAdapter;

impl ConnectionProvider for PostgresAdapter {
    fn test_connection(&self, config: &ConnectionConfig) -> Result<(), AppError> {
        self.with_runtime(async {
            let client = connect(config).await?;
            client
                .simple_query("SELECT 1 AS healthcheck")
                .await
                .map_err(|error| {
                    AppError::Database(format!("no fue posible validar la conexion: {error}"))
                })?;
            Ok(())
        })
    }
}

impl MetadataExplorer for PostgresAdapter {
    fn list_objects(&self, config: &ConnectionConfig) -> Result<Vec<DatabaseObject>, AppError> {
        self.with_runtime(async {
            let client = connect(config).await?;
            let rows = client
                .query(LIST_OBJECTS_SQL, &[])
                .await
                .map_err(|error| {
                    AppError::Database(format!("no fue posible listar objetos: {error}"))
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
            let client = connect(config).await?;
            let rows = client
                .query(TABLE_SCHEMA_SQL, &[&schema, &table])
                .await
                .map_err(|error| {
                    AppError::Database(format!(
                        "no fue posible leer la estructura de la tabla {schema}.{table}: {error}"
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

impl SqlExecutor for PostgresAdapter {
    fn execute_sql(&self, config: &ConnectionConfig, sql: &str) -> Result<(), AppError> {
        self.with_runtime(async {
            let client = connect(config).await?;
            client
                .simple_query(sql)
                .await
                .map_err(|error| {
                    AppError::Database(format!(
                        "no fue posible ejecutar SQL en PostgreSQL: {error}"
                    ))
                })?;
            Ok(())
        })
    }
}

impl PostgresAdapter {
    fn with_runtime<T, F>(&self, future: F) -> Result<T, AppError>
    where
        F: Future<Output = Result<T, AppError>>,
    {
        runtime()?.block_on(future)
    }
}

async fn connect(config: &ConnectionConfig) -> Result<Client, AppError> {
    validate_postgres_config(config)?;

    let pg_config = build_config(config)?;
    let (client, connection) = pg_config
        .connect(NoTls)
        .await
        .map_err(|error| {
            AppError::Database(format!(
                "no fue posible completar el login en PostgreSQL: {error}"
            ))
        })?;

    tokio::spawn(async move {
        if let Err(error) = connection.await {
            eprintln!("postgres connection error: {error}");
        }
    });

    Ok(client)
}

fn runtime() -> Result<Runtime, AppError> {
    Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| {
            AppError::Process(format!(
                "no fue posible crear el runtime async para PostgreSQL: {error}"
            ))
        })
}

fn validate_postgres_config(config: &ConnectionConfig) -> Result<(), AppError> {
    if config.engine != DatabaseEngine::PostgreSql {
        return Err(AppError::UnsupportedDatabaseEngine(format!(
            "se esperaba postgresql y se recibio {}",
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

fn build_config(config: &ConnectionConfig) -> Result<PgConfig, AppError> {
    if let Some(connection_string) = &config.connection_string {
        return PgConfig::from_str(connection_string).map_err(|error| {
            AppError::InvalidConfiguration(format!(
                "la connection string de PostgreSQL no es valida: {error}"
            ))
        });
    }

    let mut pg_config = PgConfig::new();
    pg_config.host(config.host.as_deref().unwrap_or("localhost"));
    pg_config.port(config.port.unwrap_or(5432));
    pg_config.dbname(config.database.as_deref().unwrap_or("postgres"));
    pg_config.user(config.username.as_deref().unwrap_or_default());
    pg_config.password(config.password.as_deref().unwrap_or_default());
    pg_config.application_name("api-from-db-cli");

    Ok(pg_config)
}

fn require_field(value: &Option<String>, field: &str) -> Result<(), AppError> {
    match value.as_deref().map(str::trim) {
        Some("") | None => Err(AppError::MissingRequiredField(field.to_string())),
        Some(_) => Ok(()),
    }
}

fn map_object_row(row: Row) -> Result<DatabaseObject, AppError> {
    let schema = required_string(&row, "schema_name")?;
    let name = required_string(&row, "object_name")?;
    let object_type = match required_string(&row, "object_type")?.as_str() {
        "table" => DatabaseObjectType::Table,
        "function" => DatabaseObjectType::Function,
        unexpected => {
            return Err(AppError::Database(format!(
                "tipo de objeto PostgreSQL no reconocido: {unexpected}"
            )));
        }
    };

    Ok(DatabaseObject {
        name,
        schema: Some(schema),
        object_type,
        engine: DatabaseEngine::PostgreSql,
    })
}

fn map_column_row(row: Row) -> Result<ColumnSchema, AppError> {
    let name = required_string(&row, "column_name")?;
    let base_type = required_string(&row, "data_type")?;
    let udt_name = optional_string(&row, "udt_name");
    let nullable = required_string(&row, "is_nullable")?.eq_ignore_ascii_case("YES");
    let ordinal_position = required_i32(&row, "ordinal_position")? as usize;
    let is_primary_key = optional_i32(&row, "is_primary_key").unwrap_or(0) == 1;

    let char_max_length = optional_i32(&row, "character_maximum_length");
    let numeric_precision = optional_i32(&row, "numeric_precision");
    let numeric_scale = optional_i32(&row, "numeric_scale");
    let datetime_precision = optional_i32(&row, "datetime_precision");

    let rendered_db_type = render_postgres_db_type(
        &base_type,
        udt_name.as_deref(),
        char_max_length,
        numeric_precision,
        numeric_scale,
        datetime_precision,
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

fn required_string(row: &Row, column: &str) -> Result<String, AppError> {
    row.try_get::<_, String>(column)
        .map_err(|error| {
            AppError::Database(format!(
                "la columna requerida `{column}` no estuvo presente en el resultado: {error}"
            ))
        })
}

fn optional_string(row: &Row, column: &str) -> Option<String> {
    row.try_get::<_, Option<String>>(column)
        .ok()
        .flatten()
        .filter(|value| !value.is_empty())
}

fn required_i32(row: &Row, column: &str) -> Result<i32, AppError> {
    optional_i32(row, column).ok_or_else(|| {
        AppError::Database(format!(
            "la columna numerica requerida `{column}` no estuvo presente o no pudo convertirse"
        ))
    })
}

fn optional_i32(row: &Row, column: &str) -> Option<i32> {
    row.try_get::<_, Option<i32>>(column)
        .ok()
        .flatten()
        .or_else(|| {
            row.try_get::<_, Option<i16>>(column)
                .ok()
                .flatten()
                .map(i32::from)
        })
        .or_else(|| {
            row.try_get::<_, Option<u32>>(column)
                .ok()
                .flatten()
                .map(|value| value as i32)
        })
}

fn render_postgres_db_type(
    base_type: &str,
    udt_name: Option<&str>,
    char_max_length: Option<i32>,
    numeric_precision: Option<i32>,
    numeric_scale: Option<i32>,
    datetime_precision: Option<i32>,
) -> String {
    let base = base_type.to_ascii_lowercase();

    if base == "array" {
        return udt_name.unwrap_or("unknown[]").to_string();
    }

    if base == "user-defined" {
        return udt_name.unwrap_or("user-defined").to_string();
    }

    match base.as_str() {
        "character varying" | "character" | "char" => match char_max_length {
            Some(-1) => format!("{base}(max)"),
            Some(length) => format!("{base}({length})"),
            None => base,
        },
        "numeric" | "decimal" => match (numeric_precision, numeric_scale) {
            (Some(precision), Some(scale)) => format!("{base}({precision},{scale})"),
            _ => base,
        },
        "timestamp" | "timestamptz" | "time" | "timetz" => match datetime_precision {
            Some(precision) => format!("{base}({precision})"),
            None => base,
        },
        _ => base,
    }
}

const LIST_OBJECTS_SQL: &str = r#"
SELECT
    c.relnamespace::regnamespace::text AS schema_name,
    c.relname AS object_name,
    CASE
        WHEN c.relkind = 'r' THEN 'table'
        WHEN c.relkind = 'p' THEN 'table'
        WHEN p.prokind = 'f' THEN 'function'
        WHEN p.prokind = 'p' THEN 'function'
        ELSE NULL
    END AS object_type
FROM pg_catalog.pg_class c
LEFT JOIN pg_catalog.pg_proc p
    ON p.oid = ANY (
        CASE
            WHEN c.relkind IN ('r', 'p') THEN ARRAY[]::oid[]
            ELSE (
                SELECT array_agg(pr.oid)
                FROM pg_catalog.pg_proc pr
                WHERE pr.pronamespace = c.relnamespace
            )
        END
    )
WHERE c.relnamespace NOT IN (
    'pg_catalog'::regnamespace,
    'information_schema'::regnamespace,
    'pg_toast'::regnamespace
)
  AND (
        c.relkind IN ('r', 'p')
        OR (p.oid IS NOT NULL AND p.prokind IN ('f', 'p'))
    )
ORDER BY schema_name, object_type, object_name;
"#;

const TABLE_SCHEMA_SQL: &str = r#"
SELECT
    c.column_name,
    c.data_type,
    c.udt_name,
    c.is_nullable,
    c.ordinal_position,
    c.character_maximum_length,
    c.numeric_precision,
    c.numeric_scale,
    c.datetime_precision,
    CASE WHEN EXISTS (
        SELECT 1
        FROM information_schema.table_constraints tc
        INNER JOIN information_schema.key_column_usage kcu
            ON tc.constraint_name = kcu.constraint_name
           AND tc.table_schema = kcu.table_schema
           AND tc.table_name = kcu.table_name
        WHERE tc.constraint_type = 'PRIMARY KEY'
          AND kcu.table_schema = c.table_schema
          AND kcu.table_name = c.table_name
          AND kcu.column_name = c.column_name
    ) THEN 1 ELSE 0 END AS is_primary_key
FROM information_schema.columns c
WHERE c.table_schema = $1
  AND c.table_name = $2
ORDER BY c.ordinal_position;
"#;

#[cfg(test)]
mod tests {
    use super::{render_postgres_db_type, validate_postgres_config};
    use crate::core::error::AppError;
    use crate::domain::{ConnectionConfig, DatabaseEngine, GeneratorConfig};

    #[test]
    fn renders_varchar_with_length() {
        assert_eq!(
            render_postgres_db_type("character varying", None, Some(50), None, None, None),
            "character varying(50)"
        );
    }

    #[test]
    fn renders_numeric_with_precision_and_scale() {
        assert_eq!(
            render_postgres_db_type("numeric", None, None, Some(10), Some(2), None),
            "numeric(10,2)"
        );
    }

    #[test]
    fn renders_timestamptz_with_precision() {
        assert_eq!(
            render_postgres_db_type("timestamptz", None, None, None, None, Some(6)),
            "timestamptz(6)"
        );
    }

    #[test]
    fn renders_array_using_udt_name() {
        assert_eq!(
            render_postgres_db_type("array", Some("_int4"), None, None, None, None),
            "_int4"
        );
    }

    #[test]
    fn rejects_incomplete_manual_postgres_config() {
        let config = ConnectionConfig {
            id: "1".to_string(),
            name: "postgres".to_string(),
            engine: DatabaseEngine::PostgreSql,
            host: Some("localhost".to_string()),
            port: Some(5432),
            database: None,
            username: Some("postgres".to_string()),
            password: Some("secret".to_string()),
            connection_string: None,
            config_file_path: None,
            generator: GeneratorConfig::default(),
        };

        let result = validate_postgres_config(&config);
        assert!(
            matches!(result, Err(AppError::MissingRequiredField(field)) if field == "database")
        );
    }

    #[test]
    fn accepts_connection_string_and_skips_field_validation() {
        let config = ConnectionConfig {
            id: "1".to_string(),
            name: "postgres".to_string(),
            engine: DatabaseEngine::PostgreSql,
            host: None,
            port: None,
            database: None,
            username: None,
            password: None,
            connection_string: Some("postgresql://localhost/db".to_string()),
            config_file_path: None,
            generator: GeneratorConfig::default(),
        };

        let result = validate_postgres_config(&config);
        assert!(result.is_ok());
    }
}