use crate::config::environment::env_config;
use crate::errors::InfraError;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::ps_script::{PsRunner, ScriptRunner};
use crate::models::project::{is_native_linux_path, Project, StorageMode};

use super::paths;

/// Effective Docker mount strategy for a WSDD project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeMountStrategy {
    pub volume_name: String,
    pub mode: StorageMode,
    pub source_path: String,
    pub runtime_path: String,
    pub requires_initial_sync: bool,
}

impl RuntimeMountStrategy {
    /// Builds the Docker volume create arguments for the selected storage mode.
    pub fn volume_create_args(&self) -> Vec<String> {
        match self.mode {
            StorageMode::ClassicBind
            | StorageMode::NativeLinuxProject
            | StorageMode::ManagedNativeLinuxStorage => vec![
                "volume".to_string(),
                "create".to_string(),
                "--driver".to_string(),
                "local".to_string(),
                "--opt".to_string(),
                "type=none".to_string(),
                "--opt".to_string(),
                format!("device={}", self.source_path),
                "--opt".to_string(),
                "o=bind".to_string(),
                self.volume_name.clone(),
            ],
            StorageMode::FastSyncWorkspace => vec![
                "volume".to_string(),
                "create".to_string(),
                self.volume_name.clone(),
            ],
        }
    }
}

/// Resolves the runtime mount strategy for a project without touching Docker.
///
/// # Errors
/// Returns [`InfraError`] when a storage mode has an invalid source path.
pub fn resolve(project: &Project) -> Result<RuntimeMountStrategy, InfraError> {
    validate_storage_source(project)?;
    let volume_name = paths::volume_name(project);
    let (source_path, runtime_path) = match project.storage_mode {
        StorageMode::ClassicBind => (project.work_path.clone(), project.work_path.clone()),
        StorageMode::FastSyncWorkspace => {
            let runtime_path = format!("docker-volume://{volume_name}");
            (project.work_path.clone(), runtime_path)
        }
        StorageMode::NativeLinuxProject => (
            native_linux_docker_source(&project.work_path)?,
            native_linux_runtime_path(&project.work_path)?,
        ),
        StorageMode::ManagedNativeLinuxStorage => {
            let storage = project.managed_storage.as_ref().ok_or_else(|| {
                InfraError::PrerequisiteNotMet(
                    "Managed Native Linux Storage requires project metadata".to_string(),
                )
            })?;
            (storage.windows_unc.clone(), storage.linux_path.clone())
        }
    };

    Ok(RuntimeMountStrategy {
        volume_name,
        mode: project.storage_mode,
        source_path,
        runtime_path,
        requires_initial_sync: project.storage_mode == StorageMode::FastSyncWorkspace,
    })
}

/// Copies the Windows source tree into the Docker-native runtime volume.
///
/// This operation intentionally runs through a short-lived helper container so
/// the PHP/runtime containers do not need to be restarted merely to sync files.
pub fn sync_fast_workspace(
    strategy: &RuntimeMountStrategy,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    if strategy.mode != StorageMode::FastSyncWorkspace {
        return Ok(());
    }

    let _ = tx.send(LogLine::info(format!(
        "[Storage] Sincronizando workspace rapido {}...",
        strategy.volume_name
    )));

    let source_mount = format!("{}:/source:ro", strategy.source_path);
    let target_mount = format!("{}:/target", strategy.volume_name);
    let sync_script =
        "set -eu; find /target -mindepth 1 -maxdepth 1 -exec rm -rf {} +; cp -a /source/. /target/";
    let out = runner.run_direct_sync(
        env_config().docker_exe(),
        &[
            "run",
            "--rm",
            "-v",
            &source_mount,
            "-v",
            &target_mount,
            "alpine:3.20",
            "sh",
            "-c",
            sync_script,
        ],
        None,
        None,
    )?;

    if !out.success {
        return Err(InfraError::DockerUnreachable(format!(
            "No se pudo sincronizar {}: {}",
            strategy.volume_name, out.text
        )));
    }

    let _ = tx.send(LogLine::success(format!(
        "[Storage] Workspace rapido {} sincronizado ✓",
        strategy.volume_name
    )));
    Ok(())
}

fn validate_storage_source(project: &Project) -> Result<(), InfraError> {
    if project.work_path.trim().is_empty() {
        return Err(InfraError::PrerequisiteNotMet(
            "Project work path cannot be empty".to_string(),
        ));
    }

    if project.storage_mode == StorageMode::NativeLinuxProject
        && !is_native_linux_path(&project.work_path)
    {
        return Err(InfraError::PrerequisiteNotMet(
            "Native Linux Project mode requires a Linux, WSL, or Docker-native path".to_string(),
        ));
    }

    if project.storage_mode == StorageMode::ManagedNativeLinuxStorage
        && project.managed_storage.is_none()
    {
        return Err(InfraError::PrerequisiteNotMet(
            "Managed Native Linux Storage requires project metadata".to_string(),
        ));
    }

    Ok(())
}

fn native_linux_runtime_path(path: &str) -> Result<String, InfraError> {
    let trimmed = path.trim();
    if trimmed.starts_with('/') {
        return Ok(trimmed.to_string());
    }

    if let Some(linux_path) = wsl_unc_to_linux_path(trimmed) {
        return Ok(linux_path);
    }

    Err(InfraError::PrerequisiteNotMet(
        "Native Linux Project mode requires a Linux, WSL, or Docker-native path".to_string(),
    ))
}

fn native_linux_docker_source(path: &str) -> Result<String, InfraError> {
    let trimmed = path.trim();
    if trimmed.starts_with('/') || wsl_unc_to_linux_path(trimmed).is_some() {
        return Ok(trimmed.to_string());
    }

    Err(InfraError::PrerequisiteNotMet(
        "Native Linux Project mode requires a Linux, WSL, or Docker-native path".to_string(),
    ))
}

fn wsl_unc_to_linux_path(path: &str) -> Option<String> {
    let rest = path
        .strip_prefix(r"\\wsl.localhost\")
        .or_else(|| path.strip_prefix(r"\\wsl$\"))?;
    let mut parts = rest.split('\\');
    let _distro = parts.next()?.trim();
    let linux_path = parts.collect::<Vec<_>>().join("/");
    if linux_path.is_empty() {
        return None;
    }
    Some(format!("/{linux_path}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::project::{PhpVersion, Project};

    fn project(mode: StorageMode, work_path: &str) -> Project {
        Project {
            name: "demo".to_string(),
            domain: "demo.dock".to_string(),
            php_version: PhpVersion::Php83,
            work_path: work_path.to_string(),
            storage_mode: mode,
            ..Project::default()
        }
    }

    #[test]
    fn classic_bind_uses_existing_bind_volume_contract() {
        let strategy = resolve(&project(StorageMode::ClassicBind, r"C:\projects\demo")).unwrap();

        assert_eq!(strategy.volume_name, "php83-demo.dock");
        assert!(!strategy.requires_initial_sync);
        assert!(strategy
            .volume_create_args()
            .contains(&"type=none".to_string()));
        assert!(strategy
            .volume_create_args()
            .contains(&r"device=C:\projects\demo".to_string()));
    }

    #[test]
    fn fast_sync_uses_docker_native_volume_and_requires_sync() {
        let strategy = resolve(&project(
            StorageMode::FastSyncWorkspace,
            r"C:\projects\demo",
        ))
        .unwrap();

        assert_eq!(strategy.runtime_path, "docker-volume://php83-demo.dock");
        assert!(strategy.requires_initial_sync);
        assert_eq!(
            strategy.volume_create_args(),
            vec!["volume", "create", "php83-demo.dock"]
        );
    }

    #[test]
    fn native_linux_project_rejects_windows_path() {
        let err = resolve(&project(
            StorageMode::NativeLinuxProject,
            r"C:\projects\demo",
        ))
        .expect_err("windows paths must be rejected");

        assert!(err.to_string().contains("Native Linux Project mode"));
    }

    #[test]
    fn native_linux_project_accepts_absolute_linux_path() {
        let strategy = resolve(&project(
            StorageMode::NativeLinuxProject,
            "/home/wsdd/projects/demo",
        ))
        .unwrap();

        assert_eq!(strategy.source_path, "/home/wsdd/projects/demo");
        assert_eq!(strategy.runtime_path, "/home/wsdd/projects/demo");
        assert!(strategy
            .volume_create_args()
            .contains(&"device=/home/wsdd/projects/demo".to_string()));
    }

    #[test]
    fn native_linux_project_converts_wsl_unc_to_linux_path_for_docker() {
        let strategy = resolve(&project(
            StorageMode::NativeLinuxProject,
            r"\\wsl.localhost\Ubuntu\home\wsdd\projects\demo",
        ))
        .unwrap();

        assert_eq!(
            strategy.source_path,
            r"\\wsl.localhost\Ubuntu\home\wsdd\projects\demo"
        );
        assert_eq!(strategy.runtime_path, "/home/wsdd/projects/demo");
        assert!(strategy
            .volume_create_args()
            .contains(&r"device=\\wsl.localhost\Ubuntu\home\wsdd\projects\demo".to_string()));
    }

    #[test]
    fn managed_native_linux_storage_uses_managed_linux_runtime_path() {
        let mut project = project(
            StorageMode::ManagedNativeLinuxStorage,
            r"C:\WSDD-Projects\demo",
        );
        project.managed_storage = Some(crate::models::project::ManagedNativeStorage {
            distro: "WSDD-Storage".to_string(),
            linux_path: "/home/wsdd/projects/demo".to_string(),
            windows_unc: r"\\wsl.localhost\WSDD-Storage\home\wsdd\projects\demo".to_string(),
            windows_alias: Some(r"C:\WSDD-Projects\demo".to_string()),
        });

        let strategy = resolve(&project).unwrap();

        assert_eq!(
            strategy.source_path,
            r"\\wsl.localhost\WSDD-Storage\home\wsdd\projects\demo"
        );
        assert_eq!(strategy.runtime_path, "/home/wsdd/projects/demo");
        assert!(strategy
            .volume_create_args()
            .contains(&r"device=\\wsl.localhost\WSDD-Storage\home\wsdd\projects\demo".to_string()));
    }
}
