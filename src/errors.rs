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
//! Jerarquía de errores tipados para WSDD.
//!
//! Se divide en tres capas independientes siguiendo el principio de
//! separación de responsabilidades:
//!
//! - [`DomainError`]: errores de reglas de negocio puras (sin I/O).
//! - [`InfraError`]: errores de infraestructura (procesos, archivos, red).
//! - [`AppError`]: error raíz que unifica ambas capas para la UI.
//!
//! # Uso
//! ```rust,ignore
//! use crate::errors::{AppError, InfraError};
//!
//! fn some_handler() -> Result<(), InfraError> {
//!     // Infraestructura retorna InfraError
//!     Ok(())
//! }
//!
//! fn use_case() -> Result<(), AppError> {
//!     some_handler()?; // InfraError → AppError via From
//!     Ok(())
//! }
//! ```

use thiserror::Error;

// ─── Errores de Dominio ───────────────────────────────────────────────────────

/// Errores de reglas de negocio puras — no involucran I/O.
///
/// Son independientes de PowerShell, Docker o el sistema de archivos.
/// Se pueden probar con tests unitarios sin infraestructura real.
#[derive(Debug, Error, PartialEq)]
pub enum DomainError {
    /// La cadena no corresponde a ninguna versión PHP soportada por WSDD.
    #[error(
        "Versión PHP inválida: '{0}'. Versiones soportadas: 5.6, 7.2, 7.4, 8.1, 8.2, 8.3, 8.4"
    )]
    InvalidPhpVersion(String),

    /// El dominio ingresado contiene caracteres no válidos o está vacío.
    #[error("Dominio inválido: '{0}'")]
    InvalidDomain(String),

    /// Ya existe un proyecto registrado con ese nombre.
    #[error("El proyecto '{0}' ya existe en el entorno WSDD")]
    ProjectAlreadyExists(String),

    /// El path indicado no existe o no es accesible.
    #[error("El directorio del proyecto no existe: '{0}'")]
    WorkPathNotFound(String),
}

// ─── Errores de Infraestructura ───────────────────────────────────────────────

/// Errores de infraestructura — procesos externos, archivos, Docker, red.
#[derive(Debug, Error)]
pub enum InfraError {
    /// Un script PowerShell falló o no produjo el output esperado.
    ///
    /// # Campos
    /// - `0`: nombre del script (e.g. `"dd-isinstalled.ps1"`)
    /// - `1`: descripción del fallo
    #[error("Script '{0}' falló: {1}")]
    ScriptFailed(String, String),

    /// Error de I/O del sistema operativo (archivo no encontrado, permisos, etc.)
    #[error("Error de I/O: {0}")]
    Io(#[from] std::io::Error),

    /// Docker Desktop no responde o no está instalado/corriendo.
    #[error("Docker no responde: {0}")]
    DockerUnreachable(String),

    /// Un script produjo output que no contiene los tokens esperados.
    ///
    /// # Campos
    /// - `0`: nombre del script
    /// - `1`: output recibido (para diagnóstico)
    #[error("Output inesperado del script '{0}': '{1}'")]
    UnexpectedOutput(String, String),

    /// El ejecutable del proceso no existe en PATH o en la ruta indicada.
    #[error("Proceso no encontrado: {0}")]
    ProcessNotFound(String),

    /// Un prerequisito del sistema no está satisfecho.
    #[error("Prerequisito no satisfecho: {0}")]
    PrerequisiteNotMet(String),

    /// Error al serializar/deserializar JSON.
    #[error("Error de serialización JSON: {0}")]
    Json(#[from] serde_json::Error),
}

// ─── Error de Aplicación (raíz) ──────────────────────────────────────────────

/// Error raíz de la aplicación — unifica dominio e infraestructura.
///
/// Es el tipo que la UI consume para mostrar mensajes de error al usuario.
/// Los errores de capas inferiores se convierten automáticamente via `From`.
#[derive(Debug, Error)]
pub enum AppError {
    /// Error de regla de negocio.
    #[error(transparent)]
    Domain(#[from] DomainError),

    /// Error de infraestructura (proceso, archivo, Docker).
    #[error(transparent)]
    Infra(#[from] InfraError),

    /// Configuración inválida o incompleta.
    #[error("Configuración inválida: {0}")]
    Config(String),

    /// El usuario canceló la operación en curso.
    #[error("Operación cancelada")]
    Cancelled,
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_error_formats_correctly() {
        let e = DomainError::InvalidPhpVersion("php9.9".to_string());
        assert!(e.to_string().contains("php9.9"));
    }

    #[test]
    fn infra_error_converts_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let infra: InfraError = io_err.into();
        assert!(infra.to_string().contains("I/O"));
    }

    #[test]
    fn app_error_converts_from_domain() {
        let domain = DomainError::ProjectAlreadyExists("myapp".to_string());
        let app: AppError = domain.into();
        assert!(app.to_string().contains("myapp"));
    }

    #[test]
    fn app_error_converts_from_infra() {
        let infra = InfraError::PrerequisiteNotMet("Docker Desktop".to_string());
        let app: AppError = infra.into();
        assert!(app.to_string().contains("Docker Desktop"));
    }
}
