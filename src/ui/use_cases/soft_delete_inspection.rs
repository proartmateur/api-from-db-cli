use crate::app::generation_service::GenerationPreview;

pub(crate) fn manual_field_candidates(preview: Option<&GenerationPreview>) -> Vec<String> {
    preview
        .map(|preview| {
            preview
                .analyzed_schema
                .columns
                .iter()
                .filter(|column| column.normalized_type.as_generator_token() == "datetime")
                .map(|column| column.name.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub(crate) fn needs_soft_delete_decision(preview: Option<&GenerationPreview>) -> bool {
    preview
        .map(|preview| preview.soft_delete.field_was_created)
        .unwrap_or(false)
}
