use std::env;

use crate::ui::state::GeneratorBinaryState;

pub(crate) struct GeneratorValidationOutcome {
    pub(crate) state: GeneratorBinaryState,
    pub(crate) is_ready: bool,
    pub(crate) message: String,
}

pub(crate) fn validate() -> GeneratorValidationOutcome {
    let state = detect_generator_binary_state();
    let (is_ready, message) = match &state {
        GeneratorBinaryState::Available { path } => (
            true,
            format!(
                "Generador detectado en `{}`. Ya puedes continuar.",
                path.display()
            ),
        ),
        GeneratorBinaryState::Missing { searched_paths } => (
            false,
            format!(
                "No se encontro `gen` ni `gen.exe` en la raiz del proyecto. Colocalo en {} y presiona Enter para reintentar.",
                searched_paths
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(" o ")
            ),
        ),
        GeneratorBinaryState::Error(error) => (
            false,
            format!(
                "No fue posible validar el generador en la raiz del proyecto: {}",
                error
            ),
        ),
    };

    GeneratorValidationOutcome {
        state,
        is_ready,
        message,
    }
}

pub(crate) fn initial_message() -> String {
    match detect_generator_binary_state() {
        GeneratorBinaryState::Available { path } => format!(
            "Se encontro el generador en `{}`. Selecciona como quieres cargar la conexion.",
            path.display()
        ),
        GeneratorBinaryState::Missing { searched_paths } => format!(
            "No se encontro `gen` ni `gen.exe` en la raiz del proyecto. Colocalo en {} y presiona Enter para reintentar.",
            searched_paths
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(" o ")
        ),
        GeneratorBinaryState::Error(error) => format!(
            "No fue posible validar el generador en la raiz del proyecto: {}",
            error
        ),
    }
}

fn detect_generator_binary_state() -> GeneratorBinaryState {
    let current_dir = match env::current_dir() {
        Ok(path) => path,
        Err(error) => return GeneratorBinaryState::Error(error.to_string()),
    };

    let candidates = [current_dir.join("gen"), current_dir.join("gen.exe")];

    for candidate in &candidates {
        if candidate.is_file() {
            return GeneratorBinaryState::Available {
                path: candidate.clone(),
            };
        }
    }

    GeneratorBinaryState::Missing {
        searched_paths: candidates.into_iter().collect(),
    }
}
