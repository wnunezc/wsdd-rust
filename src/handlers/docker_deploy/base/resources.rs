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
//! Docker host, network, and base volume provisioning.

use super::super::progress::make_log_bridge;
use crate::config::environment::{env_config, path_config, path_to_string, DOCKER_HOST_VALUE};
use crate::errors::InfraError;
use crate::handlers::docker::WSDD_NETWORK;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::ps_script::{PsRunner, ScriptRunner};

/// Sets `DOCKER_HOST` in User and Machine scopes for Docker CLI access.
pub(super) fn set_docker_host_env_sync(
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let _ = tx.send(LogLine::info(
        "Configurando DOCKER_HOST=tcp://localhost:2375...",
    ));
    let cmds = [
        format!(
            r#"[Environment]::SetEnvironmentVariable("DOCKER_HOST", "{DOCKER_HOST_VALUE}", "User")"#
        ),
        format!(
            r#"[Environment]::SetEnvironmentVariable("DOCKER_HOST", "{DOCKER_HOST_VALUE}", "Machine")"#
        ),
    ];
    for cmd in cmds {
        runner.run_ps_sync(&cmd, None, None)?;
    }
    let _ = tx.send(LogLine::success(
        "✓ DOCKER_HOST configurado (User + Machine)",
    ));
    Ok(())
}

/// Ensures the shared WSDD Docker bridge network exists.
pub(super) fn ensure_network_sync(runner: &PsRunner, tx: &LogSender) -> Result<(), InfraError> {
    let _ = tx.send(LogLine::info("Verificando red Docker wsdd-network..."));
    if check_network_sync(runner)? {
        let _ = tx.send(LogLine::success("✓ Red wsdd-network ya existe"));
        return Ok(());
    }
    let _ = tx.send(LogLine::warn("Red wsdd-network no encontrada — creando..."));
    let bridge = make_log_bridge(tx);
    runner.run_direct_sync(
        env_config().docker_exe(),
        &["network", "create", "--driver", "bridge", WSDD_NETWORK],
        None,
        Some(&bridge),
    )?;
    if check_network_sync(runner)? {
        let _ = tx.send(LogLine::success("✓ Red wsdd-network creada correctamente"));
        Ok(())
    } else {
        let _ = tx.send(LogLine::error("✗ No se pudo crear la red wsdd-network"));
        Err(InfraError::UnexpectedOutput(
            "docker network create wsdd-network".to_string(),
            "red no encontrada tras la creación".to_string(),
        ))
    }
}

/// Ensures the phpMyAdmin bind volume exists.
pub(super) fn create_pma_volume_sync(runner: &PsRunner, tx: &LogSender) -> Result<(), InfraError> {
    let _ = tx.send(LogLine::info("Verificando volumen pma-code..."));
    if check_pma_volume_sync(runner)? {
        let _ = tx.send(LogLine::success("✓ Volumen pma-code ya existe"));
        return Ok(());
    }
    let _ = tx.send(LogLine::warn("Volumen pma-code no encontrado — creando..."));
    let device = path_to_string(path_config().pma_app_dir());
    let device_opt = format!("device={device}");
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
            "pma-code",
        ],
        None,
        Some(&bridge),
    )?;
    if check_pma_volume_sync(runner)? {
        let _ = tx.send(LogLine::success("✓ Volumen pma-code creado correctamente"));
        Ok(())
    } else {
        let _ = tx.send(LogLine::error("✗ No se pudo crear el volumen pma-code"));
        Err(InfraError::UnexpectedOutput(
            "docker volume create pma-code".to_string(),
            "volumen no encontrado tras la creación".to_string(),
        ))
    }
}

fn check_network_sync(runner: &PsRunner) -> Result<bool, InfraError> {
    let out = runner.run_direct_sync(env_config().docker_exe(), &["network", "ls"], None, None)?;
    if out.text.contains("Error") {
        return Err(InfraError::DockerUnreachable(out.text));
    }
    Ok(out.contains(WSDD_NETWORK))
}

fn check_pma_volume_sync(runner: &PsRunner) -> Result<bool, InfraError> {
    let out = runner.run_direct_sync(env_config().docker_exe(), &["volume", "ls"], None, None)?;
    if out.text.contains("Error") {
        return Err(InfraError::DockerUnreachable(out.text));
    }
    Ok(out.contains("pma-code"))
}
