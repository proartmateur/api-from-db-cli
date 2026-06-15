use crate::core::error::AppError;
use crate::core::ports::Clipboard;

#[derive(Debug, Default, Clone, Copy)]
pub struct StdoutClipboard;

impl Clipboard for StdoutClipboard {
    fn copy(&self, value: &str) -> Result<(), AppError> {
        println!("{value}");
        Ok(())
    }
}
