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
//! Base Docker environment provisioning facade.

mod containers;
mod resources;

use crate::errors::InfraError;
use crate::handlers::log_types::LogSender;
use crate::handlers::ps_script::PsRunner;

/// Applies `DOCKER_HOST`, network, volume, and base container provisioning.
pub(super) fn deploy_environment_sync(runner: &PsRunner, tx: &LogSender) -> Result<(), InfraError> {
    resources::set_docker_host_env_sync(runner, tx)?;
    resources::ensure_network_sync(runner, tx)?;
    resources::create_pma_volume_sync(runner, tx)?;
    containers::deploy_base_containers_sync(runner, tx)?;
    containers::show_running_containers_sync(runner, tx);
    Ok(())
}
