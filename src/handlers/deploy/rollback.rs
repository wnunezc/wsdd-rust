use std::path::PathBuf;

use crate::errors::InfraError;
use crate::handlers::hosts;
use crate::handlers::log_types::{LogLine, LogSender};

#[derive(Debug, Clone)]
pub(super) struct FileSnapshot {
    path: PathBuf,
    contents: Option<Vec<u8>>,
}

impl FileSnapshot {
    /// Captures a file for rollback, preserving missing-file state.
    pub(super) fn capture(path: impl Into<PathBuf>) -> Result<Self, InfraError> {
        let path = path.into();
        let contents = if path.exists() {
            Some(std::fs::read(&path).map_err(InfraError::Io)?)
        } else {
            None
        };
        Ok(Self { path, contents })
    }

    fn restore(&self) -> Result<(), InfraError> {
        match &self.contents {
            Some(contents) => {
                if let Some(parent) = self.path.parent() {
                    std::fs::create_dir_all(parent).map_err(InfraError::Io)?;
                }
                std::fs::write(&self.path, contents).map_err(InfraError::Io)?;
            }
            None => {
                if self.path.exists() {
                    std::fs::remove_file(&self.path).map_err(InfraError::Io)?;
                }
            }
        }
        Ok(())
    }
}

pub(super) struct DeployRollback {
    pub project_snapshot: FileSnapshot,
    pub options_snapshot: FileSnapshot,
    pub vhost_snapshot: FileSnapshot,
    pub ssl_snapshots: Vec<FileSnapshot>,
    pub hosts_snapshot: Vec<u8>,
    pub volume_created: bool,
    pub volume_existed_before: bool,
    pub container_rebuilt: bool,
    pub container_existed_before: bool,
}

pub(super) struct RemoveRollback {
    pub project_snapshot: FileSnapshot,
    pub options_snapshot: FileSnapshot,
    pub vhost_snapshot: FileSnapshot,
    pub ssl_snapshots: Vec<FileSnapshot>,
    pub hosts_snapshot: Vec<u8>,
    pub container_rebuilt: bool,
    pub volume_removed: bool,
}

pub(super) fn capture_hosts_snapshot() -> Result<Vec<u8>, InfraError> {
    hosts::capture_snapshot().map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))
}

pub(super) fn restore_file_snapshots(tx: &LogSender, snapshots: &[(&FileSnapshot, &str)]) {
    for (snapshot, label) in snapshots {
        restore_snapshot_best_effort(snapshot, label, tx);
    }
}

pub(super) fn restore_ssl_snapshots(snapshots: &[FileSnapshot], tx: &LogSender) {
    for snapshot in snapshots {
        restore_snapshot_best_effort(snapshot, "certificados SSL", tx);
    }
}

pub(super) fn restore_hosts_snapshot_best_effort(snapshot: &[u8], tx: &LogSender) {
    if let Err(e) = hosts::restore_snapshot(snapshot, Some(tx)) {
        let _ = tx.send(LogLine::warn(format!(
            "[Rollback] No se pudo restaurar hosts: {e}"
        )));
    }
}

fn restore_snapshot_best_effort(snapshot: &FileSnapshot, label: &str, tx: &LogSender) {
    if let Err(e) = snapshot.restore() {
        let _ = tx.send(LogLine::warn(format!(
            "[Rollback] No se pudo restaurar {label}: {e}"
        )));
    }
}
