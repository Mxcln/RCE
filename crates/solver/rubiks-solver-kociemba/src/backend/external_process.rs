use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use super::{BackendError, TwoPhaseBackend};

#[derive(Clone, Debug)]
pub(crate) struct ExternalProcessBackend {
    program: PathBuf,
    args: Vec<String>,
}

impl ExternalProcessBackend {
    pub(crate) fn new(program: PathBuf, args: Vec<String>) -> Self {
        Self { program, args }
    }
}

impl TwoPhaseBackend for ExternalProcessBackend {
    fn id(&self) -> &'static str {
        "kociemba-external-process"
    }

    fn solve_facelets(
        &self,
        cubestring: &str,
        _max_depth: u8,
        timeout: Option<Duration>,
    ) -> Result<String, BackendError> {
        if timeout.is_some() {
            return Err(BackendError::Failure {
                backend: self.id(),
                message: "external process backend does not yet implement timeout handling"
                    .to_string(),
            });
        }

        let output = Command::new(&self.program)
            .args(&self.args)
            .arg(cubestring)
            .output()
            .map_err(|err| BackendError::Unavailable {
                backend: self.id(),
                message: err.to_string(),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let message = if !stderr.is_empty() {
                stderr
            } else if !stdout.is_empty() {
                stdout
            } else {
                format!("process exited with status {}", output.status)
            };

            return Err(BackendError::Failure {
                backend: self.id(),
                message,
            });
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            return Err(BackendError::Failure {
                backend: self.id(),
                message: "external process returned an empty solution".to_string(),
            });
        }

        Ok(text)
    }
}
