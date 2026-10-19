//! Optional developer service compose rendering and lifecycle.

use std::time::{Duration, Instant};

use super::progress::make_docker_progress_bridge;
use super::templates;
use crate::config::environment::{env_config, path_config};
use crate::errors::InfraError;
use crate::handlers::deploy_trace::TraceHandle;
use crate::handlers::docker::{self, WSDD_NETWORK, WSDD_TOOLS_PROJECT};
use crate::handlers::hosts;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::ngrok;
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
    Ngrok,
}

impl OptionalServiceKind {
    pub(crate) fn service_name(self) -> &'static str {
        match self {
            Self::Redis => "redis",
            Self::Mailpit => "mailpit",
            Self::Memcached => "memcached",
            Self::Ngrok => "ngrok",
        }
    }

    pub(crate) fn container_name(self) -> &'static str {
        match self {
            Self::Redis => "WSDD-Redis-Server",
            Self::Mailpit => "WSDD-Mailpit-Server",
            Self::Memcached => "WSDD-Memcached-Server",
            Self::Ngrok => "WSDD-Ngrok-Server",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Redis => "Redis",
            Self::Mailpit => "Mailpit",
            Self::Memcached => "Memcached",
            Self::Ngrok => "ngrok",
        }
    }
}

/// Writes optional service compose files without deploying them.
pub(super) fn sync_resources_sync(settings: &AppSettings) -> Result<(), InfraError> {
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
    std::fs::write(
        path_config().ngrok_yml(),
        templates::render_ngrok_yml(settings),
    )?;
    Ok(())
}

/// Deploys one optional service from its isolated compose file.
pub(super) fn deploy_service_sync(
    runner: &PsRunner,
    settings: &AppSettings,
    kind: OptionalServiceKind,
    tx: &LogSender,
    trace: Option<&TraceHandle>,
) -> Result<(), InfraError> {
    traced_stage(trace, "Preparación del servicio", tx, || {
        settings.validate_optional_services()?;
        docker::require_container_project(runner, kind.container_name(), WSDD_TOOLS_PROJECT)?;
        ensure_network_sync(runner, tx)?;
        if kind == OptionalServiceKind::Mailpit {
            let host = settings.optional_services.mailpit.virtual_host.as_str();
            hosts::update_host(Some(&[host]), tx)
                .map_err(|e| InfraError::PrerequisiteNotMet(e.to_string()))?;
        }
        sync_resources_sync(settings)
    })?;

    let compose_files = [
        path_config().redis_yml(),
        path_config().memcached_yml(),
        path_config().mailpit_yml(),
        path_config().ngrok_yml(),
    ];
    let docker_dir = path_config().docker_structure_dir();
    let command = format!(
        "{} -p {WSDD_TOOLS_PROJECT} {} up -d --no-deps {}",
        env_config().docker_compose_exe(),
        compose_files
            .iter()
            .map(|path| format!("-f \"{}\"", path.display()))
            .collect::<Vec<_>>()
            .join(" "),
        kind.service_name()
    );

    traced_stage(trace, "Descarga e inicio con Compose", tx, || {
        let bridge = make_docker_progress_bridge(tx);
        let result = runner.run_ps_sync(&command, Some(&docker_dir), Some(&bridge));
        bridge
            .complete(
                result,
                &format!("docker-compose up {}", kind.service_name()),
            )
            .map(|_| ())
    })?;

    traced_stage(trace, "Verificación del contenedor", tx, || {
        if wait_until_sync(READY_TIMEOUT, READY_POLL_INTERVAL, || {
            container_running_sync(runner, kind.container_name())
        })? {
            Ok(())
        } else {
            Err(InfraError::UnexpectedOutput(
                format!("docker-compose up {}", kind.service_name()),
                format!("{} no alcanzo estado running", kind.container_name()),
            ))
        }
    })?;

    if kind == OptionalServiceKind::Ngrok {
        traced_stage(trace, "Verificación de Agent API", tx, || {
            if wait_until_sync(READY_TIMEOUT, READY_POLL_INTERVAL, || {
                ngrok::agent_status(settings, runner).map(|status| status.api_reachable)
            })? {
                Ok(())
            } else {
                Err(InfraError::PrerequisiteNotMet(crate::i18n::tr(
                    "tool_ngrok_api_unreachable",
                )))
            }
        })?;
    }

    let _ = tx.send(LogLine::success(format!(
        "✓ Servicio opcional {} desplegado",
        kind.display_name()
    )));
    Ok(())
}

fn traced_stage<T>(
    trace: Option<&TraceHandle>,
    label: &'static str,
    tx: &LogSender,
    task: impl FnOnce() -> Result<T, InfraError>,
) -> Result<T, InfraError> {
    match trace {
        Some(trace) => trace.stage(label, tx, task),
        None => task(),
    }
}

/// Stops one optional service without deleting data volumes.
pub(super) fn stop_service_sync(
    runner: &PsRunner,
    kind: OptionalServiceKind,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let name = kind.container_name();
    docker::require_container_project(runner, name, WSDD_TOOLS_PROJECT)?;
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
    let result = runner.run_direct_sync(
        env_config().docker_exe(),
        &["network", "create", "--driver", "bridge", WSDD_NETWORK],
        None,
        Some(&bridge),
    );
    bridge.complete(result, "docker network create wsdd-network")?;
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
