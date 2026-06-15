use crate::domain::{DatabaseEngine, GeneratedSql};

#[derive(Debug, Clone)]
pub struct DdlBuilder {
    engine: DatabaseEngine,
}

impl DdlBuilder {
    pub fn new(engine: DatabaseEngine) -> Self {
        Self { engine }
    }

    pub fn build_add_deleted_at(&self, schema: &str, table: &str) -> GeneratedSql {
        let sql = match self.engine {
            DatabaseEngine::PostgreSql => format!(
                "ALTER TABLE \"{}\".\"{}\" ADD COLUMN \"deleted_at\" TIMESTAMP NULL;",
                escape_postgres_identifier(schema),
                escape_postgres_identifier(table)
            ),
            DatabaseEngine::SqlServer => format!(
                "ALTER TABLE [{}].[{}] ADD [deleted_at] DATETIME2 NULL;",
                escape_sql_server_identifier(schema),
                escape_sql_server_identifier(table)
            ),
        };

        GeneratedSql {
            engine: self.engine,
            schema: schema.to_string(),
            table: table.to_string(),
            sql,
            operation: "alter_table_add_deleted_at".to_string(),
        }
    }
}

fn escape_postgres_identifier(value: &str) -> String {
    value.replace('"', "\"\"")
}

fn escape_sql_server_identifier(value: &str) -> String {
    value.replace(']', "]]")
}

#[cfg(test)]
mod tests {
    use super::DdlBuilder;
    use crate::domain::DatabaseEngine;

    #[test]
    fn builds_postgres_deleted_at_sql() {
        let builder = DdlBuilder::new(DatabaseEngine::PostgreSql);
        let sql = builder.build_add_deleted_at("public", "users");
        assert_eq!(
            sql.sql,
            "ALTER TABLE \"public\".\"users\" ADD COLUMN \"deleted_at\" TIMESTAMP NULL;"
        );
    }

    #[test]
    fn builds_sql_server_deleted_at_sql() {
        let builder = DdlBuilder::new(DatabaseEngine::SqlServer);
        let sql = builder.build_add_deleted_at("dbo", "Users");
        assert_eq!(
            sql.sql,
            "ALTER TABLE [dbo].[Users] ADD [deleted_at] DATETIME2 NULL;"
        );
    }
}
