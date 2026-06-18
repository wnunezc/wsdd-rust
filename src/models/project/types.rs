use serde::{Deserialize, Serialize};

use super::domain::normalize_domain;
use super::entry_point::EntryPoint;
use super::php_version::PhpVersion;
use super::status::ProjectStatus;
use super::storage::{default_project_metadata_version, StorageMode};

/// Web project managed by WSDD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Versioned metadata schema. Missing legacy values are normalized to WSDD 2.0.
    #[serde(default = "default_project_metadata_version")]
    pub metadata_version: u32,
    /// Project identifier used by WSDD.
    pub name: String,
    /// Local `.dock` domain.
    pub domain: String,
    /// Assigned PHP runtime version.
    pub php_version: PhpVersion,
    /// Absolute Windows path mounted into the PHP container.
    pub work_path: String,
    /// Apache DocumentRoot suffix.
    pub entry_point: EntryPoint,
    /// Whether WSDD should generate local HTTPS certificates.
    pub ssl: bool,
    /// Storage/runtime mode used by the Docker deploy flow.
    #[serde(default)]
    pub storage_mode: StorageMode,
    /// Runtime-only container status.
    #[serde(skip)]
    pub status: ProjectStatus,
}

impl Project {
    /// Creates a project with a normalized `.dock` domain.
    pub fn new(
        name: String,
        domain_input: &str,
        php_version: PhpVersion,
        work_path: String,
    ) -> Self {
        let domain = normalize_domain(domain_input);
        Self {
            name,
            domain,
            php_version,
            work_path,
            entry_point: EntryPoint::default(),
            ssl: true,
            metadata_version: default_project_metadata_version(),
            storage_mode: StorageMode::default(),
            status: ProjectStatus::default(),
        }
    }
}

impl Default for Project {
    fn default() -> Self {
        Self {
            name: String::new(),
            domain: String::new(),
            php_version: PhpVersion::default(),
            work_path: String::new(),
            entry_point: EntryPoint::default(),
            ssl: true,
            metadata_version: default_project_metadata_version(),
            storage_mode: StorageMode::default(),
            status: ProjectStatus::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_new_normalizes_domain() {
        let project = Project::new(
            "Test".to_string(),
            "https://www.testapp.com",
            PhpVersion::Php83,
            "C:\\projects\\test".to_string(),
        );
        assert_eq!(project.domain, "testapp.dock");
    }

    #[test]
    fn legacy_project_json_defaults_to_classic_bind_storage() {
        let legacy = r#"{
          "name": "legacy",
          "domain": "legacy.dock",
          "php_version": "Php83",
          "work_path": "C:\\projects\\legacy",
          "entry_point": "Root",
          "ssl": true
        }"#;

        let project: Project = serde_json::from_str(legacy).expect("legacy project should load");

        assert_eq!(project.storage_mode, super::super::StorageMode::ClassicBind);
        assert_eq!(project.metadata_version, 2);
    }

    #[test]
    fn project_json_persists_storage_mode_metadata() {
        let project = Project {
            storage_mode: super::super::StorageMode::FastSyncWorkspace,
            ..Project::default()
        };

        let json = serde_json::to_string(&project).expect("project should serialize");

        assert!(json.contains("\"metadata_version\":2"));
        assert!(json.contains("\"storage_mode\":\"fast_sync_workspace\""));
    }
}
