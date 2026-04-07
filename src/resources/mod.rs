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
use std::path::Path;

use crate::handlers::ps_script::WSDD_ENV;

static PS_SCRIPT_ZIP: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/recursos/ps-script.zip"));
static DOCKER_STRUCTURE_ZIP: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/recursos/docker-structure.zip"));

/// Inicializa el entorno WSDD extrayendo recursos si no existen.
pub fn init() -> Result<()> {
    let env_path = Path::new(WSDD_ENV);
    std::fs::create_dir_all(env_path)?;

    // PS-Script: extraer si no existe o esta vacio (scripts pueden actualizarse con la app)
    let scripts_dir = env_path.join("PS-Script");
    if !scripts_dir.exists() || is_dir_empty(&scripts_dir)? {
        extract_zip(PS_SCRIPT_ZIP, env_path).context("Error extrayendo ps-script.zip")?;
    }

    // Docker-Structure: extraer solo en primer arranque para no sobreescribir config del usuario
    let docker_dir = env_path.join("Docker-Structure");
    if !docker_dir.exists() {
        extract_zip(DOCKER_STRUCTURE_ZIP, env_path)
            .context("Error extrayendo docker-structure.zip")?;
    }

    Ok(())
}

fn is_dir_empty(dir: &Path) -> Result<bool> {
    Ok(std::fs::read_dir(dir)?.next().is_none())
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
