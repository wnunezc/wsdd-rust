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
//! Verificacion y aseguramiento de requisitos del sistema.
//!
//! Equivalente a `Handlers/Requirement.cs` en la version C#.
//!
//! # Responsabilidades
//! - `ensure_admin`: elevar privilegios UAC al arranque
//! - `run_requirements`: orquestar el check de PowerShell/Choco/MKCert/Docker

use std::sync::mpsc;

use anyhow::Result;

use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::setting::AppSettings;
use crate::handlers::{chocolatey, docker_deploy, hosts, mkcert, powershell};

/// Resultado del proceso completo de verificacion de requisitos.
#[derive(Debug, Clone, PartialEq)]
pub enum LoaderOutcome {
    /// Todos los requisitos OK y es primer arranque: mostrar boton Continuar.
    DoneWithContinue,
    /// Todos los requisitos OK en arranque posterior: ir a Main silenciosamente.
    AllDone,
    /// Reservado para instalaciones que requieran reinicio del sistema.
    NeedsReboot,
    /// Error bloqueante: la app no puede continuar.
    BlockingError,
}

/// Ejecuta el proceso completo de verificacion e instalacion de requisitos.
///
/// Debe ejecutarse fuera del render loop de egui porque contiene operaciones
/// bloqueantes (instaladores, scripts PowerShell, Docker, mkcert).
///
/// Flujo:
/// 1. PowerShell 7.5+
/// 2. Chocolatey
/// 3. MKCert
/// 4. Docker
/// 5. Deploy Environment
/// 6. Hosts
pub fn run_requirements(
    log_tx: LogSender,
    outcome_tx: mpsc::Sender<LoaderOutcome>,
    first_run: bool,
) {
    let runner = crate::handlers::ps_script::PsRunner::new();

    let mut settings = match AppSettings::load() {
        Ok(settings) => settings,
        Err(e) => {
            let _ = log_tx.send(LogLine::error(format!(
                "Configuracion incompatible o invalida: {e}"
            )));
            let _ = outcome_tx.send(LoaderOutcome::BlockingError);
            return;
        }
    };

    if !powershell::process_requirements(&log_tx) {
        let _ = outcome_tx.send(LoaderOutcome::BlockingError);
        return;
    }

    let _ = tx_log_separator(&log_tx);
    if !chocolatey::process_requirements(&log_tx) {
        let _ = outcome_tx.send(LoaderOutcome::BlockingError);
        return;
    }

    let _ = tx_log_separator(&log_tx);
    if !mkcert::process_requirements(&log_tx) {
        let _ = outcome_tx.send(LoaderOutcome::BlockingError);
        return;
    }

    let _ = tx_log_separator(&log_tx);
    let docker_outcome = docker_deploy::process_requirements_sync(&runner, &log_tx);
    if docker_outcome != docker_deploy::DockerRequirementOutcome::Ready {
        let _ = outcome_tx.send(LoaderOutcome::BlockingError);
        return;
    }

    let _ = tx_log_separator(&log_tx);
    if let Err(e) = settings.validate_prerequisite_credentials() {
        let _ = log_tx.send(LogLine::error(format!(
            "Credenciales incompletas para MySQL/phpMyAdmin: {e}"
        )));
        let _ = log_tx.send(LogLine::error(
            "Completa la configuracion inicial antes de desplegar los prerequisitos.",
        ));
        let _ = outcome_tx.send(LoaderOutcome::BlockingError);
        return;
    }

    if let Err(e) = docker_deploy::sync_prerequisite_compose_sync(&settings) {
        let _ = log_tx.send(LogLine::error(format!(
            "Error al preparar init.yml con credenciales: {e}"
        )));
        let _ = outcome_tx.send(LoaderOutcome::BlockingError);
        return;
    }

    let _ = log_tx.send(LogLine::info("Inicializando entorno Docker..."));
    if let Err(e) = docker_deploy::deploy_environment_sync(&runner, &log_tx) {
        let _ = log_tx.send(LogLine::error(format!("Error al inicializar entorno: {e}")));
        let _ = outcome_tx.send(LoaderOutcome::BlockingError);
        return;
    }

    let _ = tx_log_separator(&log_tx);
    if let Err(e) = hosts::update_host(None, &log_tx) {
        let _ = log_tx.send(LogLine::error(format!(
            "Error al actualizar archivo hosts: {e:#}"
        )));
        let _ = outcome_tx.send(LoaderOutcome::BlockingError);
        return;
    }

    settings.setup_completed = true;
    if let Err(e) = settings.save() {
        tracing::error!("No se pudo guardar setup_completed: {e}");
    }

    let _ = tx_log_separator(&log_tx);
    let _ = log_tx.send(LogLine::success("Sistema listo para WSDD"));

    let outcome = if first_run {
        LoaderOutcome::DoneWithContinue
    } else {
        LoaderOutcome::AllDone
    };
    let _ = outcome_tx.send(outcome);
}

fn tx_log_separator(tx: &LogSender) -> Result<(), mpsc::SendError<LogLine>> {
    tx.send(LogLine::info(
        "---------------------------------------------",
    ))
}

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
