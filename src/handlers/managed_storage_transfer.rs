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
        let mut file = File::open(entry.path()).map_err(InfraError::Io)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let read = file.read(&mut buffer).map_err(InfraError::Io)?;
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

pub(super) fn copy_tree(source: &Path, destination: &Path) -> Result<(), InfraError> {
    if destination.exists() && destination.read_dir()?.next().is_some() {
        return Err(InfraError::PrerequisiteNotMet(format!(
            "El destino temporal no esta vacio: {}",
            destination.display()
        )));
    }
    fs::create_dir_all(destination).map_err(InfraError::Io)?;

    for entry in WalkDir::new(source).follow_links(false) {
        let entry = entry.map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;
        let relative = entry
            .path()
            .strip_prefix(source)
            .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;
        let target = destination.join(relative);

        if entry.file_type().is_symlink() {
            return Err(InfraError::PrerequisiteNotMet(format!(
                "No se migran enlaces internos sin validacion explicita: {}",
                entry.path().display()
            )));
        }
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).map_err(InfraError::Io)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(InfraError::Io)?;
            }
            fs::copy(entry.path(), &target).map_err(InfraError::Io)?;
        }
    }
    Ok(())
}

pub(super) fn create_validated_zip(
    source: &Path,
    destination_zip: &Path,
    expected_files: usize,
) -> Result<(), InfraError> {
    create_zip_from_dir(source, destination_zip)?;
    let file = File::open(destination_zip).map_err(InfraError::Io)?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;
    let mut archived_files = 0;
    for index in 0..archive.len() {
        let mut item = archive
            .by_index(index)
            .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;
        if item.is_dir() {
            continue;
        }
        archived_files += 1;
        std::io::copy(&mut item, &mut std::io::sink()).map_err(InfraError::Io)?;
    }
    if archived_files != expected_files {
        return Err(InfraError::UnexpectedOutput(
            destination_zip.display().to_string(),
            format!(
                "ZIP incompleto: esperados {expected_files} archivos, encontrados {archived_files}"
            ),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{build_inventory, ensure_process_success, is_internal_link};
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
}
