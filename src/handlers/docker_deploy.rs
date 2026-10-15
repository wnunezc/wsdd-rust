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
mod optional_services;
mod progress;
mod templates;

pub use optional_services::OptionalServiceKind;
pub use progress::{make_docker_progress_bridge, make_log_bridge};

use crate::config::environment::path_config;
use crate::errors::InfraError;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::ps_script::{OutputSender, PsRunner, ScriptRunner};
use crate::handlers::setting::AppSettings;
use crate::handlers::{project, yml};
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
    let result = runner.run_script_sync(script, None, Some(&out_tx));
    match out_tx.complete(result, script) {
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
    std::fs::write(
        docker_dir.join("proxy.conf"),
        templates::render_proxy_conf(),
    )?;
    Ok(())
}

/// Synchronizes WSDD-managed Docker resources into an existing local environment.
///
/// This intentionally updates only files owned by WSDD templates and performs
/// line-based migrations for live `options.phpXX.yml` files. It does not
/// re-extract the complete Docker structure, so local projects, volumes, SSL
/// certificates, database files, and project-specific compose entries remain in
/// place.
///
/// Important: project `options.phpXX.yml` files are WSDD-managed live state, not
/// immutable templates. They must never be overwritten during startup
/// reconciliation because Add/Remove Project mutates their virtual hosts,
/// volume mounts, global volume declarations, and `extra_hosts` entries.
pub fn sync_installed_docker_resources_sync(settings: &AppSettings) -> Result<(), InfraError> {
    sync_prerequisite_compose_sync(settings)?;

    let mut render_settings = settings.clone();
    for php_version in PhpVersion::all() {
        render_settings.ensure_legacy_webmin_credentials(php_version.clone());
        sync_php_version_resources_sync(&render_settings, &php_version)?;
    }

    sync_registered_project_options_sync()
}

/// Writes optional service compose files without deploying them.
pub fn sync_optional_service_resources_sync(settings: &AppSettings) -> Result<(), InfraError> {
    optional_services::sync_resources_sync(settings)
}

/// Deploys one optional developer service from its isolated compose file.
pub fn deploy_optional_service_sync(
    runner: &PsRunner,
    settings: &AppSettings,
    kind: OptionalServiceKind,
    tx: &LogSender,
) -> Result<(), InfraError> {
    optional_services::deploy_service_sync(runner, settings, kind, tx)
}

/// Stops one optional developer service without deleting its data volume.
pub fn stop_optional_service_sync(
    runner: &PsRunner,
    kind: OptionalServiceKind,
    tx: &LogSender,
) -> Result<(), InfraError> {
    optional_services::stop_service_sync(runner, kind, tx)
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
    if php_version.supports_php_admin() {
        sync_php_admin_resources_sync(php_version)?;
    }
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

fn sync_registered_project_options_sync() -> Result<(), InfraError> {
    for php_version in PhpVersion::all()
        .into_iter()
        .filter(PhpVersion::supports_php_admin)
    {
        sync_php_admin_options_sync(&php_version)?;
    }

    for project in project::list_all()? {
        let options_file = yml::options_path(
            project.php_version.dir_name(),
            project.php_version.compose_tag(),
        );
        yml::add_project_to_options_yml(
            &options_file,
            &project.domain,
            project.php_version.compose_tag(),
        )?;
    }

    Ok(())
}

fn sync_php_admin_resources_sync(php_version: &PhpVersion) -> Result<(), InfraError> {
    sync_php_admin_options_sync(php_version)?;

    let vhost_file = path_config().active_vhost_conf(php_version.dir_name());
    yml::ensure_php_admin_vhost_conf(
        &crate::config::environment::path_to_string(vhost_file),
        &php_version.php_admin_domain(),
    )?;

    // The whole sites-enabled directory is a bind mount: provision its source,
    // not a build-time file that the mount would hide. Preserve personal vhosts.
    sync_php_admin_probe_file(
        &path_config()
            .php_dir(php_version.dir_name())
            .join("vhost/wsdd-php-admin-probe.conf"),
        php_version,
    )?;

    Ok(())
}

fn sync_php_admin_probe_file(
    path: &std::path::Path,
    version: &PhpVersion,
) -> Result<(), InfraError> {
    let rendered = templates::render_php_admin_probe_conf(version);
    match std::fs::read_to_string(path) {
        Ok(existing) if existing == rendered => return Ok(()),
        Ok(existing) if !existing.starts_with("# Managed by WSDD.") => {
            return Err(InfraError::PrerequisiteNotMet(format!(
                "No se sobrescribira el vhost no administrado: {}",
                path.display()
            )));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    std::fs::write(path, rendered)?;
    Ok(())
}

fn sync_php_admin_options_sync(php_version: &PhpVersion) -> Result<(), InfraError> {
    let paths = path_config();
    std::fs::create_dir_all(paths.php_admin_state_dir(php_version.compose_tag()))?;
    std::fs::create_dir_all(paths.php_admin_log_dir(php_version.compose_tag()))?;

    let options_file = yml::options_path(php_version.dir_name(), php_version.compose_tag());
    yml::ensure_php_admin_options_yml(
        &options_file,
        &php_version.php_admin_domain(),
        php_version.compose_tag(),
    )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_probe_file_is_created_updated_and_preserves_other_vhosts() {
        let dir = tempfile::tempdir().unwrap();
        let probe = dir.path().join("wsdd-php-admin-probe.conf");
        let personal = dir.path().join("vhost.conf");
        std::fs::write(&personal, "custom personal vhosts").unwrap();
        sync_php_admin_probe_file(&probe, &PhpVersion::Php83).unwrap();
        let first = std::fs::read_to_string(&probe).unwrap();
        sync_php_admin_probe_file(&probe, &PhpVersion::Php83).unwrap();
        assert_eq!(std::fs::read_to_string(&probe).unwrap(), first);
        sync_php_admin_probe_file(&probe, &PhpVersion::Php84).unwrap();
        assert!(std::fs::read_to_string(&probe)
            .unwrap()
            .contains("/var/www/html/phpadmin84.dock/public"));
        assert_eq!(
            std::fs::read_to_string(&personal).unwrap(),
            "custom personal vhosts"
        );
    }

    #[test]
    fn unmanaged_probe_filename_collision_is_not_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let probe = dir.path().join("wsdd-php-admin-probe.conf");
        std::fs::write(&probe, "user-owned configuration").unwrap();
        assert!(sync_php_admin_probe_file(&probe, &PhpVersion::Php83).is_err());
        assert_eq!(
            std::fs::read_to_string(&probe).unwrap(),
            "user-owned configuration"
        );
    }
}
