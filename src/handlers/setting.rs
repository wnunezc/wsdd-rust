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

const CONFIG_FILE: &str = r"C:\WSDD-Environment\wsdd-config.json";
const CURRENT_CONFIG_VERSION: u32 = 1;

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
    "2.021".to_string()
}
fn default_language() -> Language {
    Language::default()
}
fn default_config_version() -> u32 {
    CURRENT_CONFIG_VERSION
}

// ─────────────────────────────────────────────────────────────────────────────

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
        }
    }
}

impl AppSettings {
    /// Carga la configuracion desde disco. Si no existe, retorna Default.
    pub fn load() -> Result<Self, InfraError> {
        let path = PathBuf::from(CONFIG_FILE);
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)?;
        let settings: Self = serde_json::from_str(&content)?;
        settings.validate_loaded()
    }

    /// Persiste la configuracion en disco.
    pub fn save(&self) -> Result<(), InfraError> {
        let path = PathBuf::from(CONFIG_FILE);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    fn validate_loaded(mut self) -> Result<Self, InfraError> {
        if self.config_version > CURRENT_CONFIG_VERSION {
            return Err(InfraError::UnsupportedConfigVersion {
                found: self.config_version,
                max_supported: CURRENT_CONFIG_VERSION,
            });
        }

        if self.config_version == 0 {
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

        if self.webmin_version.trim().is_empty() {
            self.webmin_version = default_webmin_version();
        }

        Ok(self)
    }
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
        assert_eq!(normalized.webmin_version, "2.021");
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
}
