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
//! Container operations, listing, and polling snapshots.

use super::status::docker_desktop_status_sync;
use super::types::{ContainerInfo, ContainerPollSnapshot};
use crate::config::environment::env_config;
use crate::errors::InfraError;
use crate::handlers::ps_script::{run_docker, OutputSender, PsRunner, ScriptRunner};

/// Starts a Docker container by name.
pub async fn start_container(
    container_name: &str,
    tx: Option<OutputSender>,
) -> Result<(), InfraError> {
    let args = vec!["start".to_string(), container_name.to_string()];
    let out = run_docker(args, tx).await?;
    if out.success {
        Ok(())
    } else {
        Err(InfraError::DockerUnreachable(format!(
            "No se pudo iniciar {container_name}: {}",
            out.text
        )))
    }
}

/// Stops a Docker container by name.
pub async fn stop_container(
    container_name: &str,
    tx: Option<OutputSender>,
) -> Result<(), InfraError> {
    let args = vec!["stop".to_string(), container_name.to_string()];
    let out = run_docker(args, tx).await?;
    if out.success {
        Ok(())
    } else {
        Err(InfraError::DockerUnreachable(format!(
            "No se pudo detener {container_name}: {}",
            out.text
        )))
    }
}

/// Restarts a Docker container by name.
pub async fn restart_container(
    container_name: &str,
    tx: Option<OutputSender>,
) -> Result<(), InfraError> {
    let args = vec!["restart".to_string(), container_name.to_string()];
    let out = run_docker(args, tx).await?;
    if out.success {
        Ok(())
    } else {
        Err(InfraError::DockerUnreachable(format!(
            "No se pudo reiniciar {container_name}: {}",
            out.text
        )))
    }
}

/// Lists WSDD containers asynchronously.
pub async fn list_containers(
    runner: &PsRunner,
    tx: Option<OutputSender>,
) -> Result<Vec<ContainerInfo>, InfraError> {
    let args = vec![
        "ps".to_string(),
        "-a".to_string(),
        "--format".to_string(),
        "{{.ID}}|{{.Names}}|{{.Image}}|{{.Ports}}|{{.Status}}".to_string(),
        "--filter".to_string(),
        "name=WSDD-".to_string(),
    ];
    let result = run_docker(args, tx).await?;
    let containers = parse_container_list(&result.text, runner.clone()).await;
    Ok(containers)
}

/// Lists WSDD containers synchronously.
pub fn list_containers_sync(runner: &PsRunner) -> Result<Vec<ContainerInfo>, InfraError> {
    let args = [
        "ps",
        "-a",
        "--format",
        "{{.ID}}|{{.Names}}|{{.Image}}|{{.Ports}}|{{.Status}}",
        "--filter",
        "name=WSDD-",
    ];
    let result = runner.run_direct_sync(env_config().docker_exe(), &args, None, None)?;
    Ok(parse_container_list_sync(&result.text, runner))
}

/// Gathers the UI polling snapshot from a blocking worker thread.
pub async fn gather_poll_snapshot(runner: &PsRunner) -> ContainerPollSnapshot {
    let runner = runner.clone();
    tokio::task::spawn_blocking(move || gather_poll_snapshot_sync(&runner))
        .await
        .unwrap_or_default()
}

/// Gathers containers plus Docker Desktop status synchronously.
pub fn gather_poll_snapshot_sync(runner: &PsRunner) -> ContainerPollSnapshot {
    ContainerPollSnapshot {
        containers: list_containers_sync(runner).unwrap_or_default(),
        docker_status: docker_desktop_status_sync(runner),
    }
}

/// Starts a container by name synchronously.
pub fn start_container_sync(runner: &PsRunner, name: &str) -> Result<(), InfraError> {
    let out = runner.run_direct_sync(env_config().docker_exe(), &["start", name], None, None)?;
    if out.success {
        Ok(())
    } else {
        Err(InfraError::DockerUnreachable(format!(
            "No se pudo iniciar {name}: {}",
            out.text
        )))
    }
}

/// Stops a container by name synchronously.
pub fn stop_container_sync(runner: &PsRunner, name: &str) -> Result<(), InfraError> {
    let out = runner.run_direct_sync(env_config().docker_exe(), &["stop", name], None, None)?;
    if out.success {
        Ok(())
    } else {
        Err(InfraError::DockerUnreachable(format!(
            "No se pudo detener {name}: {}",
            out.text
        )))
    }
}

/// Restarts a container by name synchronously.
pub fn restart_container_sync(runner: &PsRunner, name: &str) -> Result<(), InfraError> {
    let out = runner.run_direct_sync(env_config().docker_exe(), &["restart", name], None, None)?;
    if out.success {
        Ok(())
    } else {
        Err(InfraError::DockerUnreachable(format!(
            "No se pudo reiniciar {name}: {}",
            out.text
        )))
    }
}

async fn parse_container_list(output: &str, runner: PsRunner) -> Vec<ContainerInfo> {
    let mut containers = Vec::new();

    for line in output.lines() {
        if line.contains("error") || line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 5 {
            continue;
        }

        let name = parts[1].to_string();
        let r = runner.clone();
        let cmd = format!(
            "{} exec {name} printenv VIRTUAL_HOST",
            env_config().docker_exe()
        );
        let urls_raw = tokio::task::spawn_blocking(move || r.run_ps_sync(&cmd, None, None))
            .await
            .ok()
            .and_then(|r| r.ok())
            .map(|o| o.text)
            .unwrap_or_default();

        let urls = parse_virtual_hosts(&urls_raw);

        containers.push(ContainerInfo {
            id: parts[0].to_string(),
            name,
            image: parts[2].to_string(),
            ports: parts[3].to_string(),
            status: parts[4].to_string(),
            urls,
        });
    }

    containers
}

fn parse_container_list_sync(output: &str, runner: &PsRunner) -> Vec<ContainerInfo> {
    output
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.to_lowercase().contains("error"))
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() < 5 {
                return None;
            }
            let name = parts[1].to_string();
            Some(ContainerInfo {
                id: parts[0].to_string(),
                name: name.clone(),
                image: parts[2].to_string(),
                ports: parts[3].to_string(),
                status: parts[4].to_string(),
                urls: fetch_container_urls_sync(runner, &name),
            })
        })
        .collect()
}

fn fetch_container_urls_sync(runner: &PsRunner, name: &str) -> Vec<String> {
    let cmd = format!(
        "{} exec {name} printenv VIRTUAL_HOST",
        env_config().docker_exe()
    );
    runner
        .run_ps_sync(&cmd, None, None)
        .map(|out| parse_virtual_hosts(&out.text))
        .unwrap_or_default()
}

fn parse_virtual_hosts(output: &str) -> Vec<String> {
    output
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_virtual_hosts_splits_and_trims_urls() {
        let urls = parse_virtual_hosts("php84.wsdd.dock, cron84.wsdd.dock, wm84.wsdd.dock");
        assert_eq!(
            urls,
            vec![
                "php84.wsdd.dock".to_string(),
                "cron84.wsdd.dock".to_string(),
                "wm84.wsdd.dock".to_string()
            ]
        );
    }
}
