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
//! Persistencia de proyectos WSDD en disco.
//!
//! Equivalente a la parte de almacenamiento de `Handlers/HandlerProject.cs`.
//! La lógica de despliegue (docker volumes, vhost, SSL) vive en fases posteriores.
//!
//! # Almacenamiento
//!
//! Cada proyecto se guarda como un archivo JSON independiente:
//! `C:\WSDD-Environment\Docker-Structure\projects\{nombre}.json`
//!
//! El C# usaba XML — Rust usa JSON (serde_json) para coherencia con `wsdd-config.json`.

use std::path::{Path, PathBuf};

use crate::errors::InfraError;
use crate::models::project::Project;

/// Directorio donde se guardan los archivos de proyectos.
const PROJECTS_DIR: &str = r"C:\WSDD-Environment\Docker-Structure\projects";

// ─── API pública ──────────────────────────────────────────────────────────────

/// Guarda un proyecto en disco (crea o sobreescribe).
///
/// El nombre del archivo se deriva de `project.name` — debe ser único.
///
/// # Errors
/// [`InfraError::Io`] si el directorio no se puede crear o el archivo no se puede escribir.
pub fn save(project: &Project) -> Result<(), InfraError> {
    let dir = Path::new(PROJECTS_DIR);
    std::fs::create_dir_all(dir).map_err(InfraError::Io)?;

    let json = serde_json::to_string_pretty(project)
        .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;

    std::fs::write(project_path(&project.name), json).map_err(InfraError::Io)
}

/// Retorna la lista de todos los proyectos guardados.
///
/// Los archivos que no se puedan deserializar se omiten con un warning en tracing.
///
/// # Errors
/// [`InfraError::Io`] si el directorio no existe y no se puede crear.
pub fn list_all() -> Result<Vec<Project>, InfraError> {
    let dir = Path::new(PROJECTS_DIR);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(dir).map_err(InfraError::Io)?;
    let mut projects = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        match load_from_file(&path) {
            Ok(p) => projects.push(p),
            Err(e) => tracing::warn!(file = %path.display(), error = %e, "no se pudo cargar proyecto"),
        }
    }

    // Orden alfabético por nombre para consistencia en la UI
    projects.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(projects)
}

/// Retorna `true` si existe un proyecto con ese nombre.
pub fn exists(name: &str) -> bool {
    project_path(name).exists()
}

/// Elimina el archivo de proyecto del disco.
///
/// No es un error si el proyecto no existe.
///
/// # Errors
/// [`InfraError::Io`] si el archivo existe pero no se puede eliminar.
pub fn delete(name: &str) -> Result<(), InfraError> {
    let path = project_path(name);
    if path.exists() {
        std::fs::remove_file(&path).map_err(InfraError::Io)?;
    }
    Ok(())
}

// ─── Helpers privados ─────────────────────────────────────────────────────────

fn project_path(name: &str) -> PathBuf {
    Path::new(PROJECTS_DIR).join(format!("{name}.json"))
}

fn load_from_file(path: &Path) -> Result<Project, InfraError> {
    let content = std::fs::read_to_string(path).map_err(InfraError::Io)?;
    serde_json::from_str(&content)
        .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::project::{EntryPoint, PhpVersion, Project};

    fn test_project(name: &str) -> Project {
        Project {
            name: name.to_string(),
            domain: format!("{name}.dock"),
            php_version: PhpVersion::Php83,
            work_path: format!(r"C:\projects\{name}"),
            entry_point: EntryPoint::Root,
            ssl: true,
            status: Default::default(),
        }
    }

    /// Directorio temporal para tests — evita tocar PROJECTS_DIR en producción.
    fn with_temp_dir<F: FnOnce(&Path)>(f: F) {
        let dir = std::env::temp_dir().join("wsdd_project_tests");
        let _ = std::fs::create_dir_all(&dir);
        f(&dir);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_and_load() {
        with_temp_dir(|dir| {
            let p = test_project("testapp");
            let path = dir.join("testapp.json");

            let json = serde_json::to_string_pretty(&p).unwrap();
            std::fs::write(&path, json).unwrap();

            let loaded: Project = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
            assert_eq!(loaded.name, "testapp");
            assert_eq!(loaded.domain, "testapp.dock");
            assert_eq!(loaded.php_version, PhpVersion::Php83);
        });
    }

    #[test]
    fn exists_returns_false_for_unknown() {
        // No puede depender de PROJECTS_DIR real — solo verificamos la lógica de path
        assert!(!exists("proyecto_que_no_existe_xyz_abc_123"));
    }
}
