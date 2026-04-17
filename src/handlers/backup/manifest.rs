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
//! Manifest types and naming helpers for backup archives.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::errors::InfraError;
use crate::models::project::Project;

const MANIFEST_FILE: &str = "manifest.json";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum BackupKind {
    FullEnvironment,
    Project,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct BackupManifest {
    pub kind: BackupKind,
    pub created_on: String,
    pub version: String,
    pub project: Option<Project>,
    pub projects: Vec<ProjectRef>,
    pub docker_images: Vec<String>,
    pub docker_networks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct ProjectRef {
    pub(super) name: String,
    pub(super) domain: String,
    pub(super) work_path: String,
    pub(super) php_version: String,
    pub(super) ssl: bool,
}

/// Returns the default filename for a full environment backup.
pub fn default_full_backup_name() -> String {
    format!("wsdd-backup-full-{}.zip", today_string())
}

/// Returns the default filename for a single-project backup.
pub fn default_project_backup_name(project_name: &str) -> String {
    format!("wsdd-project-{}-{}.zip", project_name, today_string())
}

/// Writes the archive manifest into the staging root.
pub(super) fn write_manifest(root: &Path, manifest: &BackupManifest) -> Result<(), InfraError> {
    let bytes = serde_json::to_vec_pretty(manifest)
        .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;
    fs::write(root.join(MANIFEST_FILE), bytes).map_err(InfraError::Io)
}

/// Reads the archive manifest from an extracted backup root.
pub(super) fn read_manifest(root: &Path) -> Result<BackupManifest, InfraError> {
    let content = fs::read(root.join(MANIFEST_FILE)).map_err(InfraError::Io)?;
    serde_json::from_slice(&content)
        .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))
}

#[cfg(windows)]
pub(super) fn today_string() -> String {
    use windows::Win32::System::SystemInformation::GetLocalTime;

    unsafe {
        let now = GetLocalTime();
        format!("{:04}-{:02}-{:02}", now.wYear, now.wMonth, now.wDay)
    }
}

#[cfg(not(windows))]
pub(super) fn today_string() -> String {
    "1970-01-01".to_string()
}

impl From<&Project> for ProjectRef {
    fn from(project: &Project) -> Self {
        Self {
            name: project.name.clone(),
            domain: project.domain.clone(),
            work_path: project.work_path.clone(),
            php_version: project.php_version.display_name().to_string(),
            ssl: project.ssl,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::project::{EntryPoint, PhpVersion, ProjectStatus};

    fn test_project() -> Project {
        Project {
            name: "demo".to_string(),
            domain: "demo.dock".to_string(),
            php_version: PhpVersion::Php83,
            work_path: r"C:\projects\demo".to_string(),
            entry_point: EntryPoint::Root,
            ssl: true,
            status: ProjectStatus::default(),
        }
    }

    #[test]
    fn project_ref_maps_expected_fields() {
        let project = test_project();
        let snapshot = ProjectRef::from(&project);
        assert_eq!(snapshot.name, "demo");
        assert_eq!(snapshot.domain, "demo.dock");
        assert_eq!(snapshot.php_version, "PHP 8.3");
        assert!(snapshot.ssl);
    }
}
