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
//! Gestión de certificados SSL locales via mkcert.
//!
//! Equivalente a `Handlers/HandlerMKCert.cs` en la versión C#.

use anyhow::Result;
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::ps_script::PsRunner;
use crate::handlers::ps_script::ScriptRunner;

// ─── Sondas ───────────────────────────────────────────────────────────────────

/// Verifica si mkcert está disponible en el sistema.
pub fn is_installed() -> bool {
    let mut cmd = Command::new("mkcert");
    cmd.arg("--version");
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.output().map(|o| o.status.success()).unwrap_or(false)
}

// ─── Instalación y certificados ───────────────────────────────────────────────

/// Instala mkcert via Chocolatey.
pub fn install() -> Result<()> {
    super::chocolatey::install("mkcert")
}

/// Instala la autoridad certificadora local (`mkcert -install`).
///
/// Debe ejecutarse una vez tras instalar mkcert para que los
/// certificados generados sean confiados por el sistema.
pub fn generate_ca() -> Result<()> {
    let mut cmd = Command::new("mkcert");
    cmd.arg("-install");
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.status()?;
    Ok(())
}

/// Genera un certificado SSL para el dominio dado.
///
/// Los archivos `key.pem` y `cert.pem` se guardan en
/// `C:\WSDD-Environment\certs\{domain}\`.
pub fn generate(domain: &str) -> Result<()> {
    let output_dir = format!(r"C:\WSDD-Environment\certs\{domain}");
    std::fs::create_dir_all(&output_dir)?;
    let mut cmd = Command::new("mkcert");
    cmd.args(["-key-file", &format!("{output_dir}\\key.pem")])
        .args(["-cert-file", &format!("{output_dir}\\cert.pem")])
        .arg(domain);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.status()?;
    Ok(())
}

// ─── Requirements (Fase 3) ────────────────────────────────────────────────────

/// Verifica e instala mkcert si es necesario, incluida la CA local.
///
/// Instala via Chocolatey si `mkcert` no se encuentra, luego ejecuta
/// `mkcert -install` para registrar la CA en el sistema.
/// Debe llamarse desde un hilo separado (no en el render loop de egui).
///
/// Retorna `true` si mkcert está disponible al finalizar.
pub fn process_requirements(tx: &LogSender) -> bool {
    let _ = tx.send(LogLine::info("Verificando mkcert..."));

    if is_installed() {
        let _ = tx.send(LogLine::success("✓ mkcert está instalado"));
        return true;
    }

    let _ = tx.send(LogLine::warn(
        "mkcert no encontrado. Instalando via Chocolatey...",
    ));

    let runner = PsRunner::new();
    match runner.run_ps_sync("choco install mkcert -y", None, None) {
        Ok(_) => {
            if !is_installed() {
                let _ = tx.send(LogLine::error("✗ mkcert no responde tras la instalación"));
                return false;
            }
            let _ = tx.send(LogLine::success("✓ mkcert instalado correctamente"));

            // Registrar la CA local en el sistema
            let _ = tx.send(LogLine::info("Instalando autoridad certificadora local..."));
            match generate_ca() {
                Ok(_) => {
                    let _ = tx.send(LogLine::success("✓ CA local instalada"));
                }
                Err(e) => {
                    let _ = tx.send(LogLine::warn(format!("⚠ CA local: {e}")));
                }
            }
            true
        }
        Err(e) => {
            let _ = tx.send(LogLine::error(format!("✗ Error instalando mkcert: {e}")));
            false
        }
    }
}
