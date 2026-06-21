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
use super::runtime_storage;

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct VolumeCreateOutcome {
    pub created: bool,
    pub runtime_container_removed: bool,
}

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
    docker::php_container_exists_sync(runner, project.php_version.runtime_container_tag())
        .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))
}

pub(super) fn create_volume(
    project: &Project,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<VolumeCreateOutcome, InfraError> {
    let strategy = runtime_storage::resolve(project)?;
    let volume_name = strategy.volume_name.clone();
    let mut runtime_container_removed = false;

    if project_volume_exists(project, runner)? {
        if strategy.mode == crate::models::project::StorageMode::FastSyncWorkspace {
            let _ = tx.send(LogLine::success(format!(
                "[Deploy] Volumen '{}' ya existe; se reutiliza ✓",
                volume_name
            )));
            runtime_storage::sync_fast_workspace(&strategy, runner, tx)?;
            return Ok(VolumeCreateOutcome::default());
        }

        let existing_source = project_volume_source(&volume_name, runner)?;
        let source_matches =
            volume_source_matches(existing_source.as_deref(), &strategy.source_path);
        if !should_recreate_existing_volume(strategy.mode, source_matches) {
            let _ = tx.send(LogLine::success(format!(
                "[Deploy] Volumen '{}' ya existe; se reutiliza ✓",
                volume_name
            )));
            if strategy.requires_initial_sync {
                runtime_storage::sync_fast_workspace(&strategy, runner, tx)?;
            }
            return Ok(VolumeCreateOutcome::default());
        }

        let reason = if source_matches {
            "requiere renovar el enlace interno de Docker Desktop"
        } else {
            "usa un origen obsoleto"
        };
        let _ = tx.send(LogLine::warn(format!(
            "[Deploy] Volumen '{}' {reason}; se recreara.",
            volume_name
        )));
        let container_name = paths::php_container_name(project);
        if php_container_exists(project, runner)? {
            let _ = runner.run_direct_sync(
                env_config().docker_exe(),
                &["stop", &container_name],
                None,
                None,
            );
            let remove_container = runner.run_direct_sync(
                env_config().docker_exe(),
                &["rm", &container_name],
                None,
                None,
            )?;
            if !remove_container.success {
                return Err(InfraError::UnexpectedOutput(
                    "docker rm runtime container".to_string(),
                    remove_container.text,
                ));
            }
            runtime_container_removed = true;
        }

        let remove_volume = runner.run_direct_sync(
            env_config().docker_exe(),
            &["volume", "rm", &volume_name],
            None,
            None,
        )?;
        if !remove_volume.success {
            return Err(InfraError::UnexpectedOutput(
                "docker volume rm".to_string(),
                remove_volume.text,
            ));
        }
    }

    let _ = tx.send(LogLine::info(format!(
        "[Deploy] Creando volumen Docker '{}'...",
        volume_name
    )));

    let args = strategy.volume_create_args();
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let bridge = make_log_bridge(tx);
    let create =
        runner.run_direct_sync(env_config().docker_exe(), &arg_refs, None, Some(&bridge))?;
    if !create.success {
        return Err(InfraError::UnexpectedOutput(
            "docker volume create".to_string(),
            create.text,
        ));
    }

    if strategy.requires_initial_sync {
        runtime_storage::sync_fast_workspace(&strategy, runner, tx)?;
    }

    let _ = tx.send(LogLine::success(format!(
        "[Deploy] Volumen '{}' creado ✓",
        volume_name
    )));
    Ok(VolumeCreateOutcome {
        created: true,
        runtime_container_removed,
    })
}

fn project_volume_source(
    volume_name: &str,
    runner: &PsRunner,
) -> Result<Option<String>, InfraError> {
    let out = runner.run_direct_sync(
        env_config().docker_exe(),
        &[
            "volume",
            "inspect",
            volume_name,
            "--format",
            "{{if .Options}}{{index .Options \"device\"}}{{end}}",
        ],
        None,
        None,
    )?;
    if !out.success {
        return Err(InfraError::UnexpectedOutput(
            "docker volume inspect".to_string(),
            out.text,
        ));
    }
    let source = out.text.trim();
    Ok((!source.is_empty()).then(|| source.to_string()))
}

fn volume_source_matches(existing: Option<&str>, expected: &str) -> bool {
    existing.is_some_and(|source| source.trim().eq_ignore_ascii_case(expected.trim()))
}

fn should_recreate_existing_volume(
    mode: crate::models::project::StorageMode,
    source_matches: bool,
) -> bool {
    mode == crate::models::project::StorageMode::ManagedNativeLinuxStorage || !source_matches
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
    let compose = runner.run_ps_sync(
        &format!(
            "{} -p {WSDD_PROJECT} -f \"{webserver_yml}\" -f \"{options_yml}\" up -d {}--force-recreate",
            env_config().docker_compose_exe(),
            if should_build { "--build " } else { "" }
        ),
        Some(bin_dir),
        Some(&bridge),
    )?;
    if !compose.success {
        return Err(InfraError::UnexpectedOutput(
            "docker-compose up".to_string(),
            compose.text,
        ));
    }

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

#[cfg(test)]
mod tests {
    use super::{should_recreate_existing_volume, volume_source_matches};
    use crate::models::project::StorageMode;

    #[test]
    fn volume_source_comparison_accepts_unc_case_differences() {
        assert!(volume_source_matches(
            Some(r"\\wsl.localhost\WSDD-Storage\home\wsdd\projects\demo"),
            r"\\WSL.LOCALHOST\wsdd-storage\home\wsdd\projects\demo"
        ));
    }

    #[test]
    fn volume_source_comparison_rejects_linux_path_for_managed_unc() {
        assert!(!volume_source_matches(
            Some("/home/wsdd/projects/demo"),
            r"\\wsl.localhost\WSDD-Storage\home\wsdd\projects\demo"
        ));
    }

    #[test]
    fn managed_storage_refreshes_volume_even_when_source_text_matches() {
        assert!(should_recreate_existing_volume(
            StorageMode::ManagedNativeLinuxStorage,
            true
        ));
        assert!(!should_recreate_existing_volume(
            StorageMode::ClassicBind,
            true
        ));
    }
}
