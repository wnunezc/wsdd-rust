use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::errors::DomainError;

/// PHP version supported by the WSDD Docker environment.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PhpVersion {
    Php56,
    Php72,
    Php74,
    Php81,
    Php82,
    #[default]
    Php83,
    Php84,
}

impl PhpVersion {
    /// Returns all supported PHP versions in chronological order.
    pub fn all() -> Vec<Self> {
        vec![
            Self::Php56,
            Self::Php72,
            Self::Php74,
            Self::Php81,
            Self::Php82,
            Self::Php83,
            Self::Php84,
        ]
    }

    /// Returns the Docker-Structure PHP directory name.
    pub fn dir_name(&self) -> &'static str {
        match self {
            Self::Php56 => "php5.6",
            Self::Php72 => "php7.2",
            Self::Php74 => "php7.4",
            Self::Php81 => "php8.1",
            Self::Php82 => "php8.2",
            Self::Php83 => "php8.3",
            Self::Php84 => "php8.4",
        }
    }

    /// Returns the uppercase Docker container tag.
    pub fn container_tag(&self) -> &'static str {
        match self {
            Self::Php56 => "PHP56",
            Self::Php72 => "PHP72",
            Self::Php74 => "PHP74",
            Self::Php81 => "PHP81",
            Self::Php82 => "PHP82",
            Self::Php83 => "PHP83",
            Self::Php84 => "PHP84",
        }
    }

    /// Returns the lowercase Docker Compose tag.
    pub fn compose_tag(&self) -> &'static str {
        match self {
            Self::Php56 => "php56",
            Self::Php72 => "php72",
            Self::Php74 => "php74",
            Self::Php81 => "php81",
            Self::Php82 => "php82",
            Self::Php83 => "php83",
            Self::Php84 => "php84",
        }
    }

    /// Returns the user-facing PHP version label.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Php56 => "PHP 5.6",
            Self::Php72 => "PHP 7.2",
            Self::Php74 => "PHP 7.4",
            Self::Php81 => "PHP 8.1",
            Self::Php82 => "PHP 8.2",
            Self::Php83 => "PHP 8.3",
            Self::Php84 => "PHP 8.4",
        }
    }

    /// Returns the numeric suffix used by base container domains.
    pub fn host_suffix(&self) -> &'static str {
        match self {
            Self::Php56 => "56",
            Self::Php72 => "72",
            Self::Php74 => "74",
            Self::Php81 => "81",
            Self::Php82 => "82",
            Self::Php83 => "83",
            Self::Php84 => "84",
        }
    }

    /// Returns PHP, cron, and Webmin base domains for this PHP version.
    pub fn base_container_domains(&self) -> [String; 3] {
        let suffix = self.host_suffix();
        [
            format!("php{suffix}.wsdd.dock"),
            format!("cron{suffix}.wsdd.dock"),
            format!("wm{suffix}.wsdd.dock"),
        ]
    }

    /// Infers a PHP version from a WSDD container name.
    pub fn from_container_name(name: &str) -> Option<Self> {
        Self::all()
            .into_iter()
            .find(|version| name.contains(version.container_tag()))
    }
}

impl FromStr for PhpVersion {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace(' ', "").as_str() {
            "php5.6" | "php56" | "5.6" => Ok(Self::Php56),
            "php7.2" | "php72" | "7.2" => Ok(Self::Php72),
            "php7.4" | "php74" | "7.4" => Ok(Self::Php74),
            "php8.1" | "php81" | "8.1" => Ok(Self::Php81),
            "php8.2" | "php82" | "8.2" => Ok(Self::Php82),
            "php8.3" | "php83" | "8.3" => Ok(Self::Php83),
            "php8.4" | "php84" | "8.4" => Ok(Self::Php84),
            _ => Err(DomainError::InvalidPhpVersion(s.to_string())),
        }
    }
}

impl fmt::Display for PhpVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn php_version_parse_various_formats() {
        assert_eq!("php8.3".parse::<PhpVersion>().unwrap(), PhpVersion::Php83);
        assert_eq!("php83".parse::<PhpVersion>().unwrap(), PhpVersion::Php83);
        assert_eq!("8.3".parse::<PhpVersion>().unwrap(), PhpVersion::Php83);
        assert_eq!("PHP 8.3".parse::<PhpVersion>().unwrap(), PhpVersion::Php83);
    }

    #[test]
    fn php_version_parse_invalid() {
        assert!("php9.9".parse::<PhpVersion>().is_err());
        assert!("".parse::<PhpVersion>().is_err());
    }

    #[test]
    fn php_version_tags_are_correct() {
        let version = PhpVersion::Php83;
        assert_eq!(version.dir_name(), "php8.3");
        assert_eq!(version.container_tag(), "PHP83");
        assert_eq!(version.compose_tag(), "php83");
    }

    #[test]
    fn base_container_domains_match_expected_wsdd_urls() {
        let urls = PhpVersion::Php83.base_container_domains();
        assert_eq!(urls[0], "php83.wsdd.dock");
        assert_eq!(urls[1], "cron83.wsdd.dock");
        assert_eq!(urls[2], "wm83.wsdd.dock");
    }

    #[test]
    fn from_container_name_detects_php_version() {
        let version = PhpVersion::from_container_name("WSDD-Web-Server-PHP84");
        assert_eq!(version, Some(PhpVersion::Php84));
    }
}
