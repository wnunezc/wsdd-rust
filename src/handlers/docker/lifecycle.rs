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
//! Docker Desktop and WSL lifecycle operations backed by versioned PS1 scripts.

use crate::errors::InfraError;
use crate::handlers::ps_script::{OutputSender, ProcOutput, PsRunner, ScriptRunner};

/// Checks whether Docker Desktop is installed.
pub async fn probe_installed(runner: &PsRunner) -> Result<bool, InfraError> {
    let runner = runner.clone();
    let result = tokio::task::spawn_blocking(move || {
        runner.run_script_sync("dd-isinstalled.ps1", None, None)
    })
    .await
    .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))??;

    Ok(result.contains("Installed"))
}

/// Checks whether Docker Desktop has the WSDD-required settings enabled.
pub async fn probe_configured(runner: &PsRunner) -> Result<bool, InfraError> {
    let runner = runner.clone();
    let result = tokio::task::spawn_blocking(move || {
        runner.run_script_sync("dd-issettingup.ps1", None, None)
    })
    .await
    .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))??;

    Ok(result.contains("Updated"))
}

/// Checks whether Docker Desktop is running and responding.
pub async fn probe_running(runner: &PsRunner) -> Result<bool, InfraError> {
    let runner = runner.clone();
    let result =
        tokio::task::spawn_blocking(move || runner.run_script_sync("dd-isrunning.ps1", None, None))
            .await
            .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))??;

    Ok(result.contains("Running"))
}

/// Applies Docker Desktop settings required by WSDD.
pub async fn apply_settings(runner: &PsRunner, tx: Option<OutputSender>) -> Result<(), InfraError> {
    let runner = runner.clone();
    let result = tokio::task::spawn_blocking(move || {
        runner.run_script_sync("dd-setting.ps1", None, tx.as_ref())
    })
    .await
    .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))??;

    if result.contains("Continue") {
        Ok(())
    } else {
        Err(InfraError::UnexpectedOutput(
            "dd-setting.ps1".to_string(),
            result.text.clone(),
        ))
    }
}

/// Starts Docker Desktop and waits for readiness.
pub async fn start(runner: &PsRunner, tx: Option<OutputSender>) -> Result<(), InfraError> {
    let runner = runner.clone();
    tokio::task::spawn_blocking(move || runner.run_script_sync("dd-start.ps1", None, tx.as_ref()))
        .await
        .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?
        .map(|_| ())
}

/// Stops Docker Desktop and related processes.
pub async fn stop(runner: &PsRunner, tx: Option<OutputSender>) -> Result<(), InfraError> {
    let runner = runner.clone();
    tokio::task::spawn_blocking(move || runner.run_script_sync("dd-stop.ps1", None, tx.as_ref()))
        .await
        .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?
        .map(|_| ())
}

/// Restarts Docker Desktop through the same official lifecycle path.
pub async fn restart(runner: &PsRunner, tx: Option<OutputSender>) -> Result<(), InfraError> {
    stop(runner, tx.clone()).await?;
    start(runner, tx).await
}

/// Shuts down WSL completely.
pub async fn stop_wsl(runner: &PsRunner, tx: Option<OutputSender>) -> Result<(), InfraError> {
    let runner = runner.clone();
    tokio::task::spawn_blocking(move || {
        runner.run_script_sync("wsl-shutdown.ps1", None, tx.as_ref())
    })
    .await
    .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?
    .map(|_| ())
}

/// Starts the system's default WSL distribution.
pub async fn start_wsl(runner: &PsRunner, tx: Option<OutputSender>) -> Result<(), InfraError> {
    let runner = runner.clone();
    tokio::task::spawn_blocking(move || runner.run_script_sync("wsl-start.ps1", None, tx.as_ref()))
        .await
        .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?
        .map(|_| ())
}

/// Restarts WSL by running the versioned restart script.
pub async fn restart_wsl(runner: &PsRunner, tx: Option<OutputSender>) -> Result<(), InfraError> {
    let runner = runner.clone();
    tokio::task::spawn_blocking(move || {
        runner.run_script_sync("wsl-restart.ps1", None, tx.as_ref())
    })
    .await
    .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?
    .map(|_| ())
}

/// Applies FullControl permissions to the MySQL data directory.
pub async fn fix_mysql_permissions(
    runner: &PsRunner,
    tx: Option<OutputSender>,
) -> Result<ProcOutput, InfraError> {
    let runner = runner.clone();
    tokio::task::spawn_blocking(move || {
        runner.run_script_sync("dd-fixmysqlpermission.ps1", None, tx.as_ref())
    })
    .await
    .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?
}
