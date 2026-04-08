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
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::ps_script::PsRunner;
use crate::handlers::ps_script::ScriptRunner;

// ─── Sondas ───────────────────────────────────────────────────────────────────

/// Verifica si Chocolatey está disponible en el sistema.
pub fn is_installed() -> bool {
    let mut cmd = Command::new("choco");
    cmd.arg("--version");
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.output().map(|o| o.status.success()).unwrap_or(false)
}

// ─── Instalación ──────────────────────────────────────────────────────────────

/// Instala un paquete via Chocolatey.
pub fn install(package: &str) -> Result<()> {
    let mut cmd = Command::new("choco");
    cmd.args(["install", package, "-y"]);
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

    let runner = PsRunner::new();

    // Ejecutar en batch: el script puede tardar varios segundos
    match runner.run_ps_sync(ps_cmd, None, None) {
        Ok(_) => {
            if is_installed() {
                let _ = tx.send(LogLine::success("✓ Chocolatey instalado correctamente"));
                true
            } else {
                let _ = tx.send(LogLine::error(
                    "✗ Chocolatey no responde tras la instalación",
                ));
                false
            }
        }
        Err(e) => {
            let _ = tx.send(LogLine::error(format!(
                "✗ Error instalando Chocolatey: {e}"
            )));
            false
        }
    }
}
