use crate::app::generation_service::GenerationPreview;
use crate::domain::{
    ConnectionConfig, DatabaseEngine, DatabaseObject, GeneratorConfig, SoftDeletePreference,
};
use crate::ui::state::{CatalogMode, MockTable, SoftDeleteStrategy};
use crate::ui::use_cases::table_preview::{self, LoadTablePreviewInput, LoadTablePreviewOutcome};

pub(crate) struct ResolveSoftDeleteInput<'a> {
    pub(crate) strategy: SoftDeleteStrategy,
    pub(crate) selected: &'a DatabaseObject,
    pub(crate) engine: DatabaseEngine,
    pub(crate) catalog_mode: CatalogMode,
    pub(crate) catalog_tables: &'a [MockTable],
    pub(crate) connection_config: Option<&'a ConnectionConfig>,
    pub(crate) generator_config: GeneratorConfig,
}

pub(crate) struct ResolveSoftDeleteWithFieldInput<'a> {
    pub(crate) field: &'a str,
    pub(crate) selected: &'a DatabaseObject,
    pub(crate) engine: DatabaseEngine,
    pub(crate) catalog_mode: CatalogMode,
    pub(crate) catalog_tables: &'a [MockTable],
    pub(crate) connection_config: Option<&'a ConnectionConfig>,
    pub(crate) generator_config: GeneratorConfig,
}

pub(crate) enum ResolveSoftDeleteOutcome {
    ShowSqlPreview {
        preview: GenerationPreview,
        message: String,
    },
    ShowCommandPreview {
        preview: GenerationPreview,
        message: String,
    },
    RequestManualField {
        message: String,
    },
    Error {
        message: String,
    },
}

pub(crate) fn resolve(input: ResolveSoftDeleteInput<'_>) -> ResolveSoftDeleteOutcome {
    match input.strategy {
        SoftDeleteStrategy::CreateDeletedAt => load_preview(
            input.selected,
            input.engine,
            input.catalog_mode,
            input.catalog_tables,
            input.connection_config,
            input.generator_config,
            SoftDeletePreference::PreferDeleteEndpoint,
            None,
            |preview| ResolveSoftDeleteOutcome::ShowSqlPreview {
                preview,
                message: "Se genero el ALTER TABLE para revisar antes de continuar.".to_string(),
            },
        ),
        SoftDeleteStrategy::UseExistingField => ResolveSoftDeleteOutcome::RequestManualField {
            message: "Selecciona una columna datetime existente para soft delete.".to_string(),
        },
        SoftDeleteStrategy::ContinueWithoutDelete => load_preview(
            input.selected,
            input.engine,
            input.catalog_mode,
            input.catalog_tables,
            input.connection_config,
            input.generator_config,
            SoftDeletePreference::SkipDeleteEndpoint,
            None,
            |preview| ResolveSoftDeleteOutcome::ShowCommandPreview {
                preview,
                message: "Seguimos sin endpoint delete para esta API.".to_string(),
            },
        ),
    }
}

pub(crate) fn resolve_with_existing_field(
    input: ResolveSoftDeleteWithFieldInput<'_>,
) -> ResolveSoftDeleteOutcome {
    let field = input.field.to_string();
    load_preview(
        input.selected,
        input.engine,
        input.catalog_mode,
        input.catalog_tables,
        input.connection_config,
        input.generator_config,
        SoftDeletePreference::PreferDeleteEndpoint,
        Some(input.field),
        move |preview| ResolveSoftDeleteOutcome::ShowCommandPreview {
            preview,
            message: format!("Usaremos `{}` como columna de soft delete.", field),
        },
    )
}

fn load_preview<F>(
    selected: &DatabaseObject,
    engine: DatabaseEngine,
    catalog_mode: CatalogMode,
    catalog_tables: &[MockTable],
    connection_config: Option<&ConnectionConfig>,
    generator_config: GeneratorConfig,
    preference: SoftDeletePreference,
    manually_selected_field: Option<&str>,
    on_loaded: F,
) -> ResolveSoftDeleteOutcome
where
    F: FnOnce(GenerationPreview) -> ResolveSoftDeleteOutcome,
{
    match table_preview::load(LoadTablePreviewInput {
        selected,
        engine,
        catalog_mode,
        catalog_tables,
        connection_config,
        generator_config,
        preference,
        manually_selected_field,
    }) {
        LoadTablePreviewOutcome::Loaded(preview) => on_loaded(preview),
        LoadTablePreviewOutcome::Error { message } => ResolveSoftDeleteOutcome::Error { message },
    }
}
