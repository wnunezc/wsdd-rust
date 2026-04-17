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
//! Base Docker environment deployment orchestration.
//!
//! Public callers keep using this module as the stable facade for Docker
//! requirements, base environment deployment, managed templates, and log
//! bridges. Implementation details live in focused submodules.

mod base;
mod progress;
mod templates;

pub use progress::{make_docker_progress_bridge, make_log_bridge};

use crate::config::environment::path_config;
use crate::errors::InfraError;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::ps_script::{OutputSender, PsRunner, ScriptRunner};
use crate::handlers::setting::AppSettings;
use crate::models::project::PhpVersion;

/// Result of Docker Desktop requirement evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum DockerRequirementOutcome {
    /// Docker is installed, configured, and running.
    Ready,
    /// Docker is not installed and WSDD cannot continue.
    NotInstalled,
    /// Docker is installed but did not reach a usable running state.
    StartupFailed,
}

/// Verifies and initializes Docker Desktop synchronously.
///
/// This is called by the loader from a background job, never directly from the
/// egui render loop.
pub fn process_requirements_sync(runner: &PsRunner, tx: &LogSender) -> DockerRequirementOutcome {
    let _ = tx.send(LogLine::info("Verificando Docker Desktop..."));

    let installed = runner
        .run_script_sync("dd-isinstalled.ps1", None, None)
        .map(|o| o.contains("Installed"))
        .unwrap_or(false);

    if !installed {
        let _ = tx.send(LogLine::error("✗ Docker Desktop no está instalado."));
        let _ = tx.send(LogLine::error(
            "  Descárgalo desde https://www.docker.com/products/docker-desktop",
        ));
        return DockerRequirementOutcome::NotInstalled;
    }
    let _ = tx.send(LogLine::success("✓ Docker Desktop está instalado"));

    let configured = runner
        .run_script_sync("dd-issettingup.ps1", None, None)
        .map(|o| o.contains("Updated"))
        .unwrap_or(false);

    let running = runner
        .run_script_sync("dd-isrunning.ps1", None, None)
        .map(|o| o.contains("Running"))
        .unwrap_or(false);

    if configured && running {
        let _ = tx.send(LogLine::success(
            "✓ Docker Desktop está configurado y en ejecución",
        ));
        return DockerRequirementOutcome::Ready;
    }

    if configured {
        let _ = tx.send(LogLine::warn(
            "Docker Desktop no está en ejecución — iniciando...",
        ));
        return run_script_outcome(runner, tx, "dd-start.ps1", "iniciar");
    }

    let _ = tx.send(LogLine::warn(
        "Aplicando configuración de Docker Desktop...",
    ));
    let _ = tx.send(LogLine::info(
        "  (Docker se reiniciará — puede tardar hasta 2 minutos)",
    ));
    run_script_outcome(runner, tx, "dd-setting.ps1", "configurar")
}

fn run_script_outcome(
    runner: &PsRunner,
    tx: &LogSender,
    script: &str,
    action: &str,
) -> DockerRequirementOutcome {
    let out_tx = make_log_bridge(tx);
    match runner.run_script_sync(script, None, Some(&out_tx)) {
        Ok(o) if o.contains("Continue") => {
            let _ = tx.send(LogLine::success("✓ Docker Desktop listo para WSDD"));
            DockerRequirementOutcome::Ready
        }
        Ok(o) => {
            tracing::warn!(script, output = %o.text, "script no emitió Continue");
            let _ = tx.send(LogLine::error(format!(
                "✗ Docker Desktop no pudo {action} en el tiempo esperado"
            )));
            let _ = tx.send(LogLine::error(
                "  Inicia Docker Desktop manualmente y vuelve a abrir WSDD",
            ));
            DockerRequirementOutcome::StartupFailed
        }
        Err(e) => {
            tracing::error!(script, error = %e, "falló la ejecución del script");
            let _ = tx.send(LogLine::error(format!("✗ Error al {action} Docker: {e}")));
            DockerRequirementOutcome::StartupFailed
        }
    }
}

/// Initializes the WSDD base Docker environment synchronously.
pub fn deploy_environment_sync(runner: &PsRunner, tx: &LogSender) -> Result<(), InfraError> {
    base::deploy_environment_sync(runner, tx)
}

/// Regenerates managed `init.yml` with the current prerequisite credentials.
pub fn sync_prerequisite_compose_sync(settings: &AppSettings) -> Result<(), InfraError> {
    settings.validate_prerequisite_credentials()?;

    let docker_dir = path_config().docker_structure_dir();
    std::fs::create_dir_all(&docker_dir)?;

    let rendered = templates::render_init_yml(&settings.prereq_credentials);
    std::fs::write(docker_dir.join("init.yml"), rendered)?;
    Ok(())
}

/// Synchronizes managed Docker resources for one PHP version before rebuild.
pub fn sync_php_version_resources_sync(
    settings: &AppSettings,
    php_version: &PhpVersion,
) -> Result<(), InfraError> {
    let credentials = settings
        .webmin_credentials_for(php_version)
        .ok_or_else(|| {
            InfraError::PrerequisiteNotMet(format!(
                "Webmin credentials are required for {}",
                php_version.display_name()
            ))
        })?;

    credentials.validate_for_save()?;

    let php_dir = path_config().php_dir(php_version.dir_name());
    std::fs::create_dir_all(&php_dir)?;

    std::fs::write(
        php_dir.join("Dockerfile"),
        templates::dockerfile_template(php_version),
    )?;
    std::fs::write(
        php_dir.join(templates::webserver_file_name(php_version)),
        templates::render_webserver_yml(settings, php_version, credentials),
    )?;
    Ok(())
}

/// Synchronizes all PHP resources that already have stored Webmin credentials.
pub fn sync_saved_php_version_resources_sync(settings: &AppSettings) -> Result<(), InfraError> {
    settings.validate_webmin_credentials()?;

    for credentials in &settings.webmin_credentials {
        if credentials.is_blank() {
            continue;
        }
        sync_php_version_resources_sync(settings, &credentials.php_version)?;
    }

    Ok(())
}

/// Applies FullControl permissions to the MySQL data directory.
pub fn fix_mysql_permissions_sync(
    runner: &PsRunner,
    tx: Option<&OutputSender>,
) -> Result<(), InfraError> {
    runner
        .run_script_sync("dd-fixmysqlpermission.ps1", None, tx)
        .map(|_| ())
}
