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
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");
    println!("cargo:rerun-if-env-changed=WSDD_FILE_VERSION");
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
        if let Ok(version) =
            std::env::var("WSDD_FILE_VERSION").or_else(|_| std::env::var("CARGO_PKG_VERSION"))
        {
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

fn validate_embedded_zip_sync(
    source_dir: &Path,
    zip_path: &Path,
    label: &str,
) -> Result<(), String> {
    fs::metadata(zip_path)
        .map_err(|err| format!("build.rs: no se pudo leer {}: {err}", zip_path.display()))?;

    let source_prefix = source_dir.to_string_lossy().replace('\\', "/");
    let zip_name = zip_path.to_string_lossy().replace('\\', "/");
    let output = match Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .arg("--")
        .arg(source_dir)
        .arg(zip_path)
        .output()
    {
        Ok(output) if output.status.success() => output,
        Ok(output) => {
            println!(
                "cargo:warning=build.rs no pudo validar sincronizacion Git para {}: {}",
                label,
                String::from_utf8_lossy(&output.stderr).trim()
            );
            return Ok(());
        }
        Err(err) => {
            println!(
                "cargo:warning=build.rs omitio validacion Git para {}: {}",
                label, err
            );
            return Ok(());
        }
    };

    let mut source_dirty = false;
    let mut zip_dirty = false;

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let Some(path) = line.get(3..) else {
            continue;
        };
        let normalized = path.trim().replace('\\', "/");
        if normalized == zip_name {
            zip_dirty = true;
        } else if normalized == source_prefix
            || normalized.starts_with(&(source_prefix.clone() + "/"))
        {
            source_dirty = true;
        }
    }

    if source_dirty && !zip_dirty {
        return Err(format!(
            "build.rs: {} tiene cambios locales sin reflejar en {}. Regenera el ZIP funcional y commita ambos cambios juntos.",
            label,
            zip_path.display()
        ));
    }

    Ok(())
}
