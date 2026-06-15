use crate::domain::{NormalizedType, SoftDeleteConfig, SoftDeletePreference, TableSchema};

#[derive(Debug, Clone)]
pub struct SoftDeleteResolver;

impl SoftDeleteResolver {
    pub fn new(_engine: crate::domain::DatabaseEngine) -> Self {
        Self
    }

    pub fn resolve(
        &self,
        schema: &TableSchema,
        preference: SoftDeletePreference,
        manually_selected_field: Option<&str>,
    ) -> SoftDeleteConfig {
        if let Some(column) = schema
            .columns
            .iter()
            .find(|column| column.is_deleted_at_candidate)
        {
            return SoftDeleteConfig {
                enabled: true,
                field: Some(column.name.clone()),
                field_was_detected: true,
                field_was_selected_manually: false,
                field_was_created: false,
            };
        }

        if let Some(field_name) = manually_selected_field {
            let selected = schema.columns.iter().find(|column| {
                column.name.eq_ignore_ascii_case(field_name)
                    && matches!(column.normalized_type, NormalizedType::Datetime)
            });

            if let Some(column) = selected {
                return SoftDeleteConfig {
                    enabled: true,
                    field: Some(column.name.clone()),
                    field_was_detected: false,
                    field_was_selected_manually: true,
                    field_was_created: false,
                };
            }
        }

        match preference {
            SoftDeletePreference::SkipDeleteEndpoint => SoftDeleteConfig::disabled(),
            SoftDeletePreference::PreferDeleteEndpoint => SoftDeleteConfig {
                enabled: true,
                field: Some("deleted_at".to_string()),
                field_was_detected: false,
                field_was_selected_manually: false,
                field_was_created: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SoftDeleteResolver;
    use crate::app::type_mapper::TypeMapper;
    use crate::domain::{ColumnSchema, DatabaseEngine, SoftDeletePreference, TableSchema};

    #[test]
    fn detects_deleted_at_automatically() {
        let mapper = TypeMapper::new(DatabaseEngine::PostgreSql);
        let resolver = SoftDeleteResolver::new(DatabaseEngine::PostgreSql);
        let schema = mapper.analyze_table(TableSchema::new(
            "public",
            "users",
            vec![ColumnSchema::new("deleted_at", "timestamp", true, 1)],
            vec![],
        ));

        let config = resolver.resolve(&schema, SoftDeletePreference::PreferDeleteEndpoint, None);
        assert!(config.field_was_detected);
        assert_eq!(config.field.as_deref(), Some("deleted_at"));
    }

    #[test]
    fn requests_creation_when_delete_endpoint_is_required() {
        let mapper = TypeMapper::new(DatabaseEngine::PostgreSql);
        let resolver = SoftDeleteResolver::new(DatabaseEngine::PostgreSql);
        let schema = mapper.analyze_table(TableSchema::new(
            "public",
            "users",
            vec![ColumnSchema::new("updated_at", "timestamp", true, 1)],
            vec![],
        ));

        let config = resolver.resolve(&schema, SoftDeletePreference::PreferDeleteEndpoint, None);
        assert!(config.field_was_created);
        assert_eq!(config.field.as_deref(), Some("deleted_at"));
    }
}
