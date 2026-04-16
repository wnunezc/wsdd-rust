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
//! Gestion del prerequisito PowerShell 7.5+.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::config::environment::env_config;
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
            "✗ PowerShell detectado ({version}) pero no cumple el minimo requerido."
        )));
    } else {
        let _ = tx.send(LogLine::warn(
            "✗ PowerShell 7.5+ no encontrado. Instalando con MSI oficial...",
        ));
    }

    let _ = tx.send(LogLine::info(
        "✗ Bootstrap de PowerShell: Windows PowerShell/curl.exe -> descarga MSI oficial -> msiexec.",
    ));

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
    let installer_path = installer_temp_path();

    download_supported_installer(&installer_path)?;
    run_msi_installer(&installer_path)?;

    let _ = std::fs::remove_file(&installer_path);
    Ok(())
}

fn download_supported_installer(installer_path: &Path) -> Result<()> {
    if installer_path.exists() {
        let _ = std::fs::remove_file(installer_path);
    }

    let url = installer_url();
    download_with_windows_powershell(&url, installer_path).or_else(|powershell_error| {
        download_with_curl(&url, installer_path).with_context(|| {
            format!(
                "✗ No se pudo descargar el MSI oficial ni con Windows PowerShell ni con curl.exe. Fallback original: {powershell_error}"
            )
        })
    })?;

    if !installer_path.exists() {
        return Err(anyhow!(
            "✗ El instalador de PowerShell no fue descargado: {}",
            installer_path.display()
        ));
    }

    Ok(())
}

fn download_with_windows_powershell(url: &str, installer_path: &Path) -> Result<()> {
    let command = format!(
        "$ProgressPreference='SilentlyContinue'; \
         [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; \
         Invoke-WebRequest -Uri '{}' -OutFile '{}'",
        ps_single_quote(url),
        ps_single_quote(&installer_path.display().to_string())
    );

    let mut cmd = Command::new(env_config().windows_powershell_exe());
    cmd.args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        &command,
    ]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let status = cmd
        .status()
        .context("✗ Error ejecutando Windows PowerShell para descargar PowerShell 7.5+")?;

    if !status.success() {
        return Err(anyhow!(
            "✗ Windows PowerShell devolvio exit code {} al descargar el MSI oficial",
            status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}

fn download_with_curl(url: &str, installer_path: &Path) -> Result<()> {
    let output_path = installer_path.display().to_string();
    let mut cmd = Command::new(env_config().curl_exe());
    cmd.args(["-L", "--fail", "--silent", "--show-error", "--output"]);
    cmd.arg(&output_path);
    cmd.arg(url);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let status = cmd
        .status()
        .context("✗ Error ejecutando curl.exe para descargar PowerShell 7.5+")?;

    if !status.success() {
        return Err(anyhow!(
            "✗ curl.exe devolvio exit code {} al descargar el MSI oficial",
            status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}

fn run_msi_installer(installer_path: &Path) -> Result<()> {
    let path = installer_path.display().to_string();
    let mut cmd = Command::new(env_config().msiexec_exe());
    cmd.args([
        "/i",
        &path,
        "/qn",
        "/norestart",
        "ADD_PATH=1",
        "REGISTER_MANIFEST=1",
    ]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let status = cmd
        .status()
        .context("✗ Error ejecutando msiexec para instalar PowerShell 7.5+")?;

    if !status.success() {
        return Err(anyhow!(
            "✗ msiexec devolvio exit code {} al instalar PowerShell 7.5+",
            status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}

fn installer_url() -> String {
    let version = MIN_SUPPORTED_PWSH_VERSION;
    format!(
        "{}/v{version}/PowerShell-{version}-win-{}.msi",
        env_config().powershell_release_base_url(),
        installer_arch()
    )
}

fn installer_temp_path() -> PathBuf {
    std::env::temp_dir().join(format!(
        "PowerShell-{}-win-{}.msi",
        MIN_SUPPORTED_PWSH_VERSION,
        installer_arch()
    ))
}

fn installer_arch() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "arm64",
        "x86" => "x86",
        _ => "x64",
    }
}

fn ps_single_quote(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installer_url_targets_official_release_asset() {
        let url = installer_url();
        assert!(url.contains("github.com/PowerShell/PowerShell/releases/download"));
        assert!(url.contains("PowerShell-7.5.0-win-"));
        assert!(url.ends_with(".msi"));
    }

    #[test]
    fn ps_single_quote_escapes_embedded_quotes() {
        assert_eq!(ps_single_quote("o'hara"), "o''hara");
    }
}
