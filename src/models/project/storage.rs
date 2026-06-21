use serde::{Deserialize, Serialize};

/// Storage/runtime strategy used to mount a project into WSDD PHP containers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageMode {
    /// Current WSDD behavior: bind a Windows project path into Docker.
    #[default]
    ClassicBind,
    /// Keep Windows as source of truth and run PHP from a synchronized fast workspace.
    FastSyncWorkspace,
    /// Run a project that already lives in Linux/WSL/Docker-native storage.
    NativeLinuxProject,
    /// Create and manage a project under the WSDD-owned WSL storage distro.
    ManagedNativeLinuxStorage,
}

impl StorageMode {
    /// Stable key used by i18n, persisted diagnostics, and tests.
    pub fn key(self) -> &'static str {
        match self {
            Self::ClassicBind => "classic_bind",
            Self::FastSyncWorkspace => "fast_sync_workspace",
            Self::NativeLinuxProject => "native_linux_project",
            Self::ManagedNativeLinuxStorage => "managed_native_linux_storage",
        }
    }

    /// Returns all user-selectable storage modes in UI order.
    pub fn all() -> [Self; 4] {
        [
            Self::ClassicBind,
            Self::FastSyncWorkspace,
            Self::NativeLinuxProject,
            Self::ManagedNativeLinuxStorage,
        ]
    }
}

/// Returns true when a path can represent storage owned by Linux/WSL instead of Windows.
pub fn is_native_linux_path(path: &str) -> bool {
    let trimmed = path.trim();
    trimmed.starts_with('/')
        || trimmed.starts_with(r"\\wsl$")
        || trimmed.starts_with(r"\\wsl.localhost\")
}

/// Current project metadata schema for WSDD 2.0 project JSON files.
pub const CURRENT_PROJECT_METADATA_VERSION: u32 = 2;

/// Default metadata version applied when loading legacy project JSON.
pub fn default_project_metadata_version() -> u32 {
    CURRENT_PROJECT_METADATA_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_linux_path_accepts_linux_and_wsl_locations() {
        assert!(is_native_linux_path("/home/wsdd/projects/app"));
        assert!(is_native_linux_path(
            r"\\wsl.localhost\Ubuntu\home\wsdd\projects\app"
        ));
        assert!(is_native_linux_path(
            r"\\wsl$\Ubuntu\home\wsdd\projects\app"
        ));
    }

    #[test]
    fn native_linux_path_rejects_windows_locations() {
        assert!(!is_native_linux_path(r"C:\projects\app"));
        assert!(!is_native_linux_path(
            r"D:\OpsZone\DevWorkspace\Projects\Web\app"
        ));
    }
}
