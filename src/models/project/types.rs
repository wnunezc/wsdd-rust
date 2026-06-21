use serde::{Deserialize, Serialize};

use super::domain::normalize_domain;
use super::entry_point::EntryPoint;
use super::php_version::PhpVersion;
use super::status::ProjectStatus;
use super::storage::{default_project_metadata_version, StorageMode};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManagedNativeStorage {
    pub distro: String,
    pub linux_path: String,
    pub windows_unc: String,
    pub windows_alias: Option<String>,
}

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
    /// Metadata for WSDD-managed WSL-backed project storage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub managed_storage: Option<ManagedNativeStorage>,
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
            managed_storage: None,
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
            managed_storage: None,
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

    #[test]
    fn managed_native_storage_metadata_persists_when_present() {
        let project = Project {
            storage_mode: super::super::StorageMode::ManagedNativeLinuxStorage,
            managed_storage: Some(ManagedNativeStorage {
                distro: "WSDD-Storage".to_string(),
                linux_path: "/home/wsdd/projects/demo".to_string(),
                windows_unc: r"\\wsl.localhost\WSDD-Storage\home\wsdd\projects\demo".to_string(),
                windows_alias: Some(r"C:\WSDD-Projects\demo".to_string()),
            }),
            ..Project::default()
        };

        let json = serde_json::to_string(&project).expect("project should serialize");

        assert!(json.contains("\"storage_mode\":\"managed_native_linux_storage\""));
        assert!(json.contains("\"linux_path\":\"/home/wsdd/projects/demo\""));
        assert!(json.contains(r#""distro":"WSDD-Storage""#));
    }
}
