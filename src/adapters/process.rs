use std::process::Command;

use crate::core::error::AppError;
use crate::core::ports::ProcessRunner;
use crate::domain::{GeneratedCommand, ProcessResult};

#[derive(Debug, Default, Clone, Copy)]
pub struct StdProcessRunner;

impl ProcessRunner for StdProcessRunner {
    fn run(&self, command: &GeneratedCommand) -> Result<ProcessResult, AppError> {
        let output = Command::new(&command.executable)
            .args(&command.arguments)
            .output()
            .map_err(|error| AppError::Process(error.to_string()))?;

        Ok(ProcessResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            success: output.status.success(),
        })
    }
}
