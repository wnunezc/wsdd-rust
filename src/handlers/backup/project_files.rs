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
//! Project filesystem staging helpers for backup and restore operations.

use std::fs;
use std::path::{Path, PathBuf};

use super::archive::copy_dir_recursive;
use super::{FULL_PROJECTS_DIR, PROJECT_CERTS_DIR};
use crate::config::environment::path_config;
use crate::errors::InfraError;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::models::project::Project;

/// Stages the source directory for one project into a full environment backup.
pub(super) fn stage_project_source(
    root: &Path,
    project: &Project,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let source = Path::new(&project.work_path);
    if !source.exists() {
        let _ = tx.send(LogLine::warn(format!(
            "[Backup] Omitiendo source de '{}' porque no existe: {}",
            project.name,
            source.display()
        )));
        return Ok(());
    }

    let dest = root
        .join(FULL_PROJECTS_DIR)
        .join(&project.name)
        .join("workdir");
    copy_dir_recursive(source, &dest)
}

/// Stages project certificate files when SSL is enabled.
pub(super) fn stage_project_certs(root: &Path, project: &Project) -> Result<(), InfraError> {
    if !project.ssl {
        return Ok(());
    }

    let (cert_file, key_file) = ssl_file_paths(&project.domain);
    let dest_dir = root.join(PROJECT_CERTS_DIR);
    let staged_cert = dest_dir.join(format!("{}.crt", project.domain));
    let staged_key = dest_dir.join(format!("{}.key", project.domain));

    if cert_file.exists() || key_file.exists() {
        fs::create_dir_all(&dest_dir).map_err(InfraError::Io)?;
    }

    if cert_file.exists() {
        fs::copy(&cert_file, &staged_cert).map_err(InfraError::Io)?;
    }
    if key_file.exists() {
        fs::copy(&key_file, &staged_key).map_err(InfraError::Io)?;
    }
    Ok(())
}

/// Restores staged project certificate files into the WSDD SSL directory.
pub(super) fn restore_project_certs(root: &Path, project: &Project) -> Result<(), InfraError> {
    if !project.ssl {
        return Ok(());
    }

    let certs_stage = root.join(PROJECT_CERTS_DIR);
    if !certs_stage.exists() {
        return Ok(());
    }

    let (target_cert, target_key) = ssl_file_paths(&project.domain);
    if let Some(parent) = target_cert.parent() {
        fs::create_dir_all(parent).map_err(InfraError::Io)?;
    }

    let staged_cert = certs_stage.join(format!("{}.crt", project.domain));
    let staged_key = certs_stage.join(format!("{}.key", project.domain));
    let legacy_cert = certs_stage.join("cert.pem");
    let legacy_key = certs_stage.join("key.pem");

    if staged_cert.exists() {
        fs::copy(staged_cert, &target_cert).map_err(InfraError::Io)?;
    } else if legacy_cert.exists() {
        fs::copy(legacy_cert, &target_cert).map_err(InfraError::Io)?;
    }

    if staged_key.exists() {
        fs::copy(staged_key, &target_key).map_err(InfraError::Io)?;
    } else if legacy_key.exists() {
        fs::copy(legacy_key, &target_key).map_err(InfraError::Io)?;
    }

    Ok(())
}

/// Rejects project restore into a non-empty destination directory.
pub(super) fn validate_restore_work_path(project: &Project) -> Result<(), InfraError> {
    let path = Path::new(&project.work_path);
    if !path.exists() {
        return Ok(());
    }

    let mut entries = fs::read_dir(path).map_err(InfraError::Io)?;
    if entries.next().is_some() {
        return Err(InfraError::UnexpectedOutput(
            project.work_path.clone(),
            "el directorio destino ya existe y no esta vacio".to_string(),
        ));
    }

    Ok(())
}

fn ssl_file_paths(domain: &str) -> (PathBuf, PathBuf) {
    (
        path_config().ssl_cert_file(domain),
        path_config().ssl_key_file(domain),
    )
}
