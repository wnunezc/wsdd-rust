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
//! Base container deployment and readiness checks.

use std::time::{Duration, Instant};

use super::super::progress::{deploy_log_path, make_docker_progress_bridge};
use crate::config::environment::{env_config, path_config};
use crate::errors::InfraError;
use crate::handlers::docker::WSDD_PROJECT;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::ps_script::{PsRunner, ScriptRunner};

const CREATE_READY_TIMEOUT: Duration = Duration::from_secs(30);
const START_READY_TIMEOUT: Duration = Duration::from_secs(90);
const READY_POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Deploys or recovers the base WSDD Docker containers.
pub(super) fn deploy_base_containers_sync(
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    if check_base_containers_running_sync(runner)? {
        let _ = tx.send(LogLine::success(
            "✓ Contenedores WSDD ya están desplegados y activos",
        ));
        return Ok(());
    }

    let init_yml = path_config().init_yml();
    let docker_dir = path_config().docker_structure_dir();
    let log_path = deploy_log_path();
    let base_exists = check_base_containers_sync(runner)?;

    let (status_message, command_label, command) = if base_exists {
        (
            "Contenedores WSDD detectados pero no activos; intentando recuperarlos...",
            "docker-compose up -d",
            format!(
                "{} -p {WSDD_PROJECT} -f \"{}\" up -d",
                env_config().docker_compose_exe(),
                init_yml.display()
            ),
        )
    } else {
        (
            "Construyendo contenedores WSDD — puede tardar varios minutos en la primera ejecución...",
            "docker-compose up -d --build",
            format!(
                "{} -p {WSDD_PROJECT} -f \"{}\" up -d --build",
                env_config().docker_compose_exe(),
                init_yml.display()
            ),
        )
    };

    let _ = tx.send(LogLine::info(status_message));
    let _ = tx.send(LogLine::info(format!(
        "  (output detallado en: {})",
        log_path.display()
    )));
    super::super::progress::write_deploy_log_header(command_label);
    let bridge = make_docker_progress_bridge(tx);
    runner.run_ps_sync(&command, Some(&docker_dir), Some(&bridge))?;

    let _ = tx.send(LogLine::info(
        "Esperando confirmacion real de contenedores creados...",
    ));
    if !wait_until_sync(CREATE_READY_TIMEOUT, READY_POLL_INTERVAL, || {
        check_base_containers_sync(runner)
    })? {
        let _ = tx.send(LogLine::error(
            "Los contenedores no se crearon correctamente",
        ));
        return Err(InfraError::UnexpectedOutput(
            command_label.to_string(),
            "contenedores no encontrados tras la creacion".to_string(),
        ));
    }

    if !check_base_containers_sync(runner)? {
        let _ = tx.send(LogLine::error(
            "✗ Los contenedores no se crearon correctamente",
        ));
        return Err(InfraError::UnexpectedOutput(
            command_label.to_string(),
            "contenedores no encontrados tras la creación".to_string(),
        ));
    }

    let _ = tx.send(LogLine::info(
        "Esperando disponibilidad real de servicios base...",
    ));
    if !wait_until_sync(START_READY_TIMEOUT, READY_POLL_INTERVAL, || {
        check_base_containers_running_sync(runner)
    })? {
        log_final_container_status(runner, tx);
        return Err(InfraError::UnexpectedOutput(
            command_label.to_string(),
            "los servicios base no alcanzaron estado running".to_string(),
        ));
    }

    let _ = tx.send(LogLine::success(
        "✓ Contenedores WSDD desplegados correctamente",
    ));
    Ok(())
}

/// Logs active WSDD containers after provisioning completes.
pub(super) fn show_running_containers_sync(runner: &PsRunner, tx: &LogSender) {
    let _ = tx.send(LogLine::info("Servicios activos:"));
    match runner.run_direct_sync(
        env_config().docker_exe(),
        &[
            "ps",
            "-a",
            "--format",
            "{{.Names}} - {{.Status}}",
            "--filter",
            "name=WSDD-",
        ],
        None,
        None,
    ) {
        Ok(out) => {
            for line in out.text.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    let _ = tx.send(LogLine::info(format!("  {trimmed}")));
                }
            }
        }
        Err(e) => {
            let _ = tx.send(LogLine::warn(format!(
                "No se pudo listar contenedores: {e}"
            )));
        }
    }
}

fn check_base_containers_sync(runner: &PsRunner) -> Result<bool, InfraError> {
    let out = runner.run_direct_sync(env_config().docker_exe(), &["ps", "-a"], None, None)?;
    if out.text.contains("Error") {
        return Err(InfraError::DockerUnreachable(out.text));
    }
    Ok(out.contains("WSDD-Proxy-Server")
        && out.contains("WSDD-MySql-Server")
        && out.contains("WSDD-phpMyAdmin-Server"))
}

fn check_base_containers_running_sync(runner: &PsRunner) -> Result<bool, InfraError> {
    let out = runner.run_direct_sync(
        env_config().docker_exe(),
        &[
            "ps",
            "-a",
            "--format",
            "{{.Names}}|{{.Status}}",
            "--filter",
            "name=WSDD-",
        ],
        None,
        None,
    )?;

    if out.text.contains("Error") {
        return Err(InfraError::DockerUnreachable(out.text));
    }

    let mut proxy_running = false;
    let mut mysql_running = false;
    let mut pma_running = false;

    for line in out.text.lines() {
        let mut parts = line.splitn(2, '|');
        let name = parts.next().unwrap_or_default().trim();
        let status = parts.next().unwrap_or_default().trim().to_lowercase();
        let running =
            status.contains("up") || status.contains("running") || status.contains("started");

        match name {
            "WSDD-Proxy-Server" => proxy_running = running,
            "WSDD-MySql-Server" => mysql_running = running,
            "WSDD-phpMyAdmin-Server" => pma_running = running,
            _ => {}
        }
    }

    Ok(proxy_running && mysql_running && pma_running)
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

fn base_container_status_lines_sync(runner: &PsRunner) -> Result<Vec<String>, InfraError> {
    let out = runner.run_direct_sync(
        env_config().docker_exe(),
        &[
            "ps",
            "-a",
            "--format",
            "{{.Names}}|{{.Status}}",
            "--filter",
            "name=WSDD-",
        ],
        None,
        None,
    )?;

    if out.text.contains("Error") {
        return Err(InfraError::DockerUnreachable(out.text));
    }

    Ok(out
        .text
        .lines()
        .filter(|line| {
            line.contains("WSDD-Proxy-Server")
                || line.contains("WSDD-MySql-Server")
                || line.contains("WSDD-phpMyAdmin-Server")
        })
        .map(|line| line.trim().to_string())
        .collect())
}

fn log_final_container_status(runner: &PsRunner, tx: &LogSender) {
    let _ = tx.send(LogLine::warn(
        "Estado detectado de contenedores base al expirar la espera:",
    ));
    match base_container_status_lines_sync(runner) {
        Ok(lines) if !lines.is_empty() => {
            for line in lines {
                let _ = tx.send(LogLine::warn(format!("  {line}")));
            }
        }
        Ok(_) => {
            let _ = tx.send(LogLine::warn(
                "  No se pudo obtener estado visible de los contenedores base",
            ));
        }
        Err(e) => {
            let _ = tx.send(LogLine::warn(format!(
                "  No se pudo consultar estado final de contenedores: {e}"
            )));
        }
    }
    let _ = tx.send(LogLine::error(
        "Los servicios base no quedaron disponibles a tiempo",
    ));
}
