use crate::domain::{GeneratedCommand, NormalizedType, SoftDeleteConfig, TableSchema};

#[derive(Debug, Clone)]
pub struct CommandBuilder {
    executable: String,
}

impl CommandBuilder {
    pub fn new(executable: impl Into<String>) -> Self {
        Self {
            executable: executable.into(),
        }
    }

    pub fn build(&self, schema: &TableSchema, soft_delete: &SoftDeleteConfig) -> GeneratedCommand {
        let mut parts = Vec::with_capacity(schema.columns.len());

        for column in &schema.columns {
            let rendered_type = if soft_delete.enabled
                && soft_delete.field.as_deref() == Some(column.name.as_str())
            {
                NormalizedType::DeleteAt
            } else {
                column.normalized_type
            };

            parts.push(format!(
                "{}:{}",
                column.name,
                rendered_type.as_generator_token()
            ));
        }

        let fields_csv = parts.join(",");
        let arguments = vec![schema.name.clone(), fields_csv.clone()];
        let raw_command = format!("{} {} {}", self.executable, schema.name, fields_csv);

        GeneratedCommand {
            executable: self.executable.clone(),
            table_name: schema.name.clone(),
            arguments,
            raw_command,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CommandBuilder;
    use crate::app::soft_delete::SoftDeleteResolver;
    use crate::app::type_mapper::TypeMapper;
    use crate::domain::{ColumnSchema, DatabaseEngine, SoftDeletePreference, TableSchema};

    #[test]
    fn uses_delete_at_token_for_soft_delete_column() {
        let mapper = TypeMapper::new(DatabaseEngine::PostgreSql);
        let resolver = SoftDeleteResolver::new(DatabaseEngine::PostgreSql);
        let builder = CommandBuilder::new("gen.exe");
        let schema = mapper.analyze_table(TableSchema::new(
            "public",
            "users",
            vec![
                ColumnSchema::new("id", "integer", false, 1),
                ColumnSchema::new("deleted_at", "timestamp", true, 2),
            ],
            vec!["id".to_string()],
        ));
        let soft_delete =
            resolver.resolve(&schema, SoftDeletePreference::PreferDeleteEndpoint, None);
        let command = builder.build(&schema, &soft_delete);

        assert_eq!(
            command.raw_command,
            "gen.exe users id:int,deleted_at:delete_at"
        );
    }
}
