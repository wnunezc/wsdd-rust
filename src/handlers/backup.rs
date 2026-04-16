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
//! Backup y restore del entorno WSDD.
//!
//! Bloque G:
//! - Backup/restore completo del entorno Docker + `C:\WSDD-Environment\`
//! - Backup/restore de proyecto individual
//!
//! El diseño evita depender de imágenes auxiliares para extraer volúmenes:
//! en WSDD los volúmenes relevantes (`pma-code` y proyectos) usan bind mounts
//! hacia rutas Windows controladas por la app, por lo que el backup captura
//! directamente sus directorios origen.

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tempfile::tempdir;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::errors::InfraError;
use crate::handlers::docker::{self, WSDD_NETWORK, WSDD_PROJECT};
use crate::handlers::docker_deploy;
use crate::handlers::hosts;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::project as project_handler;
use crate::handlers::ps_script::{PsRunner, ScriptRunner};
use crate::handlers::yml;
use crate::models::project::Project;

const ENV_DIR: &str = r"C:\WSDD-Environment";
const SSL_DIR: &str = r"C:\WSDD-Environment\Docker-Structure\ssl";
const DOCKER_DIR: &str = r"C:\WSDD-Environment\Docker-Structure";
const FULL_ENV_DIR: &str = "environment";
const FULL_PROJECTS_DIR: &str = "projects";
const DOCKER_STAGE_DIR: &str = "docker";
const FULL_IMAGES_TAR: &str = "docker/wsdd-images.tar";
const FULL_NETWORK_JSON: &str = "docker/wsdd-network.json";
const PROJECT_STAGE_DIR: &str = "project";
const PROJECT_JSON_FILE: &str = "project/project.json";
const PROJECT_WORKDIR_DIR: &str = "project/workdir";
const PROJECT_VHOST_FILE: &str = "project/vhost.conf";
const PROJECT_OPTIONS_FILE: &str = "project/options-snapshot.txt";
const PROJECT_CERTS_DIR: &str = "project/certs";
const MANIFEST_FILE: &str = "manifest.json";
const PERSONAL_PROJECTS_MARKER: &str = "### PERSONAL PROJECTS ###";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum BackupKind {
    FullEnvironment,
    Project,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupManifest {
    kind: BackupKind,
    created_on: String,
    version: String,
    project: Option<Project>,
    projects: Vec<ProjectRef>,
    docker_images: Vec<String>,
    docker_networks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectRef {
    name: String,
    domain: String,
    work_path: String,
    php_version: String,
    ssl: bool,
}

pub fn default_full_backup_name() -> String {
    format!("wsdd-backup-full-{}.zip", today_string())
}

pub fn default_project_backup_name(project_name: &str) -> String {
    format!("wsdd-project-{}-{}.zip", project_name, today_string())
}

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

    copy_dir_recursive(Path::new(ENV_DIR), &root.join(FULL_ENV_DIR))?;

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
        created_on: today_string(),
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
    copy_dir_recursive(&env_stage, Path::new(ENV_DIR))?;

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
        runner.run_direct_sync("docker", &["load", "-i", &tar_arg], None, None)?;
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
        created_on: today_string(),
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

fn stage_project_source(root: &Path, project: &Project, tx: &LogSender) -> Result<(), InfraError> {
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

fn stage_project_certs(root: &Path, project: &Project) -> Result<(), InfraError> {
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

fn restore_project_certs(root: &Path, project: &Project) -> Result<(), InfraError> {
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

fn collect_wsdd_images(runner: &PsRunner) -> Result<Vec<String>, InfraError> {
    let out = runner.run_direct_sync(
        "docker",
        &[
            "ps",
            "-a",
            "--filter",
            "name=WSDD-",
            "--format",
            "{{.Image}}",
        ],
        None,
        None,
    )?;

    let images: BTreeSet<String> = out
        .text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("<none>"))
        .map(|line| line.to_string())
        .collect();

    Ok(images.into_iter().collect())
}

fn export_images(
    runner: &PsRunner,
    images: &[String],
    dest_tar: &Path,
    tx: &LogSender,
) -> Result<(), InfraError> {
    if let Some(parent) = dest_tar.parent() {
        fs::create_dir_all(parent).map_err(InfraError::Io)?;
    }

    let _ = tx.send(LogLine::info("[Backup] Exportando imagenes Docker WSDD..."));

    let mut args = vec![
        "save".to_string(),
        "-o".to_string(),
        dest_tar.to_string_lossy().to_string(),
    ];
    args.extend(images.iter().cloned());
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    runner.run_direct_sync("docker", &refs, None, None)?;

    let _ = tx.send(LogLine::success("[Backup] Imagenes Docker exportadas ✓"));
    Ok(())
}

fn export_network_snapshot(
    runner: &PsRunner,
    dest_json: &Path,
    tx: &LogSender,
) -> Result<(), InfraError> {
    if let Some(parent) = dest_json.parent() {
        fs::create_dir_all(parent).map_err(InfraError::Io)?;
    }

    let out = runner.run_direct_sync("docker", &["network", "inspect", WSDD_NETWORK], None, None);
    match out {
        Ok(snapshot) if snapshot.success => {
            fs::write(dest_json, snapshot.text).map_err(InfraError::Io)?;
            let _ = tx.send(LogLine::success("[Backup] Snapshot de red exportado ✓"));
        }
        Ok(_) | Err(_) => {
            let _ = tx.send(LogLine::warn(
                "[Backup] No se pudo exportar snapshot de red wsdd-network",
            ));
        }
    }
    Ok(())
}

fn stop_and_remove_wsdd_containers(runner: &PsRunner, tx: &LogSender) -> Result<(), InfraError> {
    let out = runner.run_direct_sync(
        "docker",
        &[
            "ps",
            "-a",
            "--filter",
            "name=WSDD-",
            "--format",
            "{{.Names}}",
        ],
        None,
        None,
    )?;

    let names: Vec<String> = out
        .text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect();

    if names.is_empty() {
        return Ok(());
    }

    let _ = tx.send(LogLine::info(
        "[Restore] Deteniendo y eliminando contenedores WSDD actuales...",
    ));

    for name in &names {
        let _ = runner.run_direct_sync("docker", &["stop", name], None, None);
        let _ = runner.run_direct_sync("docker", &["rm", name], None, None);
    }

    let _ = tx.send(LogLine::success(
        "[Restore] Contenedores WSDD previos detenidos ✓",
    ));
    Ok(())
}

fn rehydrate_project_runtime(
    project: &Project,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let _ = tx.send(LogLine::info(format!(
        "[Restore] Rehidratando proyecto '{}'...",
        project.name
    )));

    create_project_bind_volume(project, runner)?;
    let options_file = yml::options_path(
        project.php_version.dir_name(),
        project.php_version.compose_tag(),
    );
    yml::add_project_to_options_yml(
        &options_file,
        &project.domain,
        project.php_version.compose_tag(),
    )?;
    sync_active_vhost_for_php(&project.php_version, tx)?;
    rebuild_php_container(project, runner, tx)?;

    if project.ssl {
        restart_proxy(runner, tx)?;
    }

    let base_domains = project.php_version.base_container_domains();
    let mut domains: Vec<&str> = base_domains.iter().map(String::as_str).collect();
    domains.push(project.domain.as_str());

    hosts::update_host(Some(&domains), tx)
        .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;

    let _ = tx.send(LogLine::success(format!(
        "[Restore] Proyecto '{}' rehidratado ✓",
        project.name
    )));
    Ok(())
}

fn create_project_bind_volume(project: &Project, runner: &PsRunner) -> Result<(), InfraError> {
    let volume_name = format!("{}-{}", project.php_version.compose_tag(), project.domain);
    let device_opt = format!("device={}", project.work_path);

    let _ = runner.run_direct_sync("docker", &["volume", "rm", &volume_name], None, None);
    runner.run_direct_sync(
        "docker",
        &[
            "volume",
            "create",
            "--driver",
            "local",
            "--opt",
            "type=none",
            "--opt",
            &device_opt,
            "--opt",
            "o=bind",
            &volume_name,
        ],
        None,
        None,
    )?;
    Ok(())
}

fn rebuild_php_container(
    project: &Project,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let container_name = format!("WSDD-Web-Server-{}", project.php_version.container_tag());
    let php_dir = project.php_version.dir_name();
    let compose_tag = project.php_version.compose_tag();
    let should_build =
        docker::php_container_exists_sync(runner, project.php_version.container_tag())
            .map(|exists| !exists)
            .unwrap_or(true);
    let options_yml = yml::options_path(php_dir, compose_tag);
    let webserver_yml =
        format!(r"C:\WSDD-Environment\Docker-Structure\bin\{php_dir}\webserver.{compose_tag}.yml");

    let _ = runner.run_direct_sync("docker", &["stop", &container_name], None, None);
    let _ = runner.run_direct_sync("docker", &["rm", &container_name], None, None);

    let message = if should_build {
        format!(
            "[Restore] Reconstruyendo y creando contenedor {}...",
            container_name
        )
    } else {
        format!("[Restore] Recreando contenedor {}...", container_name)
    };
    let _ = tx.send(LogLine::info(message));

    let bridge = docker_deploy::make_log_bridge(tx);
    let docker_dir = Path::new(DOCKER_DIR).join("bin").join(php_dir);
    runner.run_ps_sync(
        &format!(
            "docker-compose -p {WSDD_PROJECT} -f \"{webserver_yml}\" -f \"{options_yml}\" up -d {}--force-recreate",
            if should_build { "--build " } else { "" }
        ),
        Some(&docker_dir),
        Some(&bridge),
    )?;

    Ok(())
}

fn restart_proxy(runner: &PsRunner, tx: &LogSender) -> Result<(), InfraError> {
    let _ = tx.send(LogLine::info("[Restore] Reiniciando WSDD-Proxy-Server..."));
    runner.run_direct_sync("docker", &["restart", "WSDD-Proxy-Server"], None, None)?;
    Ok(())
}

fn capture_vhost_block(project: &Project) -> Result<Option<String>, InfraError> {
    let vhost_path = vhost_conf_path(project);
    if !vhost_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(vhost_path).map_err(InfraError::Io)?;
    Ok(extract_vhost_block(&content, &project.domain))
}

fn capture_options_snapshot(project: &Project) -> Result<Option<String>, InfraError> {
    let options_path = yml::options_path(
        project.php_version.dir_name(),
        project.php_version.compose_tag(),
    );
    let path = Path::new(&options_path);
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(path).map_err(InfraError::Io)?;
    let snapshot = content
        .lines()
        .filter(|line| {
            line.contains(&project.domain)
                || line.contains(&format!(
                    "{}-{}",
                    project.php_version.compose_tag(),
                    project.domain
                ))
        })
        .collect::<Vec<_>>()
        .join("\n");

    if snapshot.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(snapshot))
    }
}

fn render_vhost_block(project: &Project) -> Result<String, InfraError> {
    let template = fs::read_to_string(vhost_template_path(project)).map_err(InfraError::Io)?;
    let protocol = if project.ssl {
        "Protocols h2 h2c http/1.1"
    } else {
        ""
    };
    Ok(template
        .replace("{CustomUrl}", &project.domain)
        .replace("{EntryPoint}", project.entry_point.as_path())
        .replace("{PROTOCOL}", protocol))
}

fn extract_vhost_block(content: &str, domain: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let marker = format!("ServerName {}", domain);
    let mut start_idx: Option<usize> = None;

    for (idx, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("<VirtualHost") {
            start_idx = Some(idx);
        }
        if line.contains(&marker) {
            if let Some(start) = start_idx {
                for end in idx..lines.len() {
                    if lines[end].trim_start().starts_with("</VirtualHost>") {
                        return Some(lines[start..=end].join("\n"));
                    }
                }
            }
        }
    }

    None
}

fn sync_active_vhost_for_php(
    php_version: &crate::models::project::PhpVersion,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let mut projects: Vec<Project> = project_handler::list_all()?
        .into_iter()
        .filter(|project| project.php_version == *php_version)
        .collect();
    projects.sort_by(|a, b| a.name.cmp(&b.name));

    let path = vhost_conf_path_for_php(php_version);
    let existing = fs::read_to_string(&path).map_err(InfraError::Io)?;
    let blocks = projects
        .iter()
        .map(render_vhost_block)
        .collect::<Result<Vec<_>, _>>()?;
    let rewritten = rewrite_personal_projects_section(&existing, &blocks);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(InfraError::Io)?;
    }
    fs::write(&path, rewritten).map_err(InfraError::Io)?;
    cleanup_legacy_vhost_file(php_version, tx);
    Ok(())
}

fn rewrite_personal_projects_section(content: &str, blocks: &[String]) -> String {
    let marker_positions: Vec<usize> = content
        .match_indices(PERSONAL_PROJECTS_MARKER)
        .map(|(idx, _)| idx)
        .collect();

    let rendered_blocks = if blocks.is_empty() {
        String::new()
    } else {
        format!("\n\n{}\n", blocks.join("\n\n"))
    };

    if marker_positions.len() >= 2 {
        let first_marker_end = marker_positions[0] + PERSONAL_PROJECTS_MARKER.len();
        let second_marker_start = marker_positions[1];
        format!(
            "{}{}{}",
            &content[..first_marker_end],
            rendered_blocks,
            &content[second_marker_start..]
        )
    } else if let Some(first_marker_start) = marker_positions.first().copied() {
        let first_marker_end = first_marker_start + PERSONAL_PROJECTS_MARKER.len();
        format!(
            "{}{}{}",
            &content[..first_marker_end],
            rendered_blocks,
            &content[first_marker_end..]
        )
    } else if blocks.is_empty() {
        content.to_string()
    } else {
        format!("{content}\n\n{}\n", blocks.join("\n\n"))
    }
}

fn cleanup_legacy_vhost_file(php_version: &crate::models::project::PhpVersion, tx: &LogSender) {
    let legacy_path = legacy_vhost_conf_path(php_version);
    if !legacy_path.exists() {
        return;
    }

    match fs::remove_file(&legacy_path) {
        Ok(()) => {
            let _ = tx.send(LogLine::info(
                "[Restore] Archivo legacy vhost.conf removido del flujo activo",
            ));
        }
        Err(e) => {
            let _ = tx.send(LogLine::warn(format!(
                "[Restore] Advertencia al remover vhost.conf legacy: {e}"
            )));
        }
    }
}

fn validate_restore_work_path(project: &Project) -> Result<(), InfraError> {
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

fn vhost_conf_path(project: &Project) -> PathBuf {
    vhost_conf_path_for_php(&project.php_version)
}

fn vhost_conf_path_for_php(php_version: &crate::models::project::PhpVersion) -> PathBuf {
    Path::new(DOCKER_DIR)
        .join("bin")
        .join(php_version.dir_name())
        .join("vhost")
        .join("vhost.conf")
}

fn legacy_vhost_conf_path(php_version: &crate::models::project::PhpVersion) -> PathBuf {
    Path::new(DOCKER_DIR)
        .join("bin")
        .join(php_version.dir_name())
        .join("vhost.conf")
}

fn vhost_template_path(project: &Project) -> PathBuf {
    Path::new(DOCKER_DIR)
        .join("bin")
        .join(project.php_version.dir_name())
        .join("tpl.vhost.conf")
}

fn ssl_file_paths(domain: &str) -> (PathBuf, PathBuf) {
    (
        Path::new(SSL_DIR).join(format!("{domain}.crt")),
        Path::new(SSL_DIR).join(format!("{domain}.key")),
    )
}

fn write_manifest(root: &Path, manifest: &BackupManifest) -> Result<(), InfraError> {
    let bytes = serde_json::to_vec_pretty(manifest)
        .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;
    fs::write(root.join(MANIFEST_FILE), bytes).map_err(InfraError::Io)
}

fn read_manifest(root: &Path) -> Result<BackupManifest, InfraError> {
    let content = fs::read(root.join(MANIFEST_FILE)).map_err(InfraError::Io)?;
    serde_json::from_slice(&content)
        .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), InfraError> {
    if !src.exists() {
        return Ok(());
    }

    for entry in WalkDir::new(src) {
        let entry = entry.map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;
        let path = entry.path();
        let rel = path
            .strip_prefix(src)
            .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;
        let target = dest.join(rel);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).map_err(InfraError::Io)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(InfraError::Io)?;
            }
            fs::copy(path, &target).map_err(InfraError::Io)?;
        }
    }

    Ok(())
}

fn create_zip_from_dir(src_dir: &Path, destination_zip: &Path) -> Result<(), InfraError> {
    if let Some(parent) = destination_zip.parent() {
        fs::create_dir_all(parent).map_err(InfraError::Io)?;
    }

    let file = File::create(destination_zip).map_err(InfraError::Io)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    for entry in WalkDir::new(src_dir) {
        let entry = entry.map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;
        let path = entry.path();
        let rel = path
            .strip_prefix(src_dir)
            .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;

        if rel.as_os_str().is_empty() {
            continue;
        }

        let name = to_zip_name(rel);
        if entry.file_type().is_dir() {
            zip.add_directory(name, options)
                .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;
            continue;
        }

        zip.start_file(name, options)
            .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;
        let mut src = File::open(path).map_err(InfraError::Io)?;
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let read = src.read(&mut buffer).map_err(InfraError::Io)?;
            if read == 0 {
                break;
            }
            zip.write_all(&buffer[..read]).map_err(InfraError::Io)?;
        }
    }

    zip.finish()
        .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;
    Ok(())
}

fn extract_zip_to_dir(zip_path: &Path, dest: &Path) -> Result<(), InfraError> {
    let file = File::open(zip_path).map_err(InfraError::Io)?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;

    for i in 0..archive.len() {
        let mut item = archive
            .by_index(i)
            .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;
        let enclosed = item.enclosed_name().ok_or_else(|| {
            InfraError::UnexpectedOutput(
                zip_path.display().to_string(),
                format!("entrada ZIP insegura: {}", item.name()),
            )
        })?;
        let out_path = dest.join(enclosed);

        if item.name().ends_with('/') {
            fs::create_dir_all(&out_path).map_err(InfraError::Io)?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(InfraError::Io)?;
        }
        let mut out_file = File::create(&out_path).map_err(InfraError::Io)?;
        std::io::copy(&mut item, &mut out_file).map_err(InfraError::Io)?;
    }

    Ok(())
}

fn to_zip_name(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().replace('\\', "/"))
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(windows)]
fn today_string() -> String {
    use windows::Win32::System::SystemInformation::GetLocalTime;

    unsafe {
        let now = GetLocalTime();
        format!("{:04}-{:02}-{:02}", now.wYear, now.wMonth, now.wDay)
    }
}

#[cfg(not(windows))]
fn today_string() -> String {
    "1970-01-01".to_string()
}

impl From<&Project> for ProjectRef {
    fn from(project: &Project) -> Self {
        Self {
            name: project.name.clone(),
            domain: project.domain.clone(),
            work_path: project.work_path.clone(),
            php_version: project.php_version.display_name().to_string(),
            ssl: project.ssl,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::project::{EntryPoint, PhpVersion, ProjectStatus};

    fn test_project() -> Project {
        Project {
            name: "demo".to_string(),
            domain: "demo.dock".to_string(),
            php_version: PhpVersion::Php83,
            work_path: r"C:\projects\demo".to_string(),
            entry_point: EntryPoint::Root,
            ssl: true,
            status: ProjectStatus::default(),
        }
    }

    #[test]
    fn extracts_vhost_block_for_domain() {
        let content = "\
### PERSONAL PROJECTS ###
<VirtualHost *:80>
    ServerName demo.dock
</VirtualHost>

<VirtualHost *:80>
    ServerName other.dock
</VirtualHost>
";

        let block = extract_vhost_block(content, "demo.dock").expect("block");
        assert!(block.contains("demo.dock"));
        assert!(!block.contains("other.dock"));
    }

    #[test]
    fn rewrite_personal_projects_section_preserves_marker() {
        let content = "### PERSONAL PROJECTS ###\n### PERSONAL PROJECTS ###\n";
        let result = rewrite_personal_projects_section(
            content,
            &[String::from(
                "<VirtualHost *:80>\nServerName demo.dock\n</VirtualHost>",
            )],
        );
        assert!(result.contains("### PERSONAL PROJECTS ###"));
        assert!(result.contains("demo.dock"));
    }

    #[test]
    fn zip_name_uses_forward_slashes() {
        let path = Path::new(r"docker\wsdd-images.tar");
        assert_eq!(to_zip_name(path), "docker/wsdd-images.tar");
    }

    #[test]
    fn project_ref_maps_expected_fields() {
        let project = test_project();
        let snapshot = ProjectRef::from(&project);
        assert_eq!(snapshot.name, "demo");
        assert_eq!(snapshot.domain, "demo.dock");
        assert_eq!(snapshot.php_version, "PHP 8.3");
        assert!(snapshot.ssl);
    }
}
