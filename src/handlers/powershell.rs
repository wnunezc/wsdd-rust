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
//! Gestión del prerequisito PowerShell 7.5+.

use anyhow::{anyhow, Context, Result};
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::ps_script::{
    current_pwsh_version, has_supported_pwsh, MIN_SUPPORTED_PWSH_VERSION,
};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn process_requirements(tx: &LogSender) -> bool {
    let _ = tx.send(LogLine::info(format!(
        "Verificando PowerShell {MIN_SUPPORTED_PWSH_VERSION}+..."
    )));

    if has_supported_pwsh() {
        let version =
            current_pwsh_version().unwrap_or_else(|| MIN_SUPPORTED_PWSH_VERSION.to_string());
        let _ = tx.send(LogLine::success(format!(
            "✓ PowerShell compatible detectado ({version})"
        )));
        return true;
    }

    if let Some(version) = current_pwsh_version() {
        let _ = tx.send(LogLine::warn(format!(
            "PowerShell detectado ({version}) pero no cumple el minimo requerido."
        )));
    } else {
        let _ = tx.send(LogLine::warn(
            "PowerShell 7.5+ no encontrado. Instalando via Chocolatey...",
        ));
    }

    match install_supported_version() {
        Ok(_) if has_supported_pwsh() => {
            let version =
                current_pwsh_version().unwrap_or_else(|| MIN_SUPPORTED_PWSH_VERSION.to_string());
            let _ = tx.send(LogLine::success(format!(
                "✓ PowerShell actualizado a una version compatible ({version})"
            )));
            true
        }
        Ok(_) => {
            let _ = tx.send(LogLine::error(format!(
                "✗ PowerShell sigue sin cumplir {MIN_SUPPORTED_PWSH_VERSION}+ tras la instalacion"
            )));
            false
        }
        Err(e) => {
            let _ = tx.send(LogLine::error(format!(
                "✗ Error instalando PowerShell 7.5+: {e}"
            )));
            false
        }
    }
}

fn install_supported_version() -> Result<()> {
    let mut cmd = Command::new("choco");
    cmd.args([
        "upgrade",
        "powershell-core",
        "--version=7.5.0",
        "-y",
        "--no-progress",
        "--force",
    ]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let status = cmd
        .status()
        .context("Error ejecutando Chocolatey para instalar PowerShell")?;

    if !status.success() {
        return Err(anyhow!(
            "Chocolatey devolvio exit code {}",
            status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}
