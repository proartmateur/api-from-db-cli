use crate::core::error::AppError;
use crate::core::ports::Clipboard;

/// Adapter que usa el portapapeles del sistema via `arboard`.
///
/// Si no hay display disponible (SSH headless, entorno sin Wayland/X11),
/// cae a `StdoutClipboard` para que el usuario pueda copiar manualmente.
/// Cumple RN-013: "si no es posible copiarlo, debera mostrarlo en pantalla".
#[derive(Debug, Default, Clone, Copy)]
pub struct ArboardClipboard;

impl Clipboard for ArboardClipboard {
    fn copy(&self, value: &str) -> Result<(), AppError> {
        match try_system_clipboard(value) {
            Ok(()) => Ok(()),
            Err(error) => {
                // Fallback: mostrar en stdout para que el usuario copie manualmente.
                println!("{value}");
                Err(AppError::Process(format!(
                    "no fue posible usar el portapapeles del sistema: {error}. \
                     El contenido se mostro en stdout para copia manual."
                )))
            }
        }
    }
}

fn try_system_clipboard(value: &str) -> Result<(), String> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|error| error.to_string())?;
    clipboard
        .set_text(value.to_string())
        .map_err(|error| error.to_string())
}