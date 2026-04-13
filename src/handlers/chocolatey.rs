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
//! Gestión de Chocolatey — verificación e instalación.
//!
//! Equivalente a `Handlers/HandlerChocolatey.cs` en la versión C#.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

use crate::handlers::log_types::{LogLine, LogSender};

const DEFAULT_CHOCO_EXE: &str = r"C:\ProgramData\chocolatey\bin\choco.exe";

// ─── Sondas ───────────────────────────────────────────────────────────────────

/// Verifica si Chocolatey está disponible en el sistema.
pub fn is_installed() -> bool {
    let Some(choco_exe) = resolve_choco_exe() else {
        return false;
    };

    let mut cmd = Command::new(choco_exe);
    cmd.arg("--version");
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.output().map(|o| o.status.success()).unwrap_or(false)
}

// ─── Instalación ──────────────────────────────────────────────────────────────

/// Instala un paquete via Chocolatey.
pub fn install(package: &str) -> Result<()> {
    let choco_exe =
        resolve_choco_exe().context("Chocolatey no está disponible en la sesión actual")?;
    let mut cmd = Command::new(choco_exe);
    cmd.args(["install", package, "-y", "--no-progress"]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.status()
        .context(format!("Error instalando {package} via choco"))?;
    Ok(())
}

// ─── Requirements (Fase 3) ────────────────────────────────────────────────────

/// Verifica e instala Chocolatey si es necesario.
///
/// Ejecuta el script de instalación oficial si `choco` no se encuentra.
/// Debe llamarse desde un hilo separado (no en el render loop de egui).
///
/// Retorna `true` si Chocolatey está disponible al finalizar.
pub fn process_requirements(tx: &LogSender) -> bool {
    let _ = tx.send(LogLine::info("Verificando Chocolatey..."));

    if is_installed() {
        let _ = tx.send(LogLine::success("✓ Chocolatey está instalado"));
        return true;
    }

    let _ = tx.send(LogLine::warn("Chocolatey no encontrado. Instalando..."));

    // Script de instalación oficial de Chocolatey
    // https://community.chocolatey.org/install.ps1
    let ps_cmd = concat!(
        "Set-ExecutionPolicy Bypass -Scope Process -Force; ",
        "[System.Net.ServicePointManager]::SecurityProtocol = ",
        "[System.Net.ServicePointManager]::SecurityProtocol -bor 3072; ",
        "iex ((New-Object System.Net.WebClient)",
        ".DownloadString('https://community.chocolatey.org/install.ps1'))"
    );

    match install_chocolatey(ps_cmd) {
        Ok(_) if is_installed() => {
            let _ = tx.send(LogLine::success("✓ Chocolatey instalado correctamente"));
            true
        }
        Ok(_) => {
            let _ = tx.send(LogLine::error(
                "✗ Chocolatey no responde tras la instalación",
            ));
            false
        }
        Err(e) => {
            let _ = tx.send(LogLine::error(format!(
                "✗ Error instalando Chocolatey: {e}"
            )));
            false
        }
    }
}

fn install_chocolatey(command: &str) -> Result<()> {
    let mut cmd = Command::new("powershell.exe");
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        command,
    ]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let status = cmd
        .status()
        .context("Error ejecutando instalador oficial de Chocolatey")?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Chocolatey devolvio exit code {}",
            status.code().unwrap_or(-1)
        ))
    }
}

fn resolve_choco_exe() -> Option<PathBuf> {
    std::env::var_os("ChocolateyInstall")
        .map(PathBuf::from)
        .map(|base| base.join("bin").join("choco.exe"))
        .filter(|path| path.is_file())
        .or_else(|| {
            let path = PathBuf::from(DEFAULT_CHOCO_EXE);
            path.is_file().then_some(path)
        })
        .or_else(|| resolve_from_path("choco.exe"))
}

fn resolve_from_path(program: &str) -> Option<PathBuf> {
    let mut cmd = Command::new("where.exe");
    cmd.arg(program);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(PathBuf::from)
}
