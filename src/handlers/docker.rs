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
//! Stable Docker facade for WSDD.
//!
//! This module preserves the public API used by the UI and other handlers while
//! delegating lifecycle, infrastructure, container, polling, and status concerns
//! to focused submodules.

mod containers;
mod infra;
mod lifecycle;
mod status;
mod types;

use crate::errors::InfraError;
use crate::handlers::ps_script::{OutputSender, ProcOutput, PsRunner};

#[allow(unused_imports)]
pub use types::{
    ContainerInfo, ContainerPollSnapshot, DeployStatus, DockerDesktopStatus, RequirementStatus,
};

/// Shared Docker network used by all WSDD containers.
pub const WSDD_NETWORK: &str = "wsdd-network";

/// Docker Compose project name used by WSDD.
pub const WSDD_PROJECT: &str = "wsdd-projects";

/// Checks whether Docker Desktop is installed.
pub async fn probe_installed(runner: &PsRunner) -> Result<bool, InfraError> {
    lifecycle::probe_installed(runner).await
}

/// Checks whether Docker Desktop has the WSDD-required settings enabled.
pub async fn probe_configured(runner: &PsRunner) -> Result<bool, InfraError> {
    lifecycle::probe_configured(runner).await
}

/// Checks whether Docker Desktop is running and responding.
pub async fn probe_running(runner: &PsRunner) -> Result<bool, InfraError> {
    lifecycle::probe_running(runner).await
}

/// Applies Docker Desktop settings required by WSDD.
pub async fn apply_settings(runner: &PsRunner, tx: Option<OutputSender>) -> Result<(), InfraError> {
    lifecycle::apply_settings(runner, tx).await
}

/// Starts Docker Desktop and waits for readiness.
pub async fn start(runner: &PsRunner, tx: Option<OutputSender>) -> Result<(), InfraError> {
    lifecycle::start(runner, tx).await
}

/// Stops Docker Desktop and related processes.
pub async fn stop(runner: &PsRunner, tx: Option<OutputSender>) -> Result<(), InfraError> {
    lifecycle::stop(runner, tx).await
}

/// Restarts Docker Desktop through the same official lifecycle path.
pub async fn restart(runner: &PsRunner, tx: Option<OutputSender>) -> Result<(), InfraError> {
    lifecycle::restart(runner, tx).await
}

/// Shuts down WSL completely.
pub async fn stop_wsl(runner: &PsRunner, tx: Option<OutputSender>) -> Result<(), InfraError> {
    lifecycle::stop_wsl(runner, tx).await
}

/// Starts the system's default WSL distribution.
pub async fn start_wsl(runner: &PsRunner, tx: Option<OutputSender>) -> Result<(), InfraError> {
    lifecycle::start_wsl(runner, tx).await
}

/// Restarts WSL by running the versioned restart script.
pub async fn restart_wsl(runner: &PsRunner, tx: Option<OutputSender>) -> Result<(), InfraError> {
    lifecycle::restart_wsl(runner, tx).await
}

/// Starts a Docker container by name.
pub async fn start_container(
    container_name: &str,
    tx: Option<OutputSender>,
) -> Result<(), InfraError> {
    containers::start_container(container_name, tx).await
}

/// Stops a Docker container by name.
pub async fn stop_container(
    container_name: &str,
    tx: Option<OutputSender>,
) -> Result<(), InfraError> {
    containers::stop_container(container_name, tx).await
}

/// Restarts a Docker container by name.
pub async fn restart_container(
    container_name: &str,
    tx: Option<OutputSender>,
) -> Result<(), InfraError> {
    containers::restart_container(container_name, tx).await
}

/// Checks whether the shared WSDD Docker network exists.
pub async fn network_exists() -> Result<bool, InfraError> {
    infra::network_exists().await
}

/// Creates the shared WSDD Docker bridge network when missing.
pub async fn ensure_network(
    runner: &PsRunner,
    tx: Option<OutputSender>,
) -> Result<bool, InfraError> {
    infra::ensure_network(runner, tx).await
}

/// Checks whether the phpMyAdmin bind volume exists.
pub async fn pma_volume_exists() -> Result<bool, InfraError> {
    infra::pma_volume_exists().await
}

/// Checks whether the base WSDD containers exist.
pub async fn base_containers_exist() -> Result<bool, InfraError> {
    infra::base_containers_exist().await
}

/// Checks whether a PHP container exists for the provided version tag.
pub async fn php_container_exists(php_container_tag: &str) -> Result<bool, InfraError> {
    infra::php_container_exists(php_container_tag).await
}

/// Sets `DOCKER_HOST` for the current user and machine scope.
pub async fn set_docker_host_env(runner: &PsRunner) -> Result<(), InfraError> {
    infra::set_docker_host_env(runner).await
}

/// Lists WSDD containers asynchronously.
pub async fn list_containers(
    runner: &PsRunner,
    tx: Option<OutputSender>,
) -> Result<Vec<ContainerInfo>, InfraError> {
    containers::list_containers(runner, tx).await
}

/// Applies FullControl permissions to the MySQL data directory.
pub async fn fix_mysql_permissions(
    runner: &PsRunner,
    tx: Option<OutputSender>,
) -> Result<ProcOutput, InfraError> {
    lifecycle::fix_mysql_permissions(runner, tx).await
}

/// Lists WSDD containers synchronously.
pub fn list_containers_sync(runner: &PsRunner) -> Result<Vec<ContainerInfo>, InfraError> {
    containers::list_containers_sync(runner)
}

/// Gathers the UI polling snapshot from a blocking worker thread.
pub async fn gather_poll_snapshot(runner: &PsRunner) -> ContainerPollSnapshot {
    containers::gather_poll_snapshot(runner).await
}

/// Gathers containers plus Docker Desktop status synchronously.
pub fn gather_poll_snapshot_sync(runner: &PsRunner) -> ContainerPollSnapshot {
    containers::gather_poll_snapshot_sync(runner)
}

/// Returns Docker daemon readiness and a lightweight backend process sample.
pub fn docker_desktop_status_sync(runner: &PsRunner) -> DockerDesktopStatus {
    status::docker_desktop_status_sync(runner)
}

/// Starts a container by name synchronously.
pub fn start_container_sync(runner: &PsRunner, name: &str) -> Result<(), InfraError> {
    containers::start_container_sync(runner, name)
}

/// Stops a container by name synchronously.
pub fn stop_container_sync(runner: &PsRunner, name: &str) -> Result<(), InfraError> {
    containers::stop_container_sync(runner, name)
}

/// Restarts a container by name synchronously.
pub fn restart_container_sync(runner: &PsRunner, name: &str) -> Result<(), InfraError> {
    containers::restart_container_sync(runner, name)
}

/// Synchronously checks whether a PHP container exists for the provided version tag.
pub fn php_container_exists_sync(
    runner: &PsRunner,
    php_container_tag: &str,
) -> Result<bool, InfraError> {
    infra::php_container_exists_sync(runner, php_container_tag)
}
