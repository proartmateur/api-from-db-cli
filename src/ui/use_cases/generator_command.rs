use crate::adapters::process::StdProcessRunner;
use crate::app::generation_service::GenerationPreview;
use crate::core::ports::ProcessRunner;
use crate::domain::ProcessResult;

pub(crate) fn execute(preview: Option<&GenerationPreview>) -> Result<ProcessResult, String> {
    let preview = preview.ok_or_else(|| "No hay comando generado para ejecutar.".to_string())?;

    let runner = StdProcessRunner;
    runner
        .run(&preview.generated_command)
        .map_err(|error| error.to_string())
}
