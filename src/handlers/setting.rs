// WebStack Deployer for Docker
// Copyright (c) 2026 Walter Nunez / Icaros Net S.A
// All Rights Reserved.
//
// This software is provided for development use only.
// Unauthorized commercial use is prohibited.
//
// Redistribution and modification allowed only through
// the official GitHub repository.
//
// This software is provided AS IS, without warranty of any kind.
// The author shall not be liable for any damages.
//
// Contact: wnunez@lh-2.net
// Equivalente a Handlers/HandlerSetting.cs
// Persistencia de configuracion en JSON (reemplaza XML de la version C#)

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::errors::InfraError;
use crate::i18n::Language;
use crate::models::project::PhpVersion;

const CONFIG_FILE: &str = r"C:\WSDD-Environment\wsdd-config.json";
const SECRETS_FILE: &str = r"C:\WSDD-Environment\wsdd-secrets.json";
const LEGACY_INIT_YML: &str = r"C:\WSDD-Environment\Docker-Structure\init.yml";
const CURRENT_CONFIG_VERSION: u32 = 4;
const LEGACY_WEBMIN_USER: &str = "admin";
const LEGACY_WEBMIN_PASSWORD: &str = "admin";
const CURRENT_WEBMIN_VERSION: &str = "2.630";
const LEGACY_WEBMIN_VERSION_2021: &str = "2.021";

/// Paleta de colores de la UI. Persistida en wsdd-config.json.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum AppTheme {
    /// Gris oscuro neutro — estilo VS Code.
    #[default]
    NeutralDark,
    /// Fondo con tinte índigo — estilo DevOps/terminal.
    BlueTint,
    /// Tinte cálido — estilo Monokai/Dracula.
    WarmDark,
    /// Fondo blanco/gris claro — estilo sistema operativo.
    Light,
}

impl AppTheme {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::NeutralDark => "Dark Neutral",
            Self::BlueTint => "Dark Blue",
            Self::WarmDark => "Dark Warm",
            Self::Light => "Light",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::NeutralDark,
            Self::BlueTint,
            Self::WarmDark,
            Self::Light,
        ]
    }
}

// ── Funciones default para serde ─────────────────────────────────────────────

fn default_log_max_lines() -> usize {
    500
}
fn default_php_memory_limit() -> String {
    "512M".to_string()
}
fn default_php_upload_max_filesize() -> String {
    "256M".to_string()
}
fn default_php_timezone() -> String {
    "UTC".to_string()
}
fn default_webmin_version() -> String {
    CURRENT_WEBMIN_VERSION.to_string()
}
fn default_mysql_database() -> String {
    "wsdd-database".to_string()
}
fn default_mysql_user() -> String {
    "tester".to_string()
}
fn default_language() -> Language {
    Language::default()
}
fn default_config_version() -> u32 {
    CURRENT_CONFIG_VERSION
}

// ─────────────────────────────────────────────────────────────────────────────

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
    pub fn is_complete(&self) -> bool {
        !self.mysql_database.trim().is_empty()
            && !self.mysql_user.trim().is_empty()
            && !self.mysql_password.trim().is_empty()
            && !self.mysql_root_password.trim().is_empty()
    }

    pub fn validate_for_save(&self) -> Result<(), InfraError> {
        validate_required("MySQL database", &self.mysql_database)?;
        validate_required("MySQL user", &self.mysql_user)?;
        validate_required("MySQL password", &self.mysql_password)?;
        validate_required("MySQL root password", &self.mysql_root_password)?;
        Ok(())
    }

    fn normalize_loaded(mut self) -> Self {
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

        let Ok(content) = std::fs::read_to_string(LEGACY_INIT_YML) else {
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WebminCredentials {
    pub php_version: PhpVersion,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
}

impl WebminCredentials {
    pub fn is_complete(&self) -> bool {
        !self.username.trim().is_empty() && !self.password.trim().is_empty()
    }

    pub fn is_blank(&self) -> bool {
        self.username.trim().is_empty() && self.password.trim().is_empty()
    }

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

    fn normalize_loaded(mut self) -> Self {
        self.username = self.username.trim().to_string();
        self.password = self.password.trim().to_string();
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct AppSecrets {
    #[serde(default)]
    prereq_credentials: PrereqCredentials,
    #[serde(default)]
    webmin_credentials: Vec<WebminCredentials>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Versión del esquema persistido de `wsdd-config.json`.
    #[serde(default = "default_config_version")]
    pub config_version: u32,
    pub setup_completed: bool,
    pub docker_path: Option<String>,
    pub projects_path: String,
    pub wsl_distro: Option<String>,
    pub selected_monitor: i32,
    /// Idioma activo de la UI. Default: English.
    #[serde(default = "default_language")]
    pub language: Language,
    /// Tema de color activo. Default: NeutralDark.
    #[serde(default)]
    pub theme: AppTheme,

    // ── Configuracion de log ──────────────────────────────────────────────
    /// Numero maximo de lineas a conservar en el panel de log.
    #[serde(default = "default_log_max_lines")]
    pub log_max_lines: usize,

    // ── PHP (aplicado al generar contenedores) ────────────────────────────
    /// php.ini memory_limit para contenedores generados.
    #[serde(default = "default_php_memory_limit")]
    pub php_memory_limit: String,

    /// php.ini upload_max_filesize / post_max_size para contenedores generados.
    #[serde(default = "default_php_upload_max_filesize")]
    pub php_upload_max_filesize: String,

    /// Timezone de PHP para contenedores generados (ej: "America/Mexico_City").
    #[serde(default = "default_php_timezone")]
    pub php_timezone: String,

    // ── Herramientas integradas ───────────────────────────────────────────
    /// Version de Webmin instalada en los contenedores PHP.
    #[serde(default = "default_webmin_version")]
    pub webmin_version: String,

    /// Iniciar contenedores WSDD automaticamente al arrancar la aplicacion.
    #[serde(default)]
    pub auto_start_containers: bool,

    /// Credenciales para los contenedores prerequisito (MySQL + phpMyAdmin).
    #[serde(default)]
    pub prereq_credentials: PrereqCredentials,

    /// Credenciales de Webmin por version de PHP.
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
    webmin_version: &'a String,
    auto_start_containers: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            config_version: default_config_version(),
            setup_completed: false,
            docker_path: None,
            projects_path: r"C:\WSDD-Projects".to_string(),
            wsl_distro: None,
            selected_monitor: 0,
            language: default_language(),
            theme: AppTheme::default(),
            log_max_lines: default_log_max_lines(),
            php_memory_limit: default_php_memory_limit(),
            php_upload_max_filesize: default_php_upload_max_filesize(),
            php_timezone: default_php_timezone(),
            webmin_version: default_webmin_version(),
            auto_start_containers: false,
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
            webmin_version: &settings.webmin_version,
            auto_start_containers: settings.auto_start_containers,
        }
    }
}

impl AppSettings {
    /// Carga la configuracion desde disco. Si no existe, retorna Default.
    pub fn load() -> Result<Self, InfraError> {
        let path = PathBuf::from(CONFIG_FILE);
        let mut settings = if !path.exists() {
            Self::default()
        } else {
            let content = std::fs::read_to_string(&path)?;
            let settings: Self = serde_json::from_str(&content)?;
            settings.validate_loaded()?
        };

        if let Some(secrets) = load_secrets_file()? {
            settings.prereq_credentials = secrets.prereq_credentials.normalize_loaded();
            settings.webmin_credentials = secrets.webmin_credentials;
            normalize_webmin_credentials(&mut settings.webmin_credentials);
        }

        Ok(settings)
    }

    /// Persiste la configuracion en disco.
    pub fn save(&self) -> Result<(), InfraError> {
        let path = PathBuf::from(CONFIG_FILE);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&AppSettingsDisk::from(self))?;
        std::fs::write(&path, content)?;

        let secrets_path = PathBuf::from(SECRETS_FILE);
        if let Some(parent) = secrets_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let secrets = AppSecrets {
            prereq_credentials: self.prereq_credentials.clone(),
            webmin_credentials: self.webmin_credentials.clone(),
        };
        let secrets_content = serde_json::to_string_pretty(&secrets)?;
        std::fs::write(&secrets_path, secrets_content)?;
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

        self.prereq_credentials = self.prereq_credentials.normalize_loaded();
        normalize_webmin_credentials(&mut self.webmin_credentials);

        Ok(self)
    }

    pub fn validate_prerequisite_credentials(&self) -> Result<(), InfraError> {
        self.prereq_credentials.validate_for_save()
    }

    pub fn validate_webmin_credentials(&self) -> Result<(), InfraError> {
        for credentials in &self.webmin_credentials {
            if credentials.is_blank() {
                continue;
            }
            credentials.validate_for_save()?;
        }
        Ok(())
    }

    pub fn webmin_credentials_for(&self, php_version: &PhpVersion) -> Option<&WebminCredentials> {
        self.webmin_credentials
            .iter()
            .find(|entry| &entry.php_version == php_version && entry.is_complete())
    }

    pub fn webmin_credentials_entry(&self, php_version: &PhpVersion) -> Option<&WebminCredentials> {
        self.webmin_credentials
            .iter()
            .find(|entry| &entry.php_version == php_version)
    }

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

fn normalize_webmin_credentials(list: &mut Vec<WebminCredentials>) {
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

fn load_secrets_file() -> Result<Option<AppSecrets>, InfraError> {
    let path = PathBuf::from(SECRETS_FILE);
    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(path)?;
    let mut secrets: AppSecrets = serde_json::from_str(&content)?;
    secrets.prereq_credentials = secrets.prereq_credentials.normalize_loaded();
    normalize_webmin_credentials(&mut secrets.webmin_credentials);
    Ok(Some(secrets))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_use_current_config_version() {
        let settings = AppSettings::default();
        assert_eq!(settings.config_version, CURRENT_CONFIG_VERSION);
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
