// WebStack Deployer for Docker
// Copyright (c) 2026 Walter Nunez / Icaros Net S.A
// All Rights Reserved.
//
// This software is provided for development use only.
// Unauthorized commercial use is prohibited.
//
// Redistribution and modification allowed only through
// the official GitHub repository.
//
// This software is provided AS IS, without warranty of any kind.
// The author shall not be liable for any damages.
//
// Contact: wnunez@lh-2.net
//! PowerShell script execution facade.
//!
//! This module keeps the public API used by handlers and UI code while the
//! implementation is split into focused submodules.

use std::path::PathBuf;

use crate::config::environment::{env_config, path_config, DEFAULT_WSDD_ENV};
use crate::errors::InfraError;

mod launch;
mod process;
mod pwsh;
mod runner;
mod types;

// Re-export channel types from log_types as a public facade.
// This lets callers import them from ps_script without knowing log_types.
#[allow(unused_imports)]
pub use crate::handlers::log_types::{LogLevel, LogLine, LogSender, OutputSender};
pub use launch::{launch, launch_shell_window, launch_url};
#[allow(unused_imports)]
pub use process::strip_ansi;
#[allow(unused_imports)]
pub use pwsh::{current_pwsh_version, has_supported_pwsh, supported_pwsh_executable};
pub use runner::PsRunner;
pub use types::{ProcOutput, ScriptRunner};

/// Root directory of the extracted WSDD environment.
pub const WSDD_ENV: &str = DEFAULT_WSDD_ENV;
pub use crate::config::environment::MIN_SUPPORTED_PWSH_VERSION;

/// Win32 process creation flag that hides child process windows.
#[cfg(windows)]
pub(super) const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Returns the WSDD environment root directory.
pub fn env_dir() -> PathBuf {
    path_config().environment_root().to_path_buf()
}

/// Returns the WSDD PowerShell scripts directory.
pub fn scripts_dir() -> PathBuf {
    path_config().scripts_dir()
}

/// Returns the WSDD Docker structure directory.
pub fn docker_structure_dir() -> PathBuf {
    path_config().docker_structure_dir()
}

/// Runs a WSDD PowerShell script asynchronously.
///
/// # Errors
/// Returns [`InfraError::ScriptFailed`] if PowerShell cannot start or the script fails.
pub async fn run_script(
    script_name: &str,
    work_dir: Option<PathBuf>,
    tx: Option<OutputSender>,
) -> Result<ProcOutput, InfraError> {
    let name = script_name.to_string();
    let runner = PsRunner::new();

    tokio::task::spawn_blocking(move || {
        runner.run_script_sync(&name, work_dir.as_deref(), tx.as_ref())
    })
    .await
    .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?
}

/// Runs an arbitrary PowerShell command asynchronously.
///
/// # Errors
/// Returns [`InfraError::Io`] if PowerShell cannot start.
pub async fn run_ps_command(
    command: &str,
    work_dir: Option<PathBuf>,
    tx: Option<OutputSender>,
) -> Result<ProcOutput, InfraError> {
    let cmd = command.to_string();
    let runner = PsRunner::new();

    tokio::task::spawn_blocking(move || runner.run_ps_sync(&cmd, work_dir.as_deref(), tx.as_ref()))
        .await
        .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?
}

/// Runs Docker directly without routing through PowerShell.
///
/// # Errors
/// Returns [`InfraError::ProcessNotFound`] if Docker cannot be launched.
pub async fn run_docker(
    args: Vec<String>,
    tx: Option<OutputSender>,
) -> Result<ProcOutput, InfraError> {
    tokio::task::spawn_blocking(move || {
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let runner = PsRunner::new();
        runner.run_direct_sync(env_config().docker_exe(), &refs, None, tx.as_ref())
    })
    .await
    .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?
}
