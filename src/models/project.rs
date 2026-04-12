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
//! Modelos de dominio para proyectos WSDD.
//!
//! Tipos puros — sin I/O, sin dependencias de infraestructura.
//! Las validaciones de negocio viven aquí; la lógica de despliegue en `handlers/`.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::errors::DomainError;

// ─── PhpVersion ───────────────────────────────────────────────────────────────

/// Versión PHP soportada por el entorno WSDD.
///
/// Tipo fuerte que reemplaza strings mágicos como `"php8.3"` o `"PHP83"`.
/// Garantiza en tiempo de compilación que solo se usan versiones con
/// Dockerfiles disponibles en `Docker-Structure/bin/`.
///
/// # Ejemplo
/// ```rust
/// let v: PhpVersion = "php8.3".parse().unwrap();
/// assert_eq!(v.dir_name(), "php8.3");
/// assert_eq!(v.container_tag(), "PHP83");
/// ```
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
    /// Retorna todas las versiones disponibles en orden cronológico.
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

    /// Nombre del directorio de la versión en `Docker-Structure/bin/`.
    ///
    /// Equivalente a `PhpLabel` en C#. Ejemplo: `"php8.3"`.
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

    /// Tag para naming de contenedores Docker en MAYÚSCULAS.
    ///
    /// Equivalente a `PhpVersion` en C#. Ejemplo: `"PHP83"`.
    /// Usado en: `WSDD-Web-Server-PHP83`, filtros de `docker ps`.
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

    /// Tag en minúsculas para archivos Docker Compose.
    ///
    /// Ejemplo: `"php83"`. Usado en: `webserver.php83.yml`, `options.php83.yml`.
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

    /// Nombre para mostrar al usuario en la UI.
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

    /// Sufijo numérico usado en dominios base del contenedor.
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

    /// Dominios base del contenedor PHP asociados a esta versión.
    pub fn base_container_domains(&self) -> [String; 3] {
        let suffix = self.host_suffix();
        [
            format!("php{suffix}.wsdd.dock"),
            format!("cron{suffix}.wsdd.dock"),
            format!("wm{suffix}.wsdd.dock"),
        ]
    }

    /// Intenta inferir la versión PHP a partir del nombre del contenedor WSDD.
    pub fn from_container_name(name: &str) -> Option<Self> {
        Self::all()
            .into_iter()
            .find(|version| name.contains(version.container_tag()))
    }
}

/// Permite parsear desde strings en múltiples formatos.
///
/// Formatos aceptados: `"php8.3"`, `"php83"`, `"8.3"`, `"PHP 8.3"` (case-insensitive).
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

// ─── EntryPoint ───────────────────────────────────────────────────────────────

/// Punto de entrada del servidor web del proyecto.
///
/// Determina qué subdirectorio se mapea como raíz del DocumentRoot en Apache.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum EntryPoint {
    /// Sin subdirectorio — la raíz del proyecto es el DocumentRoot.
    #[default]
    Root,
    /// Directorio `/public` — común en Laravel, Symfony.
    Public,
    /// Directorio personalizado especificado por el usuario.
    Custom(String),
}

impl EntryPoint {
    /// Retorna el path relativo para el template de vhost.
    ///
    /// Equivalente al campo `EntryPoint` de C# usado en `tpl.vhost.conf`.
    pub fn as_path(&self) -> &str {
        match self {
            Self::Root => "",
            Self::Public => "/public",
            Self::Custom(p) => p.as_str(),
        }
    }
}

impl fmt::Display for EntryPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_path())
    }
}

// ─── ProjectStatus ────────────────────────────────────────────────────────────

/// Estado actual del contenedor PHP asociado al proyecto.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum ProjectStatus {
    /// El contenedor está en ejecución.
    Running,
    /// El contenedor existe pero está detenido.
    Stopped,
    /// Estado desconocido (Docker no responde o proyecto no desplegado aún).
    #[default]
    Unknown,
    /// El contenedor reportó un error.
    Error(String),
}

impl fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Running => write!(f, "Running"),
            Self::Stopped => write!(f, "Stopped"),
            Self::Unknown => write!(f, "Unknown"),
            Self::Error(msg) => write!(f, "Error: {msg}"),
        }
    }
}

// ─── Project ──────────────────────────────────────────────────────────────────

/// Proyecto web gestionado por WSDD.
///
/// Equivalente a la clase `Project` de C#, con los siguientes cambios:
/// - `PhpLabel` + `PhpVersion` unificados en el enum [`PhpVersion`]
/// - `CustomUrl` → `domain` (siempre incluye el sufijo `.dock`)
/// - Añadido [`EntryPoint`] que en C# era un string
/// - [`ProjectStatus`] para seguimiento en tiempo real
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Nombre identificador del proyecto (sin espacios, usado como label).
    pub name: String,

    /// Dominio local del proyecto — siempre termina en `.dock`.
    /// Ejemplo: `"miapp.dock"`
    pub domain: String,

    /// Versión PHP del contenedor web asignado.
    pub php_version: PhpVersion,

    /// Ruta absoluta al directorio del código fuente en Windows.
    /// Se monta como volumen Docker: `work_path` → `/var/www/html/{domain}`.
    pub work_path: String,

    /// Subdirectorio que actúa como raíz del DocumentRoot de Apache.
    pub entry_point: EntryPoint,

    /// Si `true`, genera certificado SSL con mkcert y configura HTTPS.
    pub ssl: bool,

    /// Estado actual del contenedor asociado (actualizado en runtime, no persiste).
    #[serde(skip)]
    pub status: ProjectStatus,
}

impl Project {
    /// Crea un proyecto normalizando el dominio (agrega `.dock` automáticamente).
    pub fn new(
        name: String,
        domain_input: &str,
        php_version: PhpVersion,
        work_path: String,
    ) -> Self {
        let domain = normalize_domain(domain_input);
        Self {
            name,
            domain,
            php_version,
            work_path,
            entry_point: EntryPoint::default(),
            ssl: true,
            status: ProjectStatus::default(),
        }
    }
}

impl Default for Project {
    fn default() -> Self {
        Self {
            name: String::new(),
            domain: String::new(),
            php_version: PhpVersion::default(),
            work_path: String::new(),
            entry_point: EntryPoint::default(),
            ssl: true,
            status: ProjectStatus::default(),
        }
    }
}

/// Normaliza un dominio: elimina prefijos/sufijos comunes y agrega `.dock`.
pub fn normalize_domain(input: &str) -> String {
    let s = input.trim().to_lowercase();

    // Eliminar prefijos comunes
    let s = s
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.");

    // Eliminar sufijos comunes
    let s = s
        .trim_end_matches('/')
        .trim_end_matches(".com")
        .trim_end_matches(".net")
        .trim_end_matches(".local")
        .trim_end_matches(".dock");

    format!("{s}.dock")
}

// ─── Tests ────────────────────────────────────────────────────────────────────

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
        let v = PhpVersion::Php83;
        assert_eq!(v.dir_name(), "php8.3");
        assert_eq!(v.container_tag(), "PHP83");
        assert_eq!(v.compose_tag(), "php83");
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

    #[test]
    fn normalize_domain_adds_dock_suffix() {
        assert_eq!(normalize_domain("myapp"), "myapp.dock");
        assert_eq!(normalize_domain("myapp.dock"), "myapp.dock");
        assert_eq!(normalize_domain("http://myapp.com"), "myapp.dock");
        assert_eq!(normalize_domain("www.myapp.net/"), "myapp.dock");
    }

    #[test]
    fn entry_point_path_values() {
        assert_eq!(EntryPoint::Root.as_path(), "");
        assert_eq!(EntryPoint::Public.as_path(), "/public");
        assert_eq!(EntryPoint::Custom("/api".to_string()).as_path(), "/api");
    }

    #[test]
    fn project_new_normalizes_domain() {
        let p = Project::new(
            "Test".to_string(),
            "https://www.testapp.com",
            PhpVersion::Php83,
            "C:\\projects\\test".to_string(),
        );
        assert_eq!(p.domain, "testapp.dock");
    }
}
