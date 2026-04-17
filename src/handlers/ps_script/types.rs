use std::path::Path;

use crate::errors::InfraError;
use crate::handlers::log_types::OutputSender;

/// Complete captured output from an external process.
#[derive(Debug, Clone, Default)]
pub struct ProcOutput {
    /// Combined stdout and stderr text with ANSI escapes removed.
    pub text: String,
    /// True when the process exits with code 0.
    pub success: bool,
}

impl ProcOutput {
    /// Checks whether the process output contains an expected token.
    pub fn contains(&self, token: &str) -> bool {
        self.text.contains(token)
    }
}

/// Contract for running PowerShell scripts and direct system commands.
pub trait ScriptRunner: Send + Sync {
    /// Runs a `.ps1` script with a process-scoped unrestricted execution policy.
    ///
    /// # Errors
    /// Returns [`InfraError::ScriptFailed`] if the script cannot be launched.
    fn run_script_sync(
        &self,
        script_name: &str,
        work_dir: Option<&Path>,
        tx: Option<&OutputSender>,
    ) -> Result<ProcOutput, InfraError>;

    /// Runs an arbitrary PowerShell command.
    ///
    /// # Errors
    /// Returns [`InfraError::Io`] if PowerShell cannot be launched.
    fn run_ps_sync(
        &self,
        command: &str,
        work_dir: Option<&Path>,
        tx: Option<&OutputSender>,
    ) -> Result<ProcOutput, InfraError>;

    /// Runs a direct process without PowerShell.
    ///
    /// # Errors
    /// Returns [`InfraError::ProcessNotFound`] if the process cannot be launched.
    fn run_direct_sync(
        &self,
        program: &str,
        args: &[&str],
        work_dir: Option<&Path>,
        tx: Option<&OutputSender>,
    ) -> Result<ProcOutput, InfraError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proc_output_contains() {
        let out = ProcOutput {
            text: "Docker is Installed".to_string(),
            success: true,
        };
        assert!(out.contains("Installed"));
        assert!(!out.contains("Running"));
    }
}
