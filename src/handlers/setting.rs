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

// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
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
        Ok(serde_json::from_str(&content)?)
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
}
