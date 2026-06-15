use crate::domain::{ColumnSchema, DatabaseEngine, NormalizedType, TableSchema};

#[derive(Debug, Clone)]
pub struct TypeMapper {
    engine: DatabaseEngine,
}

impl TypeMapper {
    pub fn new(engine: DatabaseEngine) -> Self {
        Self { engine }
    }

    pub fn analyze_table(&self, schema: TableSchema) -> TableSchema {
        let primary_keys = schema.primary_keys.clone();
        let mapped_columns = schema
            .columns
            .into_iter()
            .map(|column| self.map_column(column, &primary_keys))
            .collect();

        TableSchema {
            schema: schema.schema,
            name: schema.name,
            columns: mapped_columns,
            primary_keys,
        }
    }

    pub fn map_column(&self, mut column: ColumnSchema, primary_keys: &[String]) -> ColumnSchema {
        let normalized_type = self.map_db_type(&column.db_type);
        let is_deleted_at_candidate = column.name.eq_ignore_ascii_case("deleted_at")
            && self.is_datetime_type(&normalized_type);

        column.normalized_type = normalized_type;
        column.is_primary_key = primary_keys
            .iter()
            .any(|pk| pk.eq_ignore_ascii_case(&column.name));
        column.is_deleted_at_candidate = is_deleted_at_candidate;
        column
    }

    pub fn map_db_type(&self, db_type: &str) -> NormalizedType {
        let base = canonical_db_type(db_type);

        match self.engine {
            DatabaseEngine::PostgreSql => match base.as_str() {
                "smallint" | "integer" | "int" | "bigint" | "serial" | "bigserial" => {
                    NormalizedType::Int
                }
                "real" | "double precision" => NormalizedType::Float,
                "numeric" | "decimal" => NormalizedType::Decimal,
                "character varying" | "varchar" | "character" | "char" | "text" | "citext"
                | "uuid" => NormalizedType::Str,
                "boolean" | "bool" => NormalizedType::Bool,
                "json" | "jsonb" => NormalizedType::Dict,
                "date" => NormalizedType::Date,
                "time" | "time without time zone" | "time with time zone" => NormalizedType::Time,
                "timestamp"
                | "timestamp without time zone"
                | "timestamp with time zone"
                | "timestamptz" => NormalizedType::Datetime,
                _ => NormalizedType::Unknown,
            },
            DatabaseEngine::SqlServer => match base.as_str() {
                "tinyint" | "smallint" | "int" | "bigint" => NormalizedType::Int,
                "float" | "real" => NormalizedType::Float,
                "numeric" | "decimal" | "money" | "smallmoney" => NormalizedType::Decimal,
                "varchar" | "nvarchar" | "char" | "nchar" | "text" | "ntext"
                | "uniqueidentifier" => NormalizedType::Str,
                "bit" => NormalizedType::Bool,
                "date" => NormalizedType::Date,
                "time" => NormalizedType::Time,
                "datetime" | "datetime2" | "smalldatetime" | "datetimeoffset" => {
                    NormalizedType::Datetime
                }
                _ => NormalizedType::Unknown,
            },
        }
    }

    fn is_datetime_type(&self, normalized_type: &NormalizedType) -> bool {
        matches!(normalized_type, NormalizedType::Datetime)
    }
}

fn canonical_db_type(db_type: &str) -> String {
    let without_size = db_type
        .trim()
        .split_once('(')
        .map_or(db_type.trim(), |(base, _)| base);

    without_size
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::TypeMapper;
    use crate::domain::{DatabaseEngine, NormalizedType};

    #[test]
    fn maps_postgres_varchar_to_python_str() {
        let mapper = TypeMapper::new(DatabaseEngine::PostgreSql);
        assert_eq!(mapper.map_db_type("varchar(50)"), NormalizedType::Str);
    }

    #[test]
    fn maps_sql_server_datetime2_to_python_datetime() {
        let mapper = TypeMapper::new(DatabaseEngine::SqlServer);
        assert_eq!(mapper.map_db_type("datetime2"), NormalizedType::Datetime);
    }

    #[test]
    fn maps_postgres_jsonb_to_dict() {
        let mapper = TypeMapper::new(DatabaseEngine::PostgreSql);
        assert_eq!(mapper.map_db_type("jsonb"), NormalizedType::Dict);
    }
}
