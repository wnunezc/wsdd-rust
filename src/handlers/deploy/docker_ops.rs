use std::path::Path;

use crate::config::environment::env_config;
use crate::errors::InfraError;
use crate::handlers::docker;
use crate::handlers::docker::WSDD_PROJECT;
use crate::handlers::docker_deploy::{make_docker_progress_bridge, make_log_bridge};
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::ps_script::{PsRunner, ScriptRunner};
use crate::models::project::Project;

use super::paths;

pub(super) fn project_volume_exists(
    project: &Project,
    runner: &PsRunner,
) -> Result<bool, InfraError> {
    let volume_name = paths::volume_name(project);
    let out = runner.run_direct_sync(
        env_config().docker_exe(),
        &["volume", "ls", "--format", "{{.Name}}"],
        None,
        None,
    )?;
    Ok(out
        .text
        .lines()
        .map(str::trim)
        .any(|line| line == volume_name))
}

pub(super) fn php_container_exists(
    project: &Project,
    runner: &PsRunner,
) -> Result<bool, InfraError> {
    docker::php_container_exists_sync(runner, project.php_version.container_tag())
        .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))
}

pub(super) fn create_volume(
    project: &Project,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<bool, InfraError> {
    let volume_name = paths::volume_name(project);
    let device_opt = format!("device={}", project.work_path);

    if project_volume_exists(project, runner)? {
        let _ = tx.send(LogLine::success(format!(
            "[Deploy] Volumen '{}' ya existe; se reutiliza ✓",
            volume_name
        )));
        return Ok(false);
    }

    let _ = tx.send(LogLine::info(format!(
        "[Deploy] Creando volumen Docker '{}'...",
        volume_name
    )));

    let bridge = make_log_bridge(tx);
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
        Some(&bridge),
    )?;

    let _ = tx.send(LogLine::success(format!(
        "[Deploy] Volumen '{}' creado ✓",
        volume_name
    )));
    Ok(true)
}

/// Recreates the PHP container so Apache picks up options and vhost changes.
pub(super) fn sync_php_container(
    project: &Project,
    runner: &PsRunner,
    tx: &LogSender,
    build_on_missing: bool,
) -> Result<(), InfraError> {
    let container_name = paths::php_container_name(project);
    let php_dir_name = project.php_version.dir_name();
    let compose_tag = project.php_version.compose_tag();
    let container_exists = php_container_exists(project, runner)?;
    let should_build = build_on_missing && !container_exists;

    let bin_dir_str = paths::php_bin_dir(php_dir_name);
    let bin_dir = Path::new(&bin_dir_str);
    let webserver_yml = paths::webserver_yml_path(php_dir_name, compose_tag);
    let options_yml = crate::handlers::yml::options_path(php_dir_name, compose_tag);

    let _ = tx.send(LogLine::info(format!(
        "[Runtime] Deteniendo {}...",
        container_name
    )));
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

    let _ = tx.send(LogLine::info(if should_build {
        "[Runtime] Construyendo y creando contenedor PHP (puede tardar)..."
    } else {
        "[Runtime] Recreando contenedor PHP..."
    }));
    let bridge = make_docker_progress_bridge(tx);
    runner.run_ps_sync(
        &format!(
            "{} -p {WSDD_PROJECT} -f \"{webserver_yml}\" -f \"{options_yml}\" up -d {}--force-recreate",
            env_config().docker_compose_exe(),
            if should_build { "--build " } else { "" }
        ),
        Some(bin_dir),
        Some(&bridge),
    )?;

    let _ = tx.send(LogLine::success(format!(
        "[Runtime] {} sincronizado ✓",
        container_name
    )));
    Ok(())
}

pub(super) fn remove_php_container(
    project: &Project,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let container_name = paths::php_container_name(project);
    let _ = tx.send(LogLine::info(format!(
        "[Rollback] Eliminando contenedor PHP '{}'...",
        container_name
    )));
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
    Ok(())
}

pub(super) fn remove_volume(
    project: &Project,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<bool, InfraError> {
    let volume_name = paths::volume_name(project);
    if !project_volume_exists(project, runner)? {
        let _ = tx.send(LogLine::info(format!(
            "[Remove] El volumen '{}' ya no existe.",
            volume_name
        )));
        return Ok(false);
    }

    let _ = tx.send(LogLine::info(format!(
        "[Remove] Eliminando volumen '{}'...",
        volume_name
    )));
    runner.run_direct_sync(
        env_config().docker_exe(),
        &["volume", "rm", &volume_name],
        None,
        None,
    )?;
    let _ = tx.send(LogLine::success(format!(
        "[Remove] Volumen '{}' eliminado ✓",
        volume_name
    )));
    Ok(true)
}
