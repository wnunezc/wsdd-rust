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
//! Filesystem and ZIP helpers used by WSDD backup flows.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::errors::InfraError;

/// Copies a directory tree into `dest`, preserving relative paths.
pub(super) fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), InfraError> {
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

/// Creates a ZIP archive from an already staged directory tree.
pub(super) fn create_zip_from_dir(
    src_dir: &Path,
    destination_zip: &Path,
) -> Result<(), InfraError> {
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

/// Extracts a ZIP archive while rejecting path traversal entries.
pub(super) fn extract_zip_to_dir(zip_path: &Path, dest: &Path) -> Result<(), InfraError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zip_name_uses_forward_slashes() {
        let path = Path::new(r"docker\wsdd-images.tar");
        assert_eq!(to_zip_name(path), "docker/wsdd-images.tar");
    }
}
