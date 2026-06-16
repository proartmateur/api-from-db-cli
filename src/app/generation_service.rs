use crate::app::command_builder::CommandBuilder;
use crate::app::ddl_builder::DdlBuilder;
use crate::app::soft_delete::SoftDeleteResolver;
use crate::app::type_mapper::TypeMapper;
use crate::domain::{
    DatabaseEngine, GeneratedCommand, GeneratedSql, GeneratorConfig, SoftDeleteConfig,
    SoftDeletePreference, TableSchema,
};

#[derive(Debug, Clone)]
pub struct GenerationPreview {
    pub analyzed_schema: TableSchema,
    pub soft_delete: SoftDeleteConfig,
    pub generated_sql: Option<GeneratedSql>,
    pub generated_command: GeneratedCommand,
}

#[derive(Debug, Clone)]
pub struct GenerationService {
    mapper: TypeMapper,
    soft_delete_resolver: SoftDeleteResolver,
    ddl_builder: DdlBuilder,
    command_builder: CommandBuilder,
}

impl GenerationService {
    pub fn new(engine: DatabaseEngine, generator: GeneratorConfig) -> Self {
        Self {
            mapper: TypeMapper::new(engine),
            soft_delete_resolver: SoftDeleteResolver::new(engine),
            ddl_builder: DdlBuilder::new(engine),
            command_builder: CommandBuilder::new(generator),
        }
    }

    pub fn preview_from_table(
        &self,
        schema: TableSchema,
        preference: SoftDeletePreference,
        manually_selected_field: Option<&str>,
    ) -> GenerationPreview {
        let analyzed_schema = self.mapper.analyze_table(schema);
        let soft_delete = self.soft_delete_resolver.resolve(
            &analyzed_schema,
            preference,
            manually_selected_field,
        );
        let generated_sql = if soft_delete.field_was_created {
            Some(
                self.ddl_builder
                    .build_add_deleted_at(&analyzed_schema.schema, &analyzed_schema.name),
            )
        } else {
            None
        };
        let generated_command = self.command_builder.build(&analyzed_schema, &soft_delete);

        GenerationPreview {
            analyzed_schema,
            soft_delete,
            generated_sql,
            generated_command,
        }
    }
}
