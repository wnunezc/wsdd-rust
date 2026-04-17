use std::fmt;

use serde::{Deserialize, Serialize};

/// Web server entry point used as Apache DocumentRoot suffix.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum EntryPoint {
    /// Project root is the DocumentRoot.
    #[default]
    Root,
    /// `/public` is the DocumentRoot.
    Public,
    /// Custom user-provided subdirectory.
    Custom(String),
}

impl EntryPoint {
    /// Returns the relative path used by vhost templates.
    pub fn as_path(&self) -> &str {
        match self {
            Self::Root => "",
            Self::Public => "/public",
            Self::Custom(path) => path.as_str(),
        }
    }
}

impl fmt::Display for EntryPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_point_path_values() {
        assert_eq!(EntryPoint::Root.as_path(), "");
        assert_eq!(EntryPoint::Public.as_path(), "/public");
        assert_eq!(EntryPoint::Custom("/api".to_string()).as_path(), "/api");
    }
}
