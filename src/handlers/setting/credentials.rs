use serde::{Deserialize, Serialize};

use crate::config::environment::path_config;
use crate::errors::InfraError;
use crate::models::project::PhpVersion;

use super::defaults::{default_mysql_database, default_mysql_user};

/// Credentials used by prerequisite containers such as MySQL and phpMyAdmin.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrereqCredentials {
    #[serde(default)]
    pub mysql_database: String,
    #[serde(default)]
    pub mysql_user: String,
    #[serde(default)]
    pub mysql_password: String,
    #[serde(default)]
    pub mysql_root_password: String,
}

impl PrereqCredentials {
    /// Returns true when every required prerequisite credential is present.
    pub fn is_complete(&self) -> bool {
        !self.mysql_database.trim().is_empty()
            && !self.mysql_user.trim().is_empty()
            && !self.mysql_password.trim().is_empty()
            && !self.mysql_root_password.trim().is_empty()
    }

    /// Validates prerequisite credentials before saving them.
    ///
    /// # Errors
    /// Returns [`InfraError::PrerequisiteNotMet`] when a required value is blank.
    pub fn validate_for_save(&self) -> Result<(), InfraError> {
        validate_required("MySQL database", &self.mysql_database)?;
        validate_required("MySQL user", &self.mysql_user)?;
        validate_required("MySQL password", &self.mysql_password)?;
        validate_required("MySQL root password", &self.mysql_root_password)?;
        Ok(())
    }

    pub(super) fn normalize_loaded(mut self) -> Self {
        self.try_fill_from_legacy_init();

        if self.mysql_database.trim().is_empty() {
            self.mysql_database = default_mysql_database();
        }

        if self.mysql_user.trim().is_empty() {
            self.mysql_user = default_mysql_user();
        }

        self
    }

    fn try_fill_from_legacy_init(&mut self) {
        if self.is_complete() {
            return;
        }

        let Ok(content) = std::fs::read_to_string(path_config().init_yml()) else {
            return;
        };

        if self.mysql_database.trim().is_empty() {
            if let Some(value) = find_init_value(&content, "MYSQL_DATABASE") {
                self.mysql_database = value;
            }
        }

        if self.mysql_user.trim().is_empty() {
            if let Some(value) = find_init_value(&content, "MYSQL_USER") {
                self.mysql_user = value;
            }
        }

        if self.mysql_password.trim().is_empty() {
            if let Some(value) = find_init_value(&content, "MYSQL_PASSWORD") {
                self.mysql_password = value;
            }
        }

        if self.mysql_root_password.trim().is_empty() {
            if let Some(value) = find_init_value(&content, "MYSQL_ROOT_PASSWORD") {
                self.mysql_root_password = value;
            }
        }
    }
}

/// Per-PHP-version Webmin credentials.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WebminCredentials {
    pub php_version: PhpVersion,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
}

impl WebminCredentials {
    /// Returns true when both username and password are present.
    pub fn is_complete(&self) -> bool {
        !self.username.trim().is_empty() && !self.password.trim().is_empty()
    }

    /// Returns true when this entry contains no user-provided credential.
    pub fn is_blank(&self) -> bool {
        self.username.trim().is_empty() && self.password.trim().is_empty()
    }

    /// Validates Webmin credentials before saving them.
    ///
    /// # Errors
    /// Returns [`InfraError::PrerequisiteNotMet`] when username or password is blank.
    pub fn validate_for_save(&self) -> Result<(), InfraError> {
        validate_required(
            &format!("Webmin user ({})", self.php_version.display_name()),
            &self.username,
        )?;
        validate_required(
            &format!("Webmin password ({})", self.php_version.display_name()),
            &self.password,
        )?;
        Ok(())
    }

    pub(super) fn normalize_loaded(mut self) -> Self {
        self.username = self.username.trim().to_string();
        self.password = self.password.trim().to_string();
        self
    }
}

pub(super) fn normalize_webmin_credentials(list: &mut Vec<WebminCredentials>) {
    let mut normalized = Vec::new();

    for php_version in PhpVersion::all() {
        if let Some(entry) = list
            .iter()
            .rev()
            .find(|entry| entry.php_version == php_version)
            .cloned()
        {
            let entry = entry.normalize_loaded();
            if !entry.is_blank() {
                normalized.push(entry);
            }
        }
    }

    *list = normalized;
}

fn validate_required(label: &str, value: &str) -> Result<(), InfraError> {
    if value.trim().is_empty() {
        return Err(InfraError::PrerequisiteNotMet(format!(
            "{label} is required"
        )));
    }

    if value.contains(['\r', '\n']) {
        return Err(InfraError::PrerequisiteNotMet(format!(
            "{label} cannot contain line breaks"
        )));
    }

    Ok(())
}

fn find_init_value(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        let prefix = format!("{key}:");

        if !trimmed.starts_with(&prefix) {
            continue;
        }

        let value = trimmed[prefix.len()..].trim();
        let value = strip_yaml_quotes(value);

        if value.is_empty() || value.contains("__WSDD_") || value.contains("${WSDD_") {
            continue;
        }

        return Some(value.to_string());
    }

    None
}

fn strip_yaml_quotes(value: &str) -> &str {
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        let first = bytes[0];
        let last = bytes[value.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return &value[1..value.len() - 1];
        }
    }

    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prereq_credentials_require_non_empty_fields() {
        let err = PrereqCredentials::default()
            .validate_for_save()
            .expect_err("empty credentials should fail");
        assert!(err.to_string().contains("required"));
    }

    #[test]
    fn webmin_credentials_require_complete_values() {
        let err = WebminCredentials {
            php_version: PhpVersion::Php83,
            username: "admin".to_string(),
            password: String::new(),
        }
        .validate_for_save()
        .expect_err("partial webmin credentials should fail");
        assert!(err.to_string().contains("required"));
    }

    #[test]
    fn init_yml_values_are_parsed_for_migration() {
        let content = r#"
services:
  database:
    environment:
      MYSQL_ROOT_PASSWORD: 'root-secret'
      MYSQL_DATABASE: "custom-db"
      MYSQL_USER: custom-user
      MYSQL_PASSWORD: custom-pass
"#;

        assert_eq!(
            find_init_value(content, "MYSQL_ROOT_PASSWORD").as_deref(),
            Some("root-secret")
        );
        assert_eq!(
            find_init_value(content, "MYSQL_DATABASE").as_deref(),
            Some("custom-db")
        );
        assert_eq!(
            find_init_value(content, "MYSQL_USER").as_deref(),
            Some("custom-user")
        );
        assert_eq!(
            find_init_value(content, "MYSQL_PASSWORD").as_deref(),
            Some("custom-pass")
        );
    }
}
