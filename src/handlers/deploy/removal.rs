use std::collections::BTreeSet;

use crate::config::environment::env_config;
use crate::errors::InfraError;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::ps_script::{PsRunner, ScriptRunner};
use crate::models::project::{PhpVersion, Project};

fn selection(
    projects: Vec<Project>,
    names: &[String],
    container: Option<&str>,
) -> Result<Vec<Project>, InfraError> {
    let requested: BTreeSet<&str> = names.iter().map(String::as_str).collect();
    if requested.len() != names.len() || (names.is_empty() && container.is_none()) {
        return Err(InfraError::PrerequisiteNotMet(
            "Invalid project selection".into(),
        ));
    }
    let selected = if let Some(name) = container {
        let version = PhpVersion::all()
            .into_iter()
            .find(|v| name == format!("WSDD-Web-Server-{}", v.runtime_container_tag()))
            .ok_or_else(|| {
                InfraError::PrerequisiteNotMet(
                    "Only WSDD PHP containers can be removed here".into(),
                )
            })?;
        projects
            .into_iter()
            .filter(|p| p.php_version == version)
            .collect::<Vec<_>>()
    } else {
        projects
            .into_iter()
            .filter(|p| requested.contains(p.name.as_str()))
            .collect()
    };
    let actual: BTreeSet<&str> = selected.iter().map(|p| p.name.as_str()).collect();
    if actual != requested {
        return Err(InfraError::PrerequisiteNotMet(
            "Projects changed since confirmation. Reload and confirm again.".into(),
        ));
    }
    Ok(selected)
}

pub(super) fn remove_projects(
    names: &[String],
    container: Option<&str>,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let projects = selection(
        crate::handlers::project::list_all_strict()?,
        names,
        container,
    )?;
    let mut runtime_names: BTreeSet<String> = projects
        .iter()
        .map(super::paths::php_container_name)
        .collect();
    if let Some(name) = container {
        runtime_names.insert(name.to_string());
    }
    for name in runtime_names {
        crate::handlers::docker::require_container_project(
            runner,
            &name,
            crate::handlers::docker::WSDD_WEB_PROJECT,
        )?;
    }
    execute_removal(
        &projects,
        || {
            if let Some(name) = container {
                let version = PhpVersion::from_container_name(name).ok_or_else(|| {
                    InfraError::PrerequisiteNotMet("Invalid PHP container".into())
                })?;
                if crate::handlers::docker::php_container_exists_sync(
                    runner,
                    version.runtime_container_tag(),
                )? {
                    let output = runner.run_direct_sync(
                        env_config().docker_exe(),
                        &["rm", "--force", name],
                        None,
                        None,
                    )?;
                    if !output.success {
                        return Err(InfraError::UnexpectedOutput(
                            "docker rm".into(),
                            output.text,
                        ));
                    }
                }
                let _ = tx.send(LogLine::success(format!(
                    "[Remove] Contenedor {name} eliminado; limpiando proyectos asociados"
                )));
            }
            Ok(())
        },
        |project| super::remove_project(project, runner, tx),
    )?;
    let _ = tx.send(LogLine::success(format!(
        "[Remove] {}/{} proyectos eliminados",
        projects.len(),
        projects.len()
    )));
    Ok(())
}

fn execute_removal(
    projects: &[Project],
    remove_runtime: impl FnOnce() -> Result<(), InfraError>,
    mut remove_project: impl FnMut(&Project) -> Result<(), InfraError>,
) -> Result<(), InfraError> {
    remove_runtime()?;
    for (index, project) in projects.iter().enumerate() {
        if let Err(error) = remove_project(project) {
            return Err(InfraError::PrerequisiteNotMet(format!(
                "Removed {index}/{} projects. Stopped at '{}': {error}",
                projects.len(),
                project.name
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn projects() -> Vec<Project> {
        vec![
            Project::new("a".into(), "a.dock", PhpVersion::Php84, "C:/a".into()),
            Project::new("b".into(), "b.dock", PhpVersion::Php83, "C:/b".into()),
            Project::new("c".into(), "c.dock", PhpVersion::Php84, "C:/c".into()),
        ]
    }

    #[test]
    fn batch_selects_only_confirmed_projects() {
        let result = selection(projects(), &["b".into()], None).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "b");
    }

    #[test]
    fn cascade_requires_every_project_of_exact_php_version() {
        let names = vec!["a".into(), "c".into()];
        let selected = selection(projects(), &names, Some("WSDD-Web-Server-PHP8.4")).unwrap();
        assert_eq!(selected.len(), 2);
        assert!(selected.iter().all(|p| p.php_version == PhpVersion::Php84));
        assert!(selection(projects(), &["a".into()], Some("WSDD-Web-Server-PHP8.4")).is_err());
        assert!(selection(projects(), &["b".into()], Some("WSDD-Web-Server-PHP8.4")).is_err());
    }

    #[test]
    fn rejects_base_containers_missing_projects_and_duplicates() {
        assert!(selection(projects(), &[], Some("WSDD-MySql-Server")).is_err());
        assert!(selection(projects(), &["missing".into()], None).is_err());
        assert!(selection(projects(), &["a".into(), "a".into()], None).is_err());
        assert!(selection(projects(), &[], None).is_err());
        assert!(selection(projects(), &[], Some("WSDD-Web-Server-PHP5.6")).is_ok());
    }

    #[test]
    fn failed_container_removal_keeps_all_project_records() {
        let mut touched = false;
        let result = execute_removal(
            &projects(),
            || Err(InfraError::PrerequisiteNotMet("docker rm failed".into())),
            |_| {
                touched = true;
                Ok(())
            },
        );
        assert!(result.is_err());
        assert!(!touched);
    }

    #[test]
    fn removes_container_before_projects_and_stops_at_first_failure() {
        let events = std::cell::RefCell::new(Vec::new());
        let result = execute_removal(
            &projects(),
            || {
                events.borrow_mut().push("container".to_owned());
                Ok(())
            },
            |p| {
                events.borrow_mut().push(p.name.clone());
                if p.name == "b" {
                    Err(InfraError::PrerequisiteNotMet("volume in use".into()))
                } else {
                    Ok(())
                }
            },
        );
        assert_eq!(*events.borrow(), vec!["container", "a", "b"]);
        assert!(result.unwrap_err().to_string().contains("1/3"));
    }

    #[test]
    fn successful_batch_removes_every_selected_project() {
        let mut removed = Vec::new();
        execute_removal(
            &projects(),
            || Ok(()),
            |p| {
                removed.push(p.name.clone());
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(removed, vec!["a", "b", "c"]);
    }
}
