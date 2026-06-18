use crate::adapters::sqlserver::SqlServerAdapter;
use crate::app::generation_service::{GenerationPreview, GenerationService};
use crate::core::ports::MetadataExplorer;
use crate::domain::{
    ConnectionConfig, DatabaseEngine, DatabaseObject, GeneratorConfig, SoftDeletePreference,
    TableSchema,
};
use crate::ui::state::{CatalogMode, MockTable};

pub(crate) struct LoadTablePreviewInput<'a> {
    pub(crate) selected: &'a DatabaseObject,
    pub(crate) engine: DatabaseEngine,
    pub(crate) catalog_mode: CatalogMode,
    pub(crate) catalog_tables: &'a [MockTable],
    pub(crate) connection_config: Option<&'a ConnectionConfig>,
    pub(crate) generator_config: GeneratorConfig,
    pub(crate) preference: SoftDeletePreference,
    pub(crate) manually_selected_field: Option<&'a str>,
}

pub(crate) enum LoadTablePreviewOutcome {
    Loaded(GenerationPreview),
    Error { message: String },
}

pub(crate) fn load(input: LoadTablePreviewInput<'_>) -> LoadTablePreviewOutcome {
    let schema = match input.catalog_mode {
        CatalogMode::Mock => mock_table_schema(input.selected, input.catalog_tables),
        CatalogMode::Real => real_table_schema(input.selected, input.connection_config),
    };

    match schema {
        Ok(schema) => LoadTablePreviewOutcome::Loaded(preview_from_schema(
            schema,
            input.engine,
            input.generator_config,
            input.preference,
            input.manually_selected_field,
        )),
        Err(error) => LoadTablePreviewOutcome::Error {
            message: format!(
                "No fue posible leer la estructura de {}.{}: {}",
                input.selected.schema.as_deref().unwrap_or("<sin schema>"),
                input.selected.name,
                error
            ),
        },
    }
}

fn preview_from_schema(
    schema: TableSchema,
    engine: DatabaseEngine,
    generator_config: GeneratorConfig,
    preference: SoftDeletePreference,
    manually_selected_field: Option<&str>,
) -> GenerationPreview {
    let service = GenerationService::new(engine, generator_config);
    service.preview_from_table(schema, preference, manually_selected_field)
}

fn mock_table_schema(
    selected: &DatabaseObject,
    catalog_tables: &[MockTable],
) -> Result<TableSchema, String> {
    catalog_tables
        .iter()
        .find(|table| {
            table.schema.name == selected.name
                && table.schema.schema == selected.schema.clone().unwrap_or_default()
        })
        .map(|table| table.schema.clone())
        .ok_or_else(|| "la tabla no existe en el catalogo mock".to_string())
}

fn real_table_schema(
    selected: &DatabaseObject,
    connection_config: Option<&ConnectionConfig>,
) -> Result<TableSchema, String> {
    let config = connection_config.ok_or_else(|| "no hay conexion real activa".to_string())?;
    let schema = selected
        .schema
        .as_deref()
        .ok_or_else(|| "el objeto no trae schema".to_string())?;

    let adapter = SqlServerAdapter;
    adapter
        .get_table_schema(config, schema, &selected.name)
        .map_err(|error| error.to_string())
}
