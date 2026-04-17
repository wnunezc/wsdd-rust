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
//! Project deploy/remove orchestration facade.

use crate::errors::InfraError;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::project as project_handler;
use crate::handlers::ps_script::PsRunner;
use crate::handlers::setting::AppSettings;
use crate::models::project::Project;

mod docker_ops;
mod paths;
mod rollback;
mod steps;
mod vhost;

/// Deploys a complete WSDD project.
///
/// # Errors
/// Returns [`InfraError`] when any persistence, Docker, vhost, SSL, or hosts step fails.
pub fn deploy_project(
    project: &Project,
    settings: &AppSettings,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let _ = tx.send(LogLine::info(format!(
        "[Deploy] Iniciando despliegue de '{}'...",
        project.name
    )));

    let project_snapshot =
        rollback::FileSnapshot::capture(paths::project_file_path(&project.name))?;
    let options_snapshot = rollback::FileSnapshot::capture(paths::options_yml_path(project))?;
    let vhost_snapshot = rollback::FileSnapshot::capture(paths::active_vhost_conf_path(
        project.php_version.dir_name(),
    ))?;
    let ssl_snapshots = paths::ssl_file_paths(project)
        .into_iter()
        .map(rollback::FileSnapshot::capture)
        .collect::<Result<Vec<_>, _>>()?;
    let hosts_snapshot = rollback::capture_hosts_snapshot()?;
    let volume_existed_before = docker_ops::project_volume_exists(project, runner)?;
    let container_existed_before = docker_ops::php_container_exists(project, runner)?;
    let mut volume_created = false;
    let mut container_rebuilt = false;

    let result = (|| -> Result<(), InfraError> {
        project_handler::save(project)?;
        let _ = tx.send(LogLine::success("[Deploy] Proyecto guardado ✓"));

        volume_created = docker_ops::create_volume(project, runner, tx)?;
        steps::update_options_yml(project, tx)?;

        crate::handlers::docker_deploy::sync_php_version_resources_sync(
            settings,
            &project.php_version,
        )?;
        let _ = tx.send(LogLine::success(
            "[Deploy] Recursos gestionados de PHP/Webmin sincronizados ✓",
        ));

        vhost::update(project, tx)?;
        docker_ops::sync_php_container(project, runner, tx, true)?;
        container_rebuilt = true;

        if project.ssl {
            steps::setup_ssl(project, runner, tx)?;
        }

        steps::update_hosts(project, tx)?;
        Ok(())
    })();

    if let Err(err) = result {
        rollback_failed_deploy(
            project,
            runner,
            tx,
            rollback::DeployRollback {
                project_snapshot,
                options_snapshot,
                vhost_snapshot,
                ssl_snapshots,
                hosts_snapshot,
                volume_created,
                volume_existed_before,
                container_rebuilt,
                container_existed_before,
            },
        );
        return Err(err);
    }

    let _ = tx.send(LogLine::success(format!(
        "[Deploy] '{}' desplegado correctamente ✓",
        project.name
    )));
    Ok(())
}

/// Removes a complete WSDD project.
///
/// # Errors
/// Returns [`InfraError`] when any options, Docker, vhost, hosts, SSL, or project file step fails.
pub fn remove_project(
    project: &Project,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let _ = tx.send(LogLine::info(format!(
        "[Remove] Eliminando proyecto '{}'...",
        project.name
    )));

    let project_snapshot =
        rollback::FileSnapshot::capture(paths::project_file_path(&project.name))?;
    let options_snapshot = rollback::FileSnapshot::capture(paths::options_yml_path(project))?;
    let vhost_snapshot = rollback::FileSnapshot::capture(paths::active_vhost_conf_path(
        project.php_version.dir_name(),
    ))?;
    let ssl_snapshots = paths::ssl_file_paths(project)
        .into_iter()
        .map(rollback::FileSnapshot::capture)
        .collect::<Result<Vec<_>, _>>()?;
    let hosts_snapshot = rollback::capture_hosts_snapshot()?;
    let container_existed_before = docker_ops::php_container_exists(project, runner)?;
    let mut container_rebuilt = false;
    let mut volume_removed = false;

    let result = (|| -> Result<(), InfraError> {
        steps::remove_options_yml(project, tx)?;
        vhost::remove(project, tx)?;

        if container_existed_before {
            docker_ops::sync_php_container(project, runner, tx, false)?;
            container_rebuilt = true;
        } else {
            let _ = tx.send(LogLine::info(
                "[Remove] El contenedor PHP no existe; se omite recreate",
            ));
        }

        steps::remove_hosts(project, tx)?;
        steps::remove_ssl(project, tx)?;
        volume_removed = docker_ops::remove_volume(project, runner, tx)?;
        project_handler::delete(&project.name)?;
        Ok(())
    })();

    if let Err(err) = result {
        rollback_failed_remove(
            project,
            runner,
            tx,
            rollback::RemoveRollback {
                project_snapshot,
                options_snapshot,
                vhost_snapshot,
                ssl_snapshots,
                hosts_snapshot,
                container_rebuilt,
                volume_removed,
            },
        );
        return Err(err);
    }

    let _ = tx.send(LogLine::success(format!(
        "[Remove] '{}' eliminado ✓",
        project.name
    )));
    Ok(())
}

fn rollback_failed_deploy(
    project: &Project,
    runner: &PsRunner,
    tx: &LogSender,
    state: rollback::DeployRollback,
) {
    let _ = tx.send(LogLine::warn(
        "[Rollback] Fallo detectado. Restaurando estado previo del deploy...",
    ));
    rollback::restore_file_snapshots(
        tx,
        &[
            (&state.project_snapshot, "project.json"),
            (&state.options_snapshot, "options.yml"),
            (&state.vhost_snapshot, "vhost.conf"),
        ],
    );
    rollback::restore_ssl_snapshots(&state.ssl_snapshots, tx);
    rollback::restore_hosts_snapshot_best_effort(&state.hosts_snapshot, tx);

    if state.container_rebuilt {
        if state.container_existed_before {
            if let Err(e) = docker_ops::sync_php_container(project, runner, tx, false) {
                let _ = tx.send(LogLine::warn(format!(
                    "[Rollback] No se pudo restaurar el contenedor PHP: {e}"
                )));
            }
        } else if let Err(e) = docker_ops::remove_php_container(project, runner, tx) {
            let _ = tx.send(LogLine::warn(format!(
                "[Rollback] No se pudo limpiar el contenedor PHP nuevo: {e}"
            )));
        }
    }

    if state.volume_created && !state.volume_existed_before {
        if let Err(e) = docker_ops::remove_volume(project, runner, tx) {
            let _ = tx.send(LogLine::warn(format!(
                "[Rollback] No se pudo limpiar el volumen nuevo: {e}"
            )));
        }
    }
}

fn rollback_failed_remove(
    project: &Project,
    runner: &PsRunner,
    tx: &LogSender,
    state: rollback::RemoveRollback,
) {
    let _ = tx.send(LogLine::warn(
        "[Rollback] Fallo detectado. Restaurando estado previo del remove...",
    ));
    rollback::restore_file_snapshots(
        tx,
        &[
            (&state.project_snapshot, "project.json"),
            (&state.options_snapshot, "options.yml"),
            (&state.vhost_snapshot, "vhost.conf"),
        ],
    );
    rollback::restore_ssl_snapshots(&state.ssl_snapshots, tx);
    rollback::restore_hosts_snapshot_best_effort(&state.hosts_snapshot, tx);

    if state.container_rebuilt {
        if let Err(e) = docker_ops::sync_php_container(project, runner, tx, false) {
            let _ = tx.send(LogLine::warn(format!(
                "[Rollback] No se pudo restaurar el contenedor PHP: {e}"
            )));
        }
    }

    if state.volume_removed {
        if let Err(e) = docker_ops::create_volume(project, runner, tx) {
            let _ = tx.send(LogLine::warn(format!(
                "[Rollback] No se pudo recrear el volumen removido: {e}"
            )));
        }
    }
}
