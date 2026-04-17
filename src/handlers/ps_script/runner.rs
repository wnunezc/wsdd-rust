use std::path::{Path, PathBuf};

use crate::config::environment::MIN_SUPPORTED_PWSH_VERSION;
use crate::errors::InfraError;
use crate::handlers::log_types::OutputSender;

use super::process::{exec_direct, exec_powershell};
use super::pwsh::{current_pwsh_version, supported_pwsh_executable};
use super::types::{ProcOutput, ScriptRunner};

/// Real PowerShell script runner used in production.
#[derive(Clone, Debug)]
pub struct PsRunner {
    scripts_dir: PathBuf,
}

impl PsRunner {
    /// Creates a runner pointing at the standard WSDD scripts directory.
    pub fn new() -> Self {
        Self {
            scripts_dir: super::scripts_dir(),
        }
    }

    /// Creates a runner with a custom scripts directory.
    pub fn with_scripts_dir(dir: PathBuf) -> Self {
        Self { scripts_dir: dir }
    }
}

impl Default for PsRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptRunner for PsRunner {
    fn run_script_sync(
        &self,
        script_name: &str,
        work_dir: Option<&Path>,
        tx: Option<&OutputSender>,
    ) -> Result<ProcOutput, InfraError> {
        let dir = work_dir.unwrap_or(&self.scripts_dir);
        let path = dir.join(script_name);
        let ps_exe = supported_pwsh_executable().ok_or_else(|| {
            let current = current_pwsh_version().unwrap_or_else(|| "not found".to_string());
            InfraError::PrerequisiteNotMet(format!(
                "PowerShell {MIN_SUPPORTED_PWSH_VERSION}+ is required to run PS1 scripts (found: {current})"
            ))
        })?;

        tracing::debug!(script = script_name, dir = %dir.display(), "Running PS1 script");

        let policy = "Set-ExecutionPolicy -Scope Process -ExecutionPolicy Unrestricted -Force";
        let invoke = format!("& '{}'", path.display());
        let ps_arg = format!(
            "$OutputEncoding = [System.Text.Encoding]::UTF8 ; \
             [Console]::OutputEncoding = [System.Text.Encoding]::UTF8 ; \
             {policy} ; {invoke}"
        );

        exec_direct(
            &ps_exe,
            &[
                "-NoLogo".to_string(),
                "-NoProfile".to_string(),
                "-NonInteractive".to_string(),
                "-Command".to_string(),
                ps_arg,
            ],
            Some(dir),
            tx,
        )
        .map_err(|e| InfraError::ScriptFailed(script_name.to_string(), e.to_string()))
    }

    fn run_ps_sync(
        &self,
        command: &str,
        work_dir: Option<&Path>,
        tx: Option<&OutputSender>,
    ) -> Result<ProcOutput, InfraError> {
        tracing::debug!(command = %command, "Running PowerShell command");

        let policy = "Set-ExecutionPolicy -Scope Process -ExecutionPolicy Unrestricted -Force";
        let ps_arg = format!("{policy} ; {command}");
        exec_powershell(&ps_arg, work_dir, tx)
    }

    fn run_direct_sync(
        &self,
        program: &str,
        args: &[&str],
        work_dir: Option<&Path>,
        tx: Option<&OutputSender>,
    ) -> Result<ProcOutput, InfraError> {
        tracing::debug!(program = program, ?args, "Running direct process");

        let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        exec_direct(program, &args_owned, work_dir, tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ps_runner_default_scripts_dir() {
        let runner = PsRunner::new();
        assert!(runner.scripts_dir.to_string_lossy().contains("PS-Script"));
    }

    #[test]
    fn ps_runner_custom_scripts_dir() {
        let custom = PathBuf::from(r"C:\custom\scripts");
        let runner = PsRunner::with_scripts_dir(custom.clone());
        assert_eq!(runner.scripts_dir, custom);
    }
}
