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
//! Public Docker state models used by handlers and UI polling.

/// Information about one Docker container managed by WSDD.
#[derive(Debug, Clone)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub ports: String,
    pub status: String,
    /// URLs read from the container `VIRTUAL_HOST` environment variable.
    pub urls: Vec<String>,
}

impl ContainerInfo {
    /// Returns `true` when Docker reports the container as running.
    pub fn is_running(&self) -> bool {
        let status_lower = self.status.to_lowercase();
        status_lower.contains("up")
            || status_lower.contains("running")
            || status_lower.contains("started")
    }
}

/// Lightweight Docker Desktop status shown in the main status bar.
#[derive(Debug, Clone, Default)]
pub struct DockerDesktopStatus {
    pub daemon_ready: bool,
    pub cpu_percent: Option<f32>,
    pub memory_mb: Option<u64>,
    pub process_count: usize,
    pub process_name: Option<String>,
}

/// Combined container and Docker Desktop snapshot for UI polling.
#[derive(Debug, Clone, Default)]
pub struct ContainerPollSnapshot {
    pub containers: Vec<ContainerInfo>,
    pub docker_status: DockerDesktopStatus,
}

/// Requirement probe result for Docker Desktop.
#[derive(Debug, Clone)]
pub struct RequirementStatus {
    pub installed: bool,
    pub configured: bool,
    pub running: bool,
}

impl RequirementStatus {
    /// Returns `true` only when every Docker prerequisite is ready.
    pub fn is_ready(&self) -> bool {
        self.installed && self.configured && self.running
    }
}

/// Deployment status for the base Docker environment.
#[derive(Debug, Clone)]
pub struct DeployStatus {
    pub network_ok: bool,
    pub volume_ok: bool,
    pub containers_ok: bool,
}

impl DeployStatus {
    /// Returns `true` when network, volume, and base containers are ready.
    pub fn is_complete(&self) -> bool {
        self.network_ok && self.volume_ok && self.containers_ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_info_is_running() {
        let c = ContainerInfo {
            id: "abc".to_string(),
            name: "WSDD-Proxy-Server".to_string(),
            image: "nginx".to_string(),
            ports: "80/tcp".to_string(),
            status: "Up 2 hours".to_string(),
            urls: vec![],
        };
        assert!(c.is_running());
    }

    #[test]
    fn container_info_not_running() {
        let c = ContainerInfo {
            id: "abc".to_string(),
            name: "WSDD-MySql-Server".to_string(),
            image: "mysql".to_string(),
            ports: String::new(),
            status: "Exited (0) 1 hour ago".to_string(),
            urls: vec![],
        };
        assert!(!c.is_running());
    }

    #[test]
    fn requirement_status_is_ready() {
        let s = RequirementStatus {
            installed: true,
            configured: true,
            running: true,
        };
        assert!(s.is_ready());
    }

    #[test]
    fn requirement_status_not_ready_if_not_running() {
        let s = RequirementStatus {
            installed: true,
            configured: true,
            running: false,
        };
        assert!(!s.is_ready());
    }
}
