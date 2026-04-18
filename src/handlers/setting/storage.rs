use serde::{Deserialize, Serialize};

use crate::config::environment::{path_config, path_to_string};
use crate::errors::InfraError;
use crate::i18n::Language;
use crate::models::project::PhpVersion;

use super::credentials::{normalize_webmin_credentials, PrereqCredentials, WebminCredentials};
use super::defaults::{
    default_config_version, default_language, default_log_max_lines, default_php_memory_limit,
    default_php_timezone, default_php_upload_max_filesize, default_webmin_version,
    default_xdebug_enabled, CURRENT_CONFIG_VERSION, LEGACY_WEBMIN_PASSWORD, LEGACY_WEBMIN_USER,
    LEGACY_WEBMIN_VERSION_2021,
};
use super::secrets;
use super::services::OptionalServicesSettings;
use super::theme::AppTheme;

/// User-editable application settings loaded from `wsdd-config.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Persisted schema version.
    #[serde(default = "default_config_version")]
    pub config_version: u32,
    pub setup_completed: bool,
    pub docker_path: Option<String>,
    pub projects_path: String,
    pub wsl_distro: Option<String>,
    pub selected_monitor: i32,
    /// Active UI language.
    #[serde(default = "default_language")]
    pub language: Language,
    /// Active UI color theme.
    #[serde(default)]
    pub theme: AppTheme,
    /// Maximum number of lines kept in the main log panel.
    #[serde(default = "default_log_max_lines")]
    pub log_max_lines: usize,
    /// PHP `memory_limit` used by generated containers.
    #[serde(default = "default_php_memory_limit")]
    pub php_memory_limit: String,
    /// PHP upload/post size used by generated containers.
    #[serde(default = "default_php_upload_max_filesize")]
    pub php_upload_max_filesize: String,
    /// PHP timezone used by generated containers.
    #[serde(default = "default_php_timezone")]
    pub php_timezone: String,
    /// Whether Xdebug is installed and enabled in generated PHP containers.
    #[serde(default = "default_xdebug_enabled")]
    pub xdebug_enabled: bool,
    /// Webmin package version installed in PHP containers.
    #[serde(default = "default_webmin_version")]
    pub webmin_version: String,
    /// Whether WSDD containers should start automatically on app startup.
    #[serde(default)]
    pub auto_start_containers: bool,
    /// Optional developer services that are deployed only after explicit activation.
    #[serde(default)]
    pub optional_services: OptionalServicesSettings,
    /// Credentials for prerequisite containers.
    #[serde(default)]
    pub prereq_credentials: PrereqCredentials,
    /// Webmin credentials per PHP version.
    #[serde(default)]
    pub webmin_credentials: Vec<WebminCredentials>,
}

#[derive(Serialize)]
struct AppSettingsDisk<'a> {
    config_version: u32,
    setup_completed: bool,
    docker_path: &'a Option<String>,
    projects_path: &'a String,
    wsl_distro: &'a Option<String>,
    selected_monitor: i32,
    language: Language,
    theme: AppTheme,
    log_max_lines: usize,
    php_memory_limit: &'a String,
    php_upload_max_filesize: &'a String,
    php_timezone: &'a String,
    xdebug_enabled: bool,
    webmin_version: &'a String,
    auto_start_containers: bool,
    optional_services: &'a OptionalServicesSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            config_version: default_config_version(),
            setup_completed: false,
            docker_path: None,
            projects_path: path_to_string(path_config().default_projects_root()),
            wsl_distro: None,
            selected_monitor: 0,
            language: default_language(),
            theme: AppTheme::default(),
            log_max_lines: default_log_max_lines(),
            php_memory_limit: default_php_memory_limit(),
            php_upload_max_filesize: default_php_upload_max_filesize(),
            php_timezone: default_php_timezone(),
            xdebug_enabled: default_xdebug_enabled(),
            webmin_version: default_webmin_version(),
            auto_start_containers: false,
            optional_services: OptionalServicesSettings::default(),
            prereq_credentials: PrereqCredentials::default(),
            webmin_credentials: Vec::new(),
        }
    }
}

impl<'a> From<&'a AppSettings> for AppSettingsDisk<'a> {
    fn from(settings: &'a AppSettings) -> Self {
        Self {
            config_version: settings.config_version,
            setup_completed: settings.setup_completed,
            docker_path: &settings.docker_path,
            projects_path: &settings.projects_path,
            wsl_distro: &settings.wsl_distro,
            selected_monitor: settings.selected_monitor,
            language: settings.language,
            theme: settings.theme,
            log_max_lines: settings.log_max_lines,
            php_memory_limit: &settings.php_memory_limit,
            php_upload_max_filesize: &settings.php_upload_max_filesize,
            php_timezone: &settings.php_timezone,
            xdebug_enabled: settings.xdebug_enabled,
            webmin_version: &settings.webmin_version,
            auto_start_containers: settings.auto_start_containers,
            optional_services: &settings.optional_services,
        }
    }
}

impl AppSettings {
    /// Loads application settings from disk or returns defaults when missing.
    ///
    /// # Errors
    /// Returns [`InfraError`] when JSON cannot be parsed or a future schema is found.
    pub fn load() -> Result<Self, InfraError> {
        let path = path_config().config_file();
        let mut settings = if !path.exists() {
            Self::default()
        } else {
            let content = std::fs::read_to_string(&path)?;
            let settings: Self = serde_json::from_str(&content)?;
            settings.validate_loaded()?
        };

        if let Some(secrets) = secrets::load()? {
            settings.prereq_credentials = secrets.prereq_credentials.normalize_loaded();
            settings.webmin_credentials = secrets.webmin_credentials;
            normalize_webmin_credentials(&mut settings.webmin_credentials);
        }

        Ok(settings)
    }

    /// Saves public settings and secrets to their separate JSON files.
    ///
    /// # Errors
    /// Returns [`InfraError`] when serialization or filesystem writes fail.
    pub fn save(&self) -> Result<(), InfraError> {
        let path = path_config().config_file();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&AppSettingsDisk::from(self))?;
        std::fs::write(&path, content)?;

        secrets::save(&self.prereq_credentials, &self.webmin_credentials)?;
        Ok(())
    }

    fn validate_loaded(mut self) -> Result<Self, InfraError> {
        if self.config_version > CURRENT_CONFIG_VERSION {
            return Err(InfraError::UnsupportedConfigVersion {
                found: self.config_version,
                max_supported: CURRENT_CONFIG_VERSION,
            });
        }

        if self.config_version == 0 || self.config_version < CURRENT_CONFIG_VERSION {
            self.config_version = default_config_version();
        }

        if self.projects_path.trim().is_empty() {
            self.projects_path = Self::default().projects_path;
        }

        if self.log_max_lines == 0 {
            self.log_max_lines = default_log_max_lines();
        }

        if self.php_memory_limit.trim().is_empty() {
            self.php_memory_limit = default_php_memory_limit();
        }

        if self.php_upload_max_filesize.trim().is_empty() {
            self.php_upload_max_filesize = default_php_upload_max_filesize();
        }

        if self.php_timezone.trim().is_empty() {
            self.php_timezone = default_php_timezone();
        }

        self.webmin_version = normalize_webmin_version(&self.webmin_version);
        self.optional_services.mailpit.virtual_host = self
            .optional_services
            .mailpit
            .virtual_host
            .trim()
            .to_string();
        if self.optional_services.mailpit.virtual_host.is_empty() {
            self.optional_services.mailpit.virtual_host =
                OptionalServicesSettings::default().mailpit.virtual_host;
        }
        self.optional_services.validate()?;
        self.prereq_credentials = self.prereq_credentials.normalize_loaded();
        normalize_webmin_credentials(&mut self.webmin_credentials);

        Ok(self)
    }

    /// Validates prerequisite credentials before save/deploy.
    ///
    /// # Errors
    /// Returns [`InfraError`] when a required value is missing.
    pub fn validate_prerequisite_credentials(&self) -> Result<(), InfraError> {
        self.prereq_credentials.validate_for_save()
    }

    /// Validates non-blank Webmin credential entries.
    ///
    /// # Errors
    /// Returns [`InfraError`] when an entry is partial.
    pub fn validate_webmin_credentials(&self) -> Result<(), InfraError> {
        for credentials in &self.webmin_credentials {
            if credentials.is_blank() {
                continue;
            }
            credentials.validate_for_save()?;
        }
        Ok(())
    }

    /// Validates optional developer service configuration.
    ///
    /// # Errors
    /// Returns [`InfraError`] when service ports are invalid or conflicting.
    pub fn validate_optional_services(&self) -> Result<(), InfraError> {
        self.optional_services.validate()
    }

    /// Returns complete Webmin credentials for a PHP version.
    pub fn webmin_credentials_for(&self, php_version: &PhpVersion) -> Option<&WebminCredentials> {
        self.webmin_credentials
            .iter()
            .find(|entry| &entry.php_version == php_version && entry.is_complete())
    }

    /// Returns the stored Webmin credential entry for a PHP version, even if partial.
    pub fn webmin_credentials_entry(&self, php_version: &PhpVersion) -> Option<&WebminCredentials> {
        self.webmin_credentials
            .iter()
            .find(|entry| &entry.php_version == php_version)
    }

    /// Updates or removes the editable Webmin credential entry for a PHP version.
    pub fn set_webmin_credentials_draft(
        &mut self,
        php_version: PhpVersion,
        username: String,
        password: String,
    ) {
        let username = username.trim().to_string();
        let password = password.trim().to_string();

        if username.is_empty() && password.is_empty() {
            self.webmin_credentials
                .retain(|entry| entry.php_version != php_version);
            return;
        }

        if let Some(existing) = self
            .webmin_credentials
            .iter_mut()
            .find(|entry| entry.php_version == php_version)
        {
            existing.username = username;
            existing.password = password;
        } else {
            self.webmin_credentials.push(WebminCredentials {
                php_version,
                username,
                password,
            });
        }

        normalize_webmin_credentials(&mut self.webmin_credentials);
    }

    /// Stores a validated Webmin credential entry.
    ///
    /// # Errors
    /// Returns [`InfraError`] when the entry is incomplete.
    pub fn store_webmin_credentials(
        &mut self,
        credentials: WebminCredentials,
    ) -> Result<(), InfraError> {
        let credentials = credentials.normalize_loaded();
        credentials.validate_for_save()?;
        self.set_webmin_credentials_draft(
            credentials.php_version,
            credentials.username,
            credentials.password,
        );
        Ok(())
    }

    /// Backfills legacy Webmin admin credentials for a PHP version when missing.
    pub fn ensure_legacy_webmin_credentials(&mut self, php_version: PhpVersion) -> bool {
        if self.webmin_credentials_for(&php_version).is_some() {
            return false;
        }

        self.set_webmin_credentials_draft(
            php_version,
            LEGACY_WEBMIN_USER.to_string(),
            LEGACY_WEBMIN_PASSWORD.to_string(),
        );
        true
    }
}

fn normalize_webmin_version(value: &str) -> String {
    let trimmed = value.trim();

    if trimmed.is_empty() || trimmed == LEGACY_WEBMIN_VERSION_2021 {
        return default_webmin_version();
    }

    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_use_current_config_version() {
        let settings = AppSettings::default();
        assert_eq!(settings.config_version, CURRENT_CONFIG_VERSION);
        assert!(settings.xdebug_enabled);
    }

    #[test]
    fn legacy_settings_are_normalized() {
        let settings = AppSettings {
            config_version: 0,
            projects_path: String::new(),
            log_max_lines: 0,
            php_memory_limit: String::new(),
            php_upload_max_filesize: String::new(),
            php_timezone: String::new(),
            xdebug_enabled: false,
            webmin_version: String::new(),
            ..AppSettings::default()
        };

        let normalized = settings
            .validate_loaded()
            .expect("legacy config should load");
        assert_eq!(normalized.config_version, CURRENT_CONFIG_VERSION);
        assert_eq!(normalized.projects_path, r"C:\WSDD-Projects");
        assert_eq!(normalized.log_max_lines, 500);
        assert_eq!(normalized.php_memory_limit, "512M");
        assert_eq!(normalized.php_upload_max_filesize, "256M");
        assert_eq!(normalized.php_timezone, "UTC");
        assert!(!normalized.xdebug_enabled);
        assert!(!normalized.optional_services.redis.enabled);
        assert!(!normalized.optional_services.mailpit.enabled);
        assert!(!normalized.optional_services.memcached.enabled);
        assert_eq!(normalized.webmin_version, "2.630");
        assert_eq!(
            normalized.prereq_credentials.mysql_database,
            "wsdd-database"
        );
        assert_eq!(normalized.prereq_credentials.mysql_user, "tester");
    }

    #[test]
    fn future_config_version_is_rejected() {
        let settings = AppSettings {
            config_version: CURRENT_CONFIG_VERSION + 1,
            ..AppSettings::default()
        };

        let err = settings
            .validate_loaded()
            .expect_err("future config must fail");
        match err {
            InfraError::UnsupportedConfigVersion {
                found,
                max_supported,
            } => {
                assert_eq!(found, CURRENT_CONFIG_VERSION + 1);
                assert_eq!(max_supported, CURRENT_CONFIG_VERSION);
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn set_webmin_credentials_draft_removes_blank_entries() {
        let mut settings = AppSettings::default();
        settings.set_webmin_credentials_draft(
            PhpVersion::Php83,
            "admin".to_string(),
            "secret".to_string(),
        );
        assert!(settings
            .webmin_credentials_for(&PhpVersion::Php83)
            .is_some());

        settings.set_webmin_credentials_draft(PhpVersion::Php83, String::new(), String::new());
        assert!(settings
            .webmin_credentials_for(&PhpVersion::Php83)
            .is_none());
    }

    #[test]
    fn ensure_legacy_webmin_credentials_backfills_admin_defaults() {
        let mut settings = AppSettings::default();
        assert!(settings.ensure_legacy_webmin_credentials(PhpVersion::Php82));

        let credentials = settings
            .webmin_credentials_for(&PhpVersion::Php82)
            .expect("legacy credentials should be inserted");
        assert_eq!(credentials.username, "admin");
        assert_eq!(credentials.password, "admin");
    }

    #[test]
    fn legacy_webmin_version_is_upgraded_to_current_value() {
        let settings = AppSettings {
            webmin_version: "2.021".to_string(),
            ..AppSettings::default()
        };

        let normalized = settings
            .validate_loaded()
            .expect("legacy webmin version should normalize");

        assert_eq!(normalized.webmin_version, "2.630");
    }
}
