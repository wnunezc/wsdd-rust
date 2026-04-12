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
// Gestion de recursos embebidos.
// Equivalente al mecanismo de Resources.resx + descompresion de recursos.zip en C#.
//
// Al primer arranque extrae dos ZIPs independientes a C:\WSDD-Environment\:
//   ps-script.zip     → PS-Script\    (siempre, si no existe o esta vacio)
//   docker-structure.zip → Docker-Structure\ (solo en primer arranque)
//
// Separar los ZIPs evita que una re-extraccion de scripts sobreescriba
// la estructura Docker configurada por el usuario.

use anyhow::{Context, Result};
use std::io::Read;
use std::path::Path;

use crate::handlers::ps_script::WSDD_ENV;

static PS_SCRIPT_ZIP: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/ps-script.zip"
));
static DOCKER_STRUCTURE_ZIP: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/docker-structure.zip"
));

/// Inicializa el entorno WSDD extrayendo recursos si no existen.
pub fn init() -> Result<()> {
    let env_path = Path::new(WSDD_ENV);
    std::fs::create_dir_all(env_path)?;

    // PS-Script: reparar si falta, esta vacio o fue modificado localmente.
    if ps_scripts_need_repair(PS_SCRIPT_ZIP, env_path)? {
        extract_zip(PS_SCRIPT_ZIP, env_path).context("Error extrayendo ps-script.zip")?;
    }

    // Docker-Structure: extraer en primer arranque y autocorregir layouts dañados.
    let docker_dir = env_path.join("Docker-Structure");
    if docker_structure_needs_repair(&docker_dir)? {
        extract_zip(DOCKER_STRUCTURE_ZIP, env_path)
            .context("Error extrayendo docker-structure.zip")?;
    }

    Ok(())
}

fn is_dir_empty(dir: &Path) -> Result<bool> {
    Ok(std::fs::read_dir(dir)?.next().is_none())
}

fn docker_structure_needs_repair(docker_dir: &Path) -> Result<bool> {
    if !docker_dir.exists() {
        return Ok(true);
    }

    if is_dir_empty(docker_dir)? {
        return Ok(true);
    }

    let required_paths = [
        docker_dir.join("init.yml"),
        docker_dir.join("bin").join("mysql").join("Dockerfile"),
        docker_dir.join("bin").join("pma").join("php.ini"),
    ];

    Ok(required_paths.iter().any(|path| !path.exists()))
}

fn ps_scripts_need_repair(data: &[u8], env_path: &Path) -> Result<bool> {
    let scripts_dir = env_path.join("PS-Script");
    if !scripts_dir.exists() || is_dir_empty(&scripts_dir)? {
        return Ok(true);
    }

    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor).context("Error abriendo ps-script.zip")?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.name().ends_with('/') {
            continue;
        }

        let outpath = env_path.join(file.name());
        if !outpath.exists() {
            return Ok(true);
        }

        let mut expected = Vec::new();
        file.read_to_end(&mut expected)?;
        let current = std::fs::read(&outpath)?;
        if current != expected {
            return Ok(true);
        }
    }

    Ok(false)
}

fn extract_zip(data: &[u8], dest: &Path) -> Result<()> {
    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor).context("Error abriendo ZIP embebido")?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = dest.join(file.name());

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut out = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut out)?;
        }
    }
    Ok(())
}
