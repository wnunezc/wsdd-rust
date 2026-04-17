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
//! Docker runtime export/import helpers for backup restore flows.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use super::vhost;
use crate::config::environment::{env_config, path_config, path_to_string};
use crate::errors::InfraError;
use crate::handlers::docker::{self, WSDD_NETWORK, WSDD_PROJECT};
use crate::handlers::docker_deploy;
use crate::handlers::hosts;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::ps_script::{PsRunner, ScriptRunner};
use crate::handlers::yml;
use crate::models::project::Project;

/// Returns all Docker images referenced by WSDD containers.
pub(super) fn collect_wsdd_images(runner: &PsRunner) -> Result<Vec<String>, InfraError> {
    let out = runner.run_direct_sync(
        env_config().docker_exe(),
        &[
            "ps",
            "-a",
            "--filter",
            "name=WSDD-",
            "--format",
            "{{.Image}}",
        ],
        None,
        None,
    )?;

    let images: BTreeSet<String> = out
        .text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("<none>"))
        .map(|line| line.to_string())
        .collect();

    Ok(images.into_iter().collect())
}

/// Exports the selected Docker images into a tar archive.
pub(super) fn export_images(
    runner: &PsRunner,
    images: &[String],
    dest_tar: &Path,
    tx: &LogSender,
) -> Result<(), InfraError> {
    if let Some(parent) = dest_tar.parent() {
        fs::create_dir_all(parent).map_err(InfraError::Io)?;
    }

    let _ = tx.send(LogLine::info("[Backup] Exportando imagenes Docker WSDD..."));

    let mut args = vec![
        "save".to_string(),
        "-o".to_string(),
        dest_tar.to_string_lossy().to_string(),
    ];
    args.extend(images.iter().cloned());
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    runner.run_direct_sync(env_config().docker_exe(), &refs, None, None)?;

    let _ = tx.send(LogLine::success("[Backup] Imagenes Docker exportadas ✓"));
    Ok(())
}

/// Writes a Docker network inspection snapshot when the WSDD network exists.
pub(super) fn export_network_snapshot(
    runner: &PsRunner,
    dest_json: &Path,
    tx: &LogSender,
) -> Result<(), InfraError> {
    if let Some(parent) = dest_json.parent() {
        fs::create_dir_all(parent).map_err(InfraError::Io)?;
    }

    let out = runner.run_direct_sync(
        env_config().docker_exe(),
        &["network", "inspect", WSDD_NETWORK],
        None,
        None,
    );
    match out {
        Ok(snapshot) if snapshot.success => {
            fs::write(dest_json, snapshot.text).map_err(InfraError::Io)?;
            let _ = tx.send(LogLine::success("[Backup] Snapshot de red exportado ✓"));
        }
        Ok(_) | Err(_) => {
            let _ = tx.send(LogLine::warn(
                "[Backup] No se pudo exportar snapshot de red wsdd-network",
            ));
        }
    }
    Ok(())
}

/// Stops and removes all currently registered WSDD containers.
pub(super) fn stop_and_remove_wsdd_containers(
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let out = runner.run_direct_sync(
        env_config().docker_exe(),
        &[
            "ps",
            "-a",
            "--filter",
            "name=WSDD-",
            "--format",
            "{{.Names}}",
        ],
        None,
        None,
    )?;

    let names: Vec<String> = out
        .text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect();

    if names.is_empty() {
        return Ok(());
    }

    let _ = tx.send(LogLine::info(
        "[Restore] Deteniendo y eliminando contenedores WSDD actuales...",
    ));

    for name in &names {
        let _ = runner.run_direct_sync(env_config().docker_exe(), &["stop", name], None, None);
        let _ = runner.run_direct_sync(env_config().docker_exe(), &["rm", name], None, None);
    }

    let _ = tx.send(LogLine::success(
        "[Restore] Contenedores WSDD previos detenidos ✓",
    ));
    Ok(())
}

/// Recreates Docker runtime wiring for a restored project.
pub(super) fn rehydrate_project_runtime(
    project: &Project,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let _ = tx.send(LogLine::info(format!(
        "[Restore] Rehidratando proyecto '{}'...",
        project.name
    )));

    create_project_bind_volume(project, runner)?;
    let options_file = yml::options_path(
        project.php_version.dir_name(),
        project.php_version.compose_tag(),
    );
    yml::add_project_to_options_yml(
        &options_file,
        &project.domain,
        project.php_version.compose_tag(),
    )?;
    vhost::sync_active_vhost_for_php(&project.php_version, tx)?;
    rebuild_php_container(project, runner, tx)?;

    if project.ssl {
        restart_proxy(runner, tx)?;
    }

    let base_domains = project.php_version.base_container_domains();
    let mut domains: Vec<&str> = base_domains.iter().map(String::as_str).collect();
    domains.push(project.domain.as_str());

    hosts::update_host(Some(&domains), tx)
        .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;

    let _ = tx.send(LogLine::success(format!(
        "[Restore] Proyecto '{}' rehidratado ✓",
        project.name
    )));
    Ok(())
}

fn create_project_bind_volume(project: &Project, runner: &PsRunner) -> Result<(), InfraError> {
    let volume_name = format!("{}-{}", project.php_version.compose_tag(), project.domain);
    let device_opt = format!("device={}", project.work_path);

    let _ = runner.run_direct_sync(
        env_config().docker_exe(),
        &["volume", "rm", &volume_name],
        None,
        None,
    );
    runner.run_direct_sync(
        env_config().docker_exe(),
        &[
            "volume",
            "create",
            "--driver",
            "local",
            "--opt",
            "type=none",
            "--opt",
            &device_opt,
            "--opt",
            "o=bind",
            &volume_name,
        ],
        None,
        None,
    )?;
    Ok(())
}

fn rebuild_php_container(
    project: &Project,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let container_name = format!("WSDD-Web-Server-{}", project.php_version.container_tag());
    let php_dir = project.php_version.dir_name();
    let compose_tag = project.php_version.compose_tag();
    let should_build =
        docker::php_container_exists_sync(runner, project.php_version.container_tag())
            .map(|exists| !exists)
            .unwrap_or(true);
    let options_yml = yml::options_path(php_dir, compose_tag);
    let webserver_yml = path_to_string(path_config().webserver_yml(php_dir, compose_tag));

    let _ = runner.run_direct_sync(
        env_config().docker_exe(),
        &["stop", &container_name],
        None,
        None,
    );
    let _ = runner.run_direct_sync(
        env_config().docker_exe(),
        &["rm", &container_name],
        None,
        None,
    );

    let message = if should_build {
        format!(
            "[Restore] Reconstruyendo y creando contenedor {}...",
            container_name
        )
    } else {
        format!("[Restore] Recreando contenedor {}...", container_name)
    };
    let _ = tx.send(LogLine::info(message));

    let bridge = docker_deploy::make_log_bridge(tx);
    let docker_dir = path_config().php_dir(php_dir);
    runner.run_ps_sync(
        &format!(
            "{} -p {WSDD_PROJECT} -f \"{webserver_yml}\" -f \"{options_yml}\" up -d {}--force-recreate",
            env_config().docker_compose_exe(),
            if should_build { "--build " } else { "" }
        ),
        Some(&docker_dir),
        Some(&bridge),
    )?;

    Ok(())
}

fn restart_proxy(runner: &PsRunner, tx: &LogSender) -> Result<(), InfraError> {
    let _ = tx.send(LogLine::info("[Restore] Reiniciando WSDD-Proxy-Server..."));
    runner.run_direct_sync(
        env_config().docker_exe(),
        &["restart", "WSDD-Proxy-Server"],
        None,
        None,
    )?;
    Ok(())
}
