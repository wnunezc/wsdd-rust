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
//! Backup and restore orchestration for WSDD.
//!
//! This module keeps the public backup API stable while delegating archive,
//! manifest, project filesystem, Docker runtime, and vhost responsibilities
//! to focused submodules.

mod archive;
mod manifest;
mod project_files;
mod runtime;
mod vhost;

use std::fs;
use std::path::Path;

use tempfile::tempdir;

pub use manifest::{default_full_backup_name, default_project_backup_name};

use archive::{copy_dir_recursive, create_zip_from_dir, extract_zip_to_dir};
use manifest::{read_manifest, write_manifest, BackupKind, BackupManifest, ProjectRef};
use project_files::{
    restore_project_certs, stage_project_certs, stage_project_source, validate_restore_work_path,
};
use runtime::{
    collect_wsdd_images, export_images, export_network_snapshot, rehydrate_project_runtime,
    stop_and_remove_wsdd_containers,
};
use vhost::{capture_options_snapshot, capture_vhost_block};

use crate::config::environment::{env_config, path_config};
use crate::errors::InfraError;
use crate::handlers::docker::WSDD_NETWORK;
use crate::handlers::docker_deploy;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::project as project_handler;
use crate::handlers::ps_script::{PsRunner, ScriptRunner};
use crate::models::project::Project;

const FULL_ENV_DIR: &str = "environment";
const FULL_PROJECTS_DIR: &str = "projects";
const FULL_IMAGES_TAR: &str = "docker/wsdd-images.tar";
const FULL_NETWORK_JSON: &str = "docker/wsdd-network.json";
const PROJECT_STAGE_DIR: &str = "project";
const PROJECT_JSON_FILE: &str = "project/project.json";
const PROJECT_WORKDIR_DIR: &str = "project/workdir";
const PROJECT_VHOST_FILE: &str = "project/vhost.conf";
const PROJECT_OPTIONS_FILE: &str = "project/options-snapshot.txt";
const PROJECT_CERTS_DIR: &str = "project/certs";

/// Creates a full WSDD environment backup.
pub fn backup_environment(
    destination_zip: &Path,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let _ = tx.send(LogLine::info(format!(
        "[Backup] Creando backup completo: {}",
        destination_zip.display()
    )));

    let staging = tempdir().map_err(InfraError::Io)?;
    let root = staging.path();

    copy_dir_recursive(path_config().environment_root(), &root.join(FULL_ENV_DIR))?;

    let projects = project_handler::list_all()?;
    for project in &projects {
        stage_project_source(root, project, tx)?;
    }

    let images = collect_wsdd_images(runner)?;
    if images.is_empty() {
        let _ = tx.send(LogLine::warn(
            "[Backup] No se detectaron imagenes WSDD para exportar",
        ));
    } else {
        export_images(runner, &images, &root.join(FULL_IMAGES_TAR), tx)?;
    }

    export_network_snapshot(runner, &root.join(FULL_NETWORK_JSON), tx)?;

    let manifest = BackupManifest {
        kind: BackupKind::FullEnvironment,
        created_on: manifest::today_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        project: None,
        projects: projects.iter().map(ProjectRef::from).collect(),
        docker_images: images,
        docker_networks: vec![WSDD_NETWORK.to_string()],
    };
    write_manifest(root, &manifest)?;
    create_zip_from_dir(root, destination_zip)?;

    let _ = tx.send(LogLine::success(format!(
        "[Backup] Backup completo creado ✓ {}",
        destination_zip.display()
    )));
    Ok(())
}

/// Restores a full WSDD environment backup.
pub fn restore_environment(
    source_zip: &Path,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let _ = tx.send(LogLine::info(format!(
        "[Restore] Restaurando entorno desde {}",
        source_zip.display()
    )));

    let extracted = tempdir().map_err(InfraError::Io)?;
    extract_zip_to_dir(source_zip, extracted.path())?;
    let manifest = read_manifest(extracted.path())?;

    if manifest.kind != BackupKind::FullEnvironment {
        return Err(InfraError::UnexpectedOutput(
            source_zip.display().to_string(),
            "el backup no es de entorno completo".to_string(),
        ));
    }

    stop_and_remove_wsdd_containers(runner, tx)?;

    let env_stage = extracted.path().join(FULL_ENV_DIR);
    if !env_stage.exists() {
        return Err(InfraError::UnexpectedOutput(
            source_zip.display().to_string(),
            "faltan archivos del entorno en el backup".to_string(),
        ));
    }
    copy_dir_recursive(&env_stage, path_config().environment_root())?;

    for project in &manifest.projects {
        let src = extracted
            .path()
            .join(FULL_PROJECTS_DIR)
            .join(&project.name)
            .join("workdir");
        if src.exists() {
            copy_dir_recursive(&src, Path::new(&project.work_path))?;
        }
    }

    let images_tar = extracted.path().join(FULL_IMAGES_TAR);
    if images_tar.exists() {
        let _ = tx.send(LogLine::info("[Restore] Importando imagenes Docker..."));
        let tar_arg = images_tar.to_string_lossy().to_string();
        runner.run_direct_sync(
            env_config().docker_exe(),
            &["load", "-i", &tar_arg],
            None,
            None,
        )?;
        let _ = tx.send(LogLine::success("[Restore] Imagenes importadas ✓"));
    }

    docker_deploy::deploy_environment_sync(runner, tx)?;

    let restored_projects = project_handler::list_all()?;
    for project in &restored_projects {
        rehydrate_project_runtime(project, runner, tx)?;
    }

    let _ = tx.send(LogLine::success(
        "[Restore] Entorno restaurado correctamente ✓",
    ));
    Ok(())
}

/// Creates a ZIP backup for one configured project.
pub fn backup_project(
    project: &Project,
    destination_zip: &Path,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let _ = tx.send(LogLine::info(format!(
        "[Backup] Creando backup del proyecto '{}'...",
        project.name
    )));

    let staging = tempdir().map_err(InfraError::Io)?;
    let root = staging.path();

    fs::create_dir_all(root.join(PROJECT_STAGE_DIR)).map_err(InfraError::Io)?;
    fs::write(
        root.join(PROJECT_JSON_FILE),
        serde_json::to_vec_pretty(project)
            .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?,
    )
    .map_err(InfraError::Io)?;

    let workdir = Path::new(&project.work_path);
    if !workdir.exists() {
        return Err(InfraError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "No existe el directorio del proyecto: {}",
                workdir.display()
            ),
        )));
    }
    copy_dir_recursive(workdir, &root.join(PROJECT_WORKDIR_DIR))?;

    if let Some(snapshot) = capture_options_snapshot(project)? {
        fs::write(root.join(PROJECT_OPTIONS_FILE), snapshot).map_err(InfraError::Io)?;
    }

    if let Some(vhost_block) = capture_vhost_block(project)? {
        fs::write(root.join(PROJECT_VHOST_FILE), vhost_block).map_err(InfraError::Io)?;
    }

    stage_project_certs(root, project)?;

    let manifest = BackupManifest {
        kind: BackupKind::Project,
        created_on: manifest::today_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        project: Some(project.clone()),
        projects: vec![ProjectRef::from(project)],
        docker_images: Vec::new(),
        docker_networks: Vec::new(),
    };
    write_manifest(root, &manifest)?;
    create_zip_from_dir(root, destination_zip)?;

    let _ = tx.send(LogLine::success(format!(
        "[Backup] Backup del proyecto '{}' creado ✓ {}",
        project.name,
        destination_zip.display()
    )));
    Ok(())
}

/// Restores a project backup and rehydrates its Docker runtime wiring.
pub fn restore_project(
    source_zip: &Path,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<Project, InfraError> {
    let _ = tx.send(LogLine::info(format!(
        "[Restore] Restaurando proyecto desde {}",
        source_zip.display()
    )));

    let extracted = tempdir().map_err(InfraError::Io)?;
    extract_zip_to_dir(source_zip, extracted.path())?;
    let manifest = read_manifest(extracted.path())?;

    if manifest.kind != BackupKind::Project {
        return Err(InfraError::UnexpectedOutput(
            source_zip.display().to_string(),
            "el backup no es de proyecto".to_string(),
        ));
    }

    let project = manifest.project.ok_or_else(|| {
        InfraError::UnexpectedOutput(
            source_zip.display().to_string(),
            "el backup no contiene metadata de proyecto".to_string(),
        )
    })?;

    if project_handler::exists(&project.name) {
        return Err(InfraError::UnexpectedOutput(
            project.name.clone(),
            "ya existe un proyecto con ese nombre".to_string(),
        ));
    }
    if project_handler::list_all()?
        .iter()
        .any(|p| p.domain.eq_ignore_ascii_case(&project.domain))
    {
        return Err(InfraError::UnexpectedOutput(
            project.domain.clone(),
            "ya existe un proyecto con ese dominio".to_string(),
        ));
    }

    validate_restore_work_path(&project)?;

    let workdir_stage = extracted.path().join(PROJECT_WORKDIR_DIR);
    if workdir_stage.exists() {
        copy_dir_recursive(&workdir_stage, Path::new(&project.work_path))?;
    }

    project_handler::save(&project)?;

    let options_stage = extracted.path().join(PROJECT_OPTIONS_FILE);
    if options_stage.exists() {
        let _ = tx.send(LogLine::info(
            "[Restore] Snapshot de options.yml encontrado en el backup",
        ));
    }

    let vhost_stage = extracted.path().join(PROJECT_VHOST_FILE);
    if vhost_stage.exists() {
        let _ = tx.send(LogLine::info(
            "[Restore] Snapshot de vhost encontrado; se regenerara desde metadata del proyecto",
        ));
    }

    restore_project_certs(extracted.path(), &project)?;
    rehydrate_project_runtime(&project, runner, tx)?;

    let _ = tx.send(LogLine::success(format!(
        "[Restore] Proyecto '{}' restaurado ✓",
        project.name
    )));
    Ok(project)
}
