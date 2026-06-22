use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use walkdir::WalkDir;
use zip::ZipArchive;

use crate::errors::InfraError;
use crate::handlers::backup::archive::create_zip_from_dir;
use crate::handlers::ps_script::ProcOutput;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct InventoryEntry {
    size: u64,
    sha256: [u8; 32],
}

pub(super) type FileInventory = BTreeMap<String, InventoryEntry>;

#[derive(Debug, Default)]
pub(super) struct ManagedStorageTransition {
    alias_path: Option<PathBuf>,
    preserved_path: Option<PathBuf>,
}

impl ManagedStorageTransition {
    pub(super) fn record_original_preserved(&mut self, alias: &Path, preserved: &Path) {
        self.alias_path = Some(alias.to_path_buf());
        self.preserved_path = Some(preserved.to_path_buf());
    }

    pub(super) fn rollback(self) -> Result<(), InfraError> {
        let (Some(alias), Some(preserved)) = (self.alias_path, self.preserved_path) else {
            return Ok(());
        };

        if let Ok(metadata) = fs::symlink_metadata(&alias) {
            if !is_reparse_point(&metadata) {
                return Err(InfraError::PrerequisiteNotMet(format!(
                    "Rollback detenido: la ruta del alias ya no es un enlace: {}",
                    alias.display()
                )));
            }
            fs::remove_dir(&alias).map_err(InfraError::Io)?;
        }

        if !preserved.is_dir() {
            return Err(InfraError::PrerequisiteNotMet(format!(
                "Rollback detenido: no existe la carpeta original preservada: {}",
                preserved.display()
            )));
        }
        fs::rename(&preserved, &alias).map_err(InfraError::Io)
    }
}

#[cfg(windows)]
pub(super) fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    metadata.file_attributes() & 0x400 != 0
}

#[cfg(not(windows))]
pub(super) fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

pub(super) fn ensure_process_success(
    step: &str,
    output: ProcOutput,
) -> Result<ProcOutput, InfraError> {
    if output.success {
        Ok(output)
    } else {
        Err(InfraError::UnexpectedOutput(step.to_string(), output.text))
    }
}

pub(super) fn build_inventory(root: &Path) -> Result<FileInventory, InfraError> {
    if !root.is_dir() {
        return Err(InfraError::PrerequisiteNotMet(format!(
            "No existe el directorio para inventario: {}",
            root.display()
        )));
    }

    let root_metadata = fs::symlink_metadata(root).map_err(InfraError::Io)?;
    let walk_root = if is_reparse_point(&root_metadata) {
        fs::canonicalize(root).map_err(InfraError::Io)?
    } else {
        root.to_path_buf()
    };
    let mut inventory = BTreeMap::new();
    for entry in WalkDir::new(&walk_root).follow_links(false) {
        let entry = entry.map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;
        if entry.file_type().is_dir() {
            continue;
        }
        if entry.file_type().is_symlink() && is_internal_link(&walk_root, entry.path()) {
            return Err(InfraError::PrerequisiteNotMet(format!(
                "No se migran enlaces internos sin validacion explicita: {}",
                entry.path().display()
            )));
        }

        let relative = entry
            .path()
            .strip_prefix(&walk_root)
            .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?
            .components()
            .map(|part| part.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        let metadata = entry.metadata().map_err(|e| {
            InfraError::Io(std::io::Error::other(format!(
                "{}: {e}",
                entry.path().display()
            )))
        })?;
        let mut file = File::open(entry.path()).map_err(|error| {
            InfraError::Io(std::io::Error::other(format!(
                "No se pudo abrir para inventario '{}': {error}",
                entry.path().display()
            )))
        })?;
        let mut hasher = Sha256::new();
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let read = file.read(&mut buffer).map_err(|error| {
                InfraError::Io(std::io::Error::other(format!(
                    "No se pudo leer para inventario '{}': {error}",
                    entry.path().display()
                )))
            })?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        inventory.insert(
            relative,
            InventoryEntry {
                size: metadata.len(),
                sha256: hasher.finalize().into(),
            },
        );
    }
    Ok(inventory)
}

fn is_internal_link(root: &Path, entry: &Path) -> bool {
    entry != root
}

pub(super) fn create_validated_zip(
    source: &Path,
    destination_zip: &Path,
    expected_inventory: &FileInventory,
) -> Result<FileInventory, InfraError> {
    create_zip_from_dir(source, destination_zip)?;
    let archived_inventory = build_zip_inventory(destination_zip)?;
    if archived_inventory != *expected_inventory {
        return Err(InfraError::UnexpectedOutput(
            destination_zip.display().to_string(),
            format!(
                "El ZIP no coincide con el inventario inicial:\n{}",
                inventory_diff(expected_inventory, &archived_inventory, 50)
            ),
        ));
    }
    Ok(archived_inventory)
}

fn build_zip_inventory(zip_path: &Path) -> Result<FileInventory, InfraError> {
    let file = File::open(zip_path).map_err(InfraError::Io)?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;
    let mut inventory = BTreeMap::new();
    for index in 0..archive.len() {
        let mut item = archive
            .by_index(index)
            .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;
        if item.is_dir() {
            continue;
        }
        let name = item
            .enclosed_name()
            .ok_or_else(|| {
                InfraError::UnexpectedOutput(
                    zip_path.display().to_string(),
                    format!("Entrada ZIP insegura: {}", item.name()),
                )
            })?
            .components()
            .map(|part| part.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        let size = item.size();
        let mut hasher = Sha256::new();
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let read = item.read(&mut buffer).map_err(|error| {
                InfraError::Io(std::io::Error::other(format!(
                    "No se pudo leer '{name}' desde '{}': {error}",
                    zip_path.display()
                )))
            })?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        inventory.insert(
            name,
            InventoryEntry {
                size,
                sha256: hasher.finalize().into(),
            },
        );
    }
    Ok(inventory)
}

pub(super) fn extract_zip_snapshot(zip_path: &Path, destination: &Path) -> Result<(), InfraError> {
    if destination.exists() && destination.read_dir()?.next().is_some() {
        return Err(InfraError::PrerequisiteNotMet(format!(
            "El destino temporal no esta vacio: {}",
            destination.display()
        )));
    }
    fs::create_dir_all(destination).map_err(InfraError::Io)?;
    let file = File::open(zip_path).map_err(InfraError::Io)?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;
    for index in 0..archive.len() {
        let mut item = archive
            .by_index(index)
            .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;
        let enclosed = item.enclosed_name().ok_or_else(|| {
            InfraError::UnexpectedOutput(
                zip_path.display().to_string(),
                format!("Entrada ZIP insegura: {}", item.name()),
            )
        })?;
        let output_path = destination.join(enclosed);
        if item.is_dir() {
            fs::create_dir_all(&output_path).map_err(|error| {
                InfraError::Io(std::io::Error::other(format!(
                    "No se pudo crear directorio '{}': {error}",
                    output_path.display()
                )))
            })?;
            continue;
        }
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                InfraError::Io(std::io::Error::other(format!(
                    "No se pudo crear directorio '{}': {error}",
                    parent.display()
                )))
            })?;
        }
        let mut output = File::create(&output_path).map_err(|error| {
            InfraError::Io(std::io::Error::other(format!(
                "No se pudo crear archivo destino '{}': {error}",
                output_path.display()
            )))
        })?;
        std::io::copy(&mut item, &mut output).map_err(|error| {
            InfraError::Io(std::io::Error::other(format!(
                "No se pudo extraer '{}' hacia '{}': {error}",
                item.name(),
                output_path.display()
            )))
        })?;
    }
    Ok(())
}

pub(super) fn inventory_diff(
    expected: &FileInventory,
    actual: &FileInventory,
    limit: usize,
) -> String {
    let mut differences = Vec::new();
    for path in expected.keys() {
        match actual.get(path) {
            None => differences.push(format!("FALTA {path}")),
            Some(value) if value != &expected[path] => {
                differences.push(format!("TAMANO/SHA256 {path}"))
            }
            Some(_) => {}
        }
    }
    for path in actual.keys() {
        if !expected.contains_key(path) {
            differences.push(format!("ADICIONAL {path}"));
        }
    }
    let total = differences.len();
    differences.truncate(limit);
    if total > limit {
        differences.push(format!(
            "... {} diferencias adicionales omitidas",
            total - limit
        ));
    }
    differences.join("\n")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{
        build_inventory, create_validated_zip, ensure_process_success, extract_zip_snapshot,
        inventory_diff, is_internal_link,
    };
    use crate::handlers::ps_script::ProcOutput;

    #[test]
    fn failed_process_output_is_rejected() {
        let output = ProcOutput {
            text: "operation failed".to_string(),
            success: false,
        };

        let error = ensure_process_success("storage step", output)
            .expect_err("failed process must stop the transaction");

        assert!(error.to_string().contains("storage step"));
        assert!(error.to_string().contains("operation failed"));
    }

    #[test]
    fn inventories_match_for_identical_trees() {
        let source = tempdir().expect("source tempdir");
        let target = tempdir().expect("target tempdir");
        fs::create_dir_all(source.path().join("public")).expect("source dir");
        fs::create_dir_all(target.path().join("public")).expect("target dir");
        fs::write(source.path().join("public/index.html"), b"same").expect("source file");
        fs::write(target.path().join("public/index.html"), b"same").expect("target file");

        assert_eq!(
            build_inventory(source.path()).expect("source inventory"),
            build_inventory(target.path()).expect("target inventory")
        );
    }

    #[test]
    fn inventories_detect_content_changes() {
        let source = tempdir().expect("source tempdir");
        let target = tempdir().expect("target tempdir");
        fs::write(source.path().join("index.html"), b"source").expect("source file");
        fs::write(target.path().join("index.html"), b"target").expect("target file");

        assert_ne!(
            build_inventory(source.path()).expect("source inventory"),
            build_inventory(target.path()).expect("target inventory")
        );
    }

    #[test]
    fn root_alias_is_allowed_but_nested_links_are_rejected() {
        let root = std::path::Path::new(r"D:\projects\demo");

        assert!(!is_internal_link(root, root));
        assert!(is_internal_link(root, &root.join("vendor-link")));
    }

    #[test]
    fn validated_zip_is_the_snapshot_extracted_to_destination() {
        let source = tempdir().expect("source tempdir");
        let destination = tempdir().expect("destination tempdir");
        let zip_dir = tempdir().expect("zip tempdir");
        fs::create_dir_all(source.path().join("public")).expect("source dir");
        fs::write(
            source.path().join("public/index.php"),
            b"<?php echo 'snapshot';",
        )
        .expect("source file");
        let source_inventory = build_inventory(source.path()).expect("source inventory");
        let zip_path = zip_dir.path().join("snapshot.zip");

        let zip_inventory =
            create_validated_zip(source.path(), &zip_path, &source_inventory).expect("valid zip");
        extract_zip_snapshot(&zip_path, destination.path()).expect("extract snapshot");

        assert_eq!(zip_inventory, source_inventory);
        assert_eq!(
            build_inventory(destination.path()).expect("destination inventory"),
            zip_inventory
        );
    }

    #[test]
    fn inventory_diff_reports_only_actionable_mismatches() {
        let expected_root = tempdir().expect("expected tempdir");
        let actual_root = tempdir().expect("actual tempdir");
        fs::write(expected_root.path().join("missing.txt"), b"missing").expect("expected missing");
        fs::write(expected_root.path().join("changed.txt"), b"before").expect("expected changed");
        fs::write(actual_root.path().join("changed.txt"), b"after").expect("actual changed");
        fs::write(actual_root.path().join("extra.txt"), b"extra").expect("actual extra");

        let expected = build_inventory(expected_root.path()).expect("expected inventory");
        let actual = build_inventory(actual_root.path()).expect("actual inventory");
        let report = inventory_diff(&expected, &actual, 20);

        assert!(report.contains("FALTA missing.txt"));
        assert!(report.contains("ADICIONAL extra.txt"));
        assert!(report.contains("TAMANO/SHA256 changed.txt"));
    }
}
