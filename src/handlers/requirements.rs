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
//! Verificación y aseguramiento de requisitos del sistema.
//!
//! Equivalente a `Handlers/Requirement.cs` en la versión C#.
//!
//! # Responsabilidades
//! - `ensure_admin`: elevar privilegios UAC al arranque
//! - `run_requirements`: orquestar el check de Docker/Choco/MKCert (Fase 3)

use std::sync::mpsc;

use anyhow::Result;

use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::setting::AppSettings;
use crate::handlers::{chocolatey, docker_deploy, hosts, mkcert};

// ─── Resultado del proceso de requirements ────────────────────────────────────

/// Resultado del proceso completo de verificación de requisitos.
///
/// Enviado por `run_requirements` al canal `outcome_tx` cuando termina.
#[derive(Debug, Clone, PartialEq)]
pub enum LoaderOutcome {
    /// Todos los requisitos OK — primer arranque: mostrar botón "Continuar".
    DoneWithContinue,
    /// Todos los requisitos OK — arranque posterior: avanzar a Main silenciosamente.
    AllDone,
    /// Docker instalado por primera vez o actualizado — el sistema necesita reinicio.
    NeedsReboot,
    /// Error bloqueante — Docker no está instalado en el sistema.
    BlockingError,
}

// ─── Proceso de requirements (Fase 3) ────────────────────────────────────────

/// Ejecuta el proceso completo de verificación e instalación de requisitos.
///
/// **Debe ejecutarse en un hilo separado** — contiene operaciones bloqueantes
/// (scripts PowerShell, instalaciones via choco) que no pueden correr en el
/// render loop de egui.
///
/// # Flujo
/// 1. Docker (bloqueante si no instalado)
/// 2. Chocolatey (instala si falta)
/// 3. MKCert (instala via choco si falta, genera CA)
/// 4. Envía `LoaderOutcome` al canal `outcome_tx`
///
/// # Parámetros
/// - `log_tx`: canal para enviar líneas de log a la UI en tiempo real
/// - `outcome_tx`: canal para el resultado final (controla botones del Loader)
/// - `first_run`: `true` si viene de Welcome → Comenzar (muestra botón Continuar)
pub fn run_requirements(
    log_tx: LogSender,
    outcome_tx: mpsc::Sender<LoaderOutcome>,
    first_run: bool,
) {
    let runner = crate::handlers::ps_script::PsRunner::new();

    // Cargar settings al inicio — se guardan al final solo si todo OK
    let mut settings = AppSettings::load().unwrap_or_default();

    // ── 1. Docker ──────────────────────────────────────────────────────────
    let docker_outcome = docker_deploy::process_requirements_sync(&runner, &log_tx);
    if docker_outcome != docker_deploy::DockerRequirementOutcome::Ready {
        let _ = outcome_tx.send(LoaderOutcome::BlockingError);
        return;
    }

    // ── 2. Chocolatey ──────────────────────────────────────────────────────
    let _ = tx_log_separator(&log_tx);
    if !chocolatey::process_requirements(&log_tx) {
        let _ = outcome_tx.send(LoaderOutcome::BlockingError);
        return;
    }

    // ── 3. MKCert ──────────────────────────────────────────────────────────
    let _ = tx_log_separator(&log_tx);
    if !mkcert::process_requirements(&log_tx) {
        let _ = outcome_tx.send(LoaderOutcome::BlockingError);
        return;
    }

    // ── 4. Deploy Environment ─────────────────────────────────────────────
    let _ = tx_log_separator(&log_tx);
    let _ = log_tx.send(LogLine::info("Inicializando entorno Docker..."));
    if let Err(e) = docker_deploy::deploy_environment_sync(&runner, &log_tx) {
        let _ = log_tx.send(LogLine::error(format!(
            "✗ Error al inicializar entorno: {e}"
        )));
        let _ = outcome_tx.send(LoaderOutcome::BlockingError);
        return;
    }

    // ── 5. Hosts ──────────────────────────────────────────────────────────
    let _ = tx_log_separator(&log_tx);
    if let Err(e) = hosts::update_host(None, &log_tx) {
        let _ = log_tx.send(LogLine::error(format!(
            "✗ Error al actualizar archivo hosts: {e:#}"
        )));
        let _ = outcome_tx.send(LoaderOutcome::BlockingError);
        return;
    }

    // ── Guardar setup completado ──────────────────────────────────────────
    // Solo se guarda si TODOS los pasos anteriores completaron sin error
    settings.setup_completed = true;
    if let Err(e) = settings.save() {
        tracing::error!("No se pudo guardar setup_completed: {e}");
        // No es bloqueante para la UI — en el próximo arranque re-ejecutará el deploy
    }

    // ── Resultado final ────────────────────────────────────────────────────
    let _ = tx_log_separator(&log_tx);
    let _ = log_tx.send(LogLine::success("✓ Sistema listo para WSDD"));

    let outcome = if first_run {
        LoaderOutcome::DoneWithContinue
    } else {
        LoaderOutcome::AllDone
    };
    let _ = outcome_tx.send(outcome);
}

fn tx_log_separator(tx: &LogSender) -> Result<(), mpsc::SendError<LogLine>> {
    tx.send(LogLine::info(
        "─────────────────────────────────────────────",
    ))
}

// ─── Admin / UAC ──────────────────────────────────────────────────────────────

/// Verifica que la aplicacion se ejecuta con privilegios de administrador.
/// Si no, relanza el proceso elevado (UAC).
#[cfg(windows)]
pub fn ensure_admin() -> Result<()> {
    use windows::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    let elevated = unsafe {
        let mut token = windows::Win32::Foundation::HANDLE::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)?;

        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
        GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            size,
            &mut size,
        )?;
        elevation.TokenIsElevated != 0
    };

    if !elevated {
        relaunch_as_admin()?;
        std::process::exit(0);
    }
    Ok(())
}

#[cfg(windows)]
fn relaunch_as_admin() -> Result<()> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let exe = std::env::current_exe()?;
    let exe_wide: Vec<u16> = OsStr::new(exe.as_os_str())
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let verb: Vec<u16> = OsStr::new("runas")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        ShellExecuteW(
            None,
            PCWSTR(verb.as_ptr()),
            PCWSTR(exe_wide.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        );
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn ensure_admin() -> Result<()> {
    Ok(())
}
