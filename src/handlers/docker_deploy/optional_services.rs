//! Optional developer service compose rendering and lifecycle.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::progress::make_docker_progress_bridge;
use super::templates;
use crate::config::environment::{env_config, path_config};
use crate::errors::InfraError;
use crate::handlers::docker::WSDD_NETWORK;
use crate::handlers::hosts;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::ps_script::{PsRunner, ScriptRunner};
use crate::handlers::setting::AppSettings;

const READY_TIMEOUT: Duration = Duration::from_secs(60);
const READY_POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Optional developer services managed outside the base stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionalServiceKind {
    Redis,
    Mailpit,
    Memcached,
}

impl OptionalServiceKind {
    fn service_name(self) -> &'static str {
        match self {
            Self::Redis => "redis",
            Self::Mailpit => "mailpit",
            Self::Memcached => "memcached",
        }
    }

    fn container_name(self) -> &'static str {
        match self {
            Self::Redis => "WSDD-Redis-Server",
            Self::Mailpit => "WSDD-Mailpit-Server",
            Self::Memcached => "WSDD-Memcached-Server",
        }
    }

    fn project_name(self) -> &'static str {
        match self {
            Self::Redis => "wsdd-redis",
            Self::Mailpit => "wsdd-mailpit",
            Self::Memcached => "wsdd-memcached",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Redis => "Redis",
            Self::Mailpit => "Mailpit",
            Self::Memcached => "Memcached",
        }
    }

    fn compose_file(self) -> PathBuf {
        match self {
            Self::Redis => path_config().redis_yml(),
            Self::Mailpit => path_config().mailpit_yml(),
            Self::Memcached => path_config().memcached_yml(),
        }
    }
}

/// Writes optional service compose files without deploying them.
pub(super) fn sync_resources_sync(settings: &AppSettings) -> Result<(), InfraError> {
    settings.validate_optional_services()?;

    let services_dir = path_config().optional_services_dir();
    std::fs::create_dir_all(&services_dir)?;
    std::fs::write(
        path_config().redis_yml(),
        templates::render_redis_yml(settings),
    )?;
    std::fs::write(
        path_config().mailpit_yml(),
        templates::render_mailpit_yml(settings),
    )?;
    std::fs::write(
        path_config().memcached_yml(),
        templates::render_memcached_yml(settings),
    )?;
    Ok(())
}

/// Deploys one optional service from its isolated compose file.
pub(super) fn deploy_service_sync(
    runner: &PsRunner,
    settings: &AppSettings,
    kind: OptionalServiceKind,
    tx: &LogSender,
) -> Result<(), InfraError> {
    settings.validate_optional_services()?;

    ensure_network_sync(runner, tx)?;
    if kind == OptionalServiceKind::Mailpit {
        let host = settings.optional_services.mailpit.virtual_host.as_str();
        hosts::update_host(Some(&[host]), tx)
            .map_err(|e| InfraError::PrerequisiteNotMet(e.to_string()))?;
    }

    let compose_file = kind.compose_file();
    if !compose_file.is_file() {
        sync_resources_sync(settings)?;
    }

    let docker_dir = path_config().docker_structure_dir();
    let command = format!(
        "{} -p {} -f \"{}\" up -d {}",
        env_config().docker_compose_exe(),
        kind.project_name(),
        compose_file.display(),
        kind.service_name()
    );

    let _ = tx.send(LogLine::info(format!(
        "Desplegando servicio opcional {}...",
        kind.display_name()
    )));
    let bridge = make_docker_progress_bridge(tx);
    runner.run_ps_sync(&command, Some(&docker_dir), Some(&bridge))?;

    if !wait_until_sync(READY_TIMEOUT, READY_POLL_INTERVAL, || {
        container_running_sync(runner, kind.container_name())
    })? {
        return Err(InfraError::UnexpectedOutput(
            format!("docker-compose up {}", kind.service_name()),
            format!("{} no alcanzo estado running", kind.container_name()),
        ));
    }

    let _ = tx.send(LogLine::success(format!(
        "✓ Servicio opcional {} desplegado",
        kind.display_name()
    )));
    Ok(())
}

/// Stops one optional service without deleting data volumes.
pub(super) fn stop_service_sync(
    runner: &PsRunner,
    kind: OptionalServiceKind,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let name = kind.container_name();
    if !container_exists_sync(runner, name)? {
        let _ = tx.send(LogLine::info(format!(
            "{} no estaba desplegado.",
            kind.display_name()
        )));
        return Ok(());
    }

    let _ = tx.send(LogLine::info(format!(
        "Deteniendo servicio opcional {}...",
        kind.display_name()
    )));
    let out = runner.run_direct_sync(env_config().docker_exe(), &["stop", name], None, None)?;
    if !out.success {
        return Err(InfraError::DockerUnreachable(format!(
            "No se pudo detener {name}: {}",
            out.text
        )));
    }

    let _ = tx.send(LogLine::success(format!(
        "✓ Servicio opcional {} detenido",
        kind.display_name()
    )));
    Ok(())
}

fn ensure_network_sync(runner: &PsRunner, tx: &LogSender) -> Result<(), InfraError> {
    if network_exists_sync(runner)? {
        return Ok(());
    }

    let _ = tx.send(LogLine::info("Creando red Docker wsdd-network..."));
    let bridge = make_docker_progress_bridge(tx);
    runner.run_direct_sync(
        env_config().docker_exe(),
        &["network", "create", "--driver", "bridge", WSDD_NETWORK],
        None,
        Some(&bridge),
    )?;
    Ok(())
}

fn network_exists_sync(runner: &PsRunner) -> Result<bool, InfraError> {
    let out = runner.run_direct_sync(
        env_config().docker_exe(),
        &["network", "inspect", WSDD_NETWORK],
        None,
        None,
    )?;
    Ok(out.success)
}

fn container_exists_sync(runner: &PsRunner, name: &str) -> Result<bool, InfraError> {
    let out = runner.run_direct_sync(
        env_config().docker_exe(),
        &[
            "ps",
            "-a",
            "--format",
            "{{.Names}}",
            "--filter",
            &format!("name={name}"),
        ],
        None,
        None,
    )?;
    Ok(out.text.lines().any(|line| line.trim() == name))
}

fn container_running_sync(runner: &PsRunner, name: &str) -> Result<bool, InfraError> {
    let out = runner.run_direct_sync(
        env_config().docker_exe(),
        &[
            "ps",
            "--format",
            "{{.Names}}",
            "--filter",
            &format!("name={name}"),
        ],
        None,
        None,
    )?;
    Ok(out.text.lines().any(|line| line.trim() == name))
}

fn wait_until_sync<F>(
    timeout: Duration,
    interval: Duration,
    mut check: F,
) -> Result<bool, InfraError>
where
    F: FnMut() -> Result<bool, InfraError>,
{
    let deadline = Instant::now() + timeout;

    loop {
        if check()? {
            return Ok(true);
        }

        if Instant::now() >= deadline {
            return Ok(false);
        }

        std::thread::sleep(interval);
    }
}
