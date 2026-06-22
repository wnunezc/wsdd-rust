use crate::models::project::PhpVersion;
use crate::models::project::{Project, StorageMode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WslRuntimeTarget {
    pub project_name: String,
    pub domain: String,
    pub php_version: PhpVersion,
    pub volume_name: String,
    pub distro: String,
    pub linux_path: String,
    pub windows_unc: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetResolution {
    Target(WslRuntimeTarget),
    NotWslDependent,
    UnresolvedNative {
        project_name: String,
        reason: String,
    },
}

pub fn resolve_project(project: &Project, default_wsl_distro: Option<&str>) -> TargetResolution {
    let resolved = match project.storage_mode {
        StorageMode::ManagedNativeLinuxStorage => project
            .managed_storage
            .as_ref()
            .map(|storage| {
                (
                    storage.distro.clone(),
                    storage.linux_path.clone(),
                    storage.windows_unc.clone(),
                )
            })
            .ok_or_else(|| "Managed project metadata is missing".to_string()),
        StorageMode::NativeLinuxProject => resolve_native_path(
            &project.work_path,
            default_wsl_distro.filter(|value| !value.trim().is_empty()),
        ),
        StorageMode::ClassicBind | StorageMode::FastSyncWorkspace => {
            return TargetResolution::NotWslDependent;
        }
    };

    match resolved {
        Ok((distro, linux_path, windows_unc)) => TargetResolution::Target(WslRuntimeTarget {
            project_name: project.name.clone(),
            domain: project.domain.clone(),
            php_version: project.php_version.clone(),
            volume_name: format!("{}-{}", project.php_version.compose_tag(), project.domain),
            distro,
            linux_path,
            windows_unc,
        }),
        Err(reason) => TargetResolution::UnresolvedNative {
            project_name: project.name.clone(),
            reason,
        },
    }
}

pub fn resolve_all(
    projects: &[Project],
    default_wsl_distro: Option<&str>,
) -> Vec<TargetResolution> {
    projects
        .iter()
        .map(|project| resolve_project(project, default_wsl_distro))
        .collect()
}

pub fn has_wsl_dependent_projects(projects: &[Project], default_wsl_distro: Option<&str>) -> bool {
    resolve_all(projects, default_wsl_distro)
        .iter()
        .any(|item| {
            matches!(
                item,
                TargetResolution::Target(_) | TargetResolution::UnresolvedNative { .. }
            )
        })
}

fn resolve_native_path(
    path: &str,
    default_wsl_distro: Option<&str>,
) -> Result<(String, String, String), String> {
    let trimmed = path.trim();
    if let Some(rest) = trimmed
        .strip_prefix(r"\\wsl.localhost\")
        .or_else(|| trimmed.strip_prefix(r"\\wsl$\"))
    {
        let mut parts = rest.split('\\');
        let distro = parts.next().unwrap_or_default().trim();
        let tail = parts.collect::<Vec<_>>();
        if distro.is_empty() || tail.is_empty() {
            return Err(format!("Invalid WSL UNC path: {trimmed}"));
        }
        let linux_path = format!("/{}", tail.join("/"));
        let windows_unc = format!(r"\\wsl.localhost\{}\{}", distro, tail.join(r"\"));
        return Ok((distro.to_string(), linux_path, windows_unc));
    }

    if trimmed.starts_with('/') {
        let distro = default_wsl_distro
            .ok_or_else(|| "Absolute Linux path requires the configured WSL distro".to_string())?;
        let windows_tail = trimmed.trim_start_matches('/').replace('/', r"\");
        return Ok((
            distro.to_string(),
            trimmed.to_string(),
            format!(r"\\wsl.localhost\{distro}\{windows_tail}"),
        ));
    }

    Err(format!("Native project path is not WSL-backed: {trimmed}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::project::{ManagedNativeStorage, Project};

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
    fn managed_project_resolves_metadata_paths() {
        let mut value = project(
            StorageMode::ManagedNativeLinuxStorage,
            r"C:\WSDD-Projects\demo",
        );
        value.managed_storage = Some(ManagedNativeStorage {
            distro: "WSDD-Storage".to_string(),
            linux_path: "/home/wsdd/projects/demo".to_string(),
            windows_unc: r"\\wsl.localhost\WSDD-Storage\home\wsdd\projects\demo".to_string(),
            windows_alias: Some(r"C:\WSDD-Projects\demo".to_string()),
        });

        let TargetResolution::Target(target) = resolve_project(&value, None) else {
            panic!("managed project must resolve");
        };
        assert_eq!(target.distro, "WSDD-Storage");
        assert_eq!(target.linux_path, "/home/wsdd/projects/demo");
        assert_eq!(target.volume_name, "php83-demo.dock");
    }

    #[test]
    fn native_wsl_localhost_unc_is_resolved() {
        let value = project(
            StorageMode::NativeLinuxProject,
            r"\\wsl.localhost\Ubuntu\home\dev\demo",
        );
        let TargetResolution::Target(target) = resolve_project(&value, None) else {
            panic!("native UNC must resolve");
        };
        assert_eq!(target.distro, "Ubuntu");
        assert_eq!(target.linux_path, "/home/dev/demo");
    }

    #[test]
    fn native_wsl_dollar_unc_is_resolved() {
        let value = project(
            StorageMode::NativeLinuxProject,
            r"\\wsl$\Debian\home\dev\demo",
        );
        let TargetResolution::Target(target) = resolve_project(&value, None) else {
            panic!("legacy UNC must resolve");
        };
        assert_eq!(target.distro, "Debian");
        assert_eq!(target.windows_unc, r"\\wsl.localhost\Debian\home\dev\demo");
    }

    #[test]
    fn native_absolute_path_uses_configured_default_distro() {
        let value = project(StorageMode::NativeLinuxProject, "/home/dev/demo");
        let TargetResolution::Target(target) = resolve_project(&value, Some("Ubuntu")) else {
            panic!("absolute Linux path must use configured distro");
        };
        assert_eq!(target.distro, "Ubuntu");
        assert_eq!(target.windows_unc, r"\\wsl.localhost\Ubuntu\home\dev\demo");
    }

    #[test]
    fn classic_and_fast_sync_are_ignored() {
        for mode in [StorageMode::ClassicBind, StorageMode::FastSyncWorkspace] {
            assert!(matches!(
                resolve_project(&project(mode, r"C:\projects\demo"), None),
                TargetResolution::NotWslDependent
            ));
        }
    }

    #[test]
    fn unresolved_native_absolute_path_is_reported_without_guessing() {
        let value = project(StorageMode::NativeLinuxProject, "/home/dev/demo");
        assert!(matches!(
            resolve_project(&value, None),
            TargetResolution::UnresolvedNative { .. }
        ));
    }
}
