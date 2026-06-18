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
}

impl StorageMode {
    /// Stable key used by i18n, persisted diagnostics, and tests.
    pub fn key(self) -> &'static str {
        match self {
            Self::ClassicBind => "classic_bind",
            Self::FastSyncWorkspace => "fast_sync_workspace",
            Self::NativeLinuxProject => "native_linux_project",
        }
    }

    /// Returns all user-selectable storage modes in UI order.
    pub fn all() -> [Self; 3] {
        [
            Self::ClassicBind,
            Self::FastSyncWorkspace,
            Self::NativeLinuxProject,
        ]
    }
}

/// Current project metadata schema for WSDD 2.0 project JSON files.
pub const CURRENT_PROJECT_METADATA_VERSION: u32 = 2;

/// Default metadata version applied when loading legacy project JSON.
pub fn default_project_metadata_version() -> u32 {
    CURRENT_PROJECT_METADATA_VERSION
}
