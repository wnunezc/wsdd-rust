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

// build.rs — script de compilacion
// Embebe el icono en el exe (PE resources), declara requireAdministrator via winres,
// y exige que los ZIP embebidos de runtime esten sincronizados con sus arboles fuente.

use std::fs;
use std::path::Path;
use std::time::SystemTime;

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");
    println!("cargo:rerun-if-env-changed=WSDD_SKIP_WINDOWS_MANIFEST");
    println!("cargo:rerun-if-changed=assets/WSDD-64.ico");
    println!("cargo:rerun-if-changed=wsdd.manifest");

    emit_rerun_if_changed_tree(Path::new("recursos/recursos/PS-Script"));
    emit_rerun_if_changed_tree(Path::new("recursos/recursos/Docker-Structure"));
    println!("cargo:rerun-if-changed=recursos/ps-script.zip");
    println!("cargo:rerun-if-changed=recursos/docker-structure.zip");

    if let Err(err) = validate_embedded_zip_sync(
        Path::new("recursos/recursos/PS-Script"),
        Path::new("recursos/ps-script.zip"),
        "PS-Script",
    ) {
        eprintln!("{err}");
        std::process::exit(1);
    }

    if let Err(err) = validate_embedded_zip_sync(
        Path::new("recursos/recursos/Docker-Structure"),
        Path::new("recursos/docker-structure.zip"),
        "Docker-Structure",
    ) {
        eprintln!("{err}");
        std::process::exit(1);
    }

    // Embeber icono + manifest requireAdministrador en el exe (solo Windows, linker MSVC)
    #[cfg(target_os = "windows")]
    {
        let skip_manifest = std::env::var_os("WSDD_SKIP_WINDOWS_MANIFEST").is_some();
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/WSDD-64.ico");
        if !skip_manifest {
            res.set_manifest_file("wsdd.manifest");
        }
        if let Ok(version) = std::env::var("CARGO_PKG_VERSION") {
            res.set("FileVersion", &version);
            res.set("ProductVersion", &version);
        }
        if let Err(e) = res.compile() {
            eprintln!("winres error: {e}");
            std::process::exit(1);
        }
    }
}

fn emit_rerun_if_changed_tree(root: &Path) {
    println!("cargo:rerun-if-changed={}", root.display());
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            println!("cargo:rerun-if-changed={}", path.display());
            if path.is_dir() {
                emit_rerun_if_changed_tree(&path);
            }
        }
    }
}

fn validate_embedded_zip_sync(source_dir: &Path, zip_path: &Path, label: &str) -> Result<(), String> {
    let latest_source = latest_tree_modified(source_dir).ok_or_else(|| {
        format!(
            "build.rs: no se encontraron timestamps auditables en {} ({})",
            label,
            source_dir.display()
        )
    })?;

    let zip_modified = fs::metadata(zip_path)
        .map_err(|err| format!("build.rs: no se pudo leer {}: {err}", zip_path.display()))?
        .modified()
        .map_err(|err| format!("build.rs: no se pudo leer fecha de {}: {err}", zip_path.display()))?;

    if latest_source > zip_modified {
        return Err(format!(
            "build.rs: {} esta desincronizado con {}. Regenera el ZIP embebido antes de compilar.",
            label,
            zip_path.display()
        ));
    }

    Ok(())
}

fn latest_tree_modified(path: &Path) -> Option<SystemTime> {
    let mut latest = fs::metadata(path).ok()?.modified().ok();
    let entries = fs::read_dir(path).ok()?;

    for entry in entries.flatten() {
        let child = entry.path();
        let child_time = if child.is_dir() {
            latest_tree_modified(&child)
        } else {
            fs::metadata(&child).ok().and_then(|meta| meta.modified().ok())
        };

        if let Some(child_time) = child_time {
            latest = Some(match latest {
                Some(current) if current >= child_time => current,
                _ => child_time,
            });
        }
    }

    latest
}
