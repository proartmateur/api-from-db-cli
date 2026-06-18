use crate::app::generation_service::GenerationPreview;
use crate::domain::ProcessResult;

pub fn external_process_result(preview: Option<&GenerationPreview>) -> ProcessResult {
    let command = preview
        .map(|item| item.generated_command.raw_command.clone())
        .unwrap_or_else(|| "gen.exe demo id:int".to_string());

    ProcessResult {
        exit_code: 0,
        stdout: format!(
            "Comando copiado para ejecucion externa:\n{command}\nPuedes correrlo en otra herramienta y volver despues."
        ),
        stderr: String::new(),
        success: true,
    }
}
