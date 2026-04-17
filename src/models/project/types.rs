use serde::{Deserialize, Serialize};

use super::domain::normalize_domain;
use super::entry_point::EntryPoint;
use super::php_version::PhpVersion;
use super::status::ProjectStatus;

/// Web project managed by WSDD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
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
}
