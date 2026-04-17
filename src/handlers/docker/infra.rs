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
//! Docker network, volume, container existence, and environment helpers.

use super::WSDD_NETWORK;
use crate::config::environment::{env_config, DOCKER_HOST_VALUE};
use crate::errors::InfraError;
use crate::handlers::ps_script::{run_docker, OutputSender, PsRunner, ScriptRunner};

/// Checks whether the shared WSDD Docker network exists.
pub async fn network_exists() -> Result<bool, InfraError> {
    let out = run_docker(vec!["network".to_string(), "ls".to_string()], None).await?;
    if out.text.contains("Error") {
        return Err(InfraError::DockerUnreachable(out.text.clone()));
    }
    Ok(out.contains(WSDD_NETWORK))
}

/// Creates the shared WSDD Docker bridge network when missing.
pub async fn ensure_network(
    runner: &PsRunner,
    tx: Option<OutputSender>,
) -> Result<bool, InfraError> {
    if network_exists().await? {
        return Ok(true);
    }
    let cmd = format!(
        "{} network create --driver bridge {WSDD_NETWORK}",
        env_config().docker_exe()
    );
    let runner = runner.clone();
    let result = tokio::task::spawn_blocking(move || runner.run_ps_sync(&cmd, None, tx.as_ref()))
        .await
        .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))??;

    network_exists().await.inspect(|&exists| {
        if !exists {
            tracing::warn!(output = %result.text, "La red no se creó correctamente");
        }
    })
}

/// Checks whether the phpMyAdmin bind volume exists.
pub async fn pma_volume_exists() -> Result<bool, InfraError> {
    let out = run_docker(vec!["volume".to_string(), "ls".to_string()], None).await?;
    if out.text.contains("Error") {
        return Err(InfraError::DockerUnreachable(out.text.clone()));
    }
    Ok(out.contains("pma-code"))
}

/// Checks whether the base WSDD containers exist.
pub async fn base_containers_exist() -> Result<bool, InfraError> {
    let out = run_docker(vec!["ps".to_string(), "-a".to_string()], None).await?;
    if out.text.contains("Error") {
        return Err(InfraError::DockerUnreachable(out.text.clone()));
    }
    Ok(out.contains("WSDD-Proxy-Server")
        && out.contains("WSDD-MySql-Server")
        && out.contains("WSDD-phpMyAdmin-Server"))
}

/// Checks whether a PHP container exists for the provided version tag.
pub async fn php_container_exists(php_container_tag: &str) -> Result<bool, InfraError> {
    let out = run_docker(vec!["ps".to_string(), "-a".to_string()], None).await?;
    if out.text.contains("Error") {
        return Err(InfraError::DockerUnreachable(out.text.clone()));
    }
    Ok(out.contains(php_container_tag))
}

/// Sets `DOCKER_HOST` for the current user and machine scope.
pub async fn set_docker_host_env(runner: &PsRunner) -> Result<(), InfraError> {
    let cmds = [
        format!(
            r#"[Environment]::SetEnvironmentVariable("DOCKER_HOST", "{DOCKER_HOST_VALUE}", "User")"#
        ),
        format!(
            r#"[Environment]::SetEnvironmentVariable("DOCKER_HOST", "{DOCKER_HOST_VALUE}", "Machine")"#
        ),
    ];

    let runner_ref = runner.clone();
    for cmd in cmds {
        let r = runner_ref.clone();
        tokio::task::spawn_blocking(move || r.run_ps_sync(&cmd, None, None))
            .await
            .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))??;
    }
    Ok(())
}

/// Synchronously checks whether a PHP container exists for the provided version tag.
pub fn php_container_exists_sync(
    runner: &PsRunner,
    php_container_tag: &str,
) -> Result<bool, InfraError> {
    let out = runner.run_direct_sync(env_config().docker_exe(), &["ps", "-a"], None, None)?;
    if out.text.contains("Error") {
        return Err(InfraError::DockerUnreachable(out.text));
    }
    Ok(out.contains(php_container_tag))
}
