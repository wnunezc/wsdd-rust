use std::fmt;

use serde::{Deserialize, Serialize};

/// Runtime status for the PHP container associated with a project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum ProjectStatus {
    /// The container is running.
    Running,
    /// The container exists but is stopped.
    Stopped,
    /// Docker status is unknown or not loaded yet.
    #[default]
    Unknown,
    /// The container reported an error.
    Error(String),
}

impl fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Running => write!(f, "Running"),
            Self::Stopped => write!(f, "Stopped"),
            Self::Unknown => write!(f, "Unknown"),
            Self::Error(message) => write!(f, "Error: {message}"),
        }
    }
}
