use crate::config::environment::env_config;
use crate::errors::InfraError;
use crate::handlers::ps_script::ScriptRunner;

/// Expected Compose owner for a WSDD-managed fixed container name.
pub fn expected_project_for_container(name: &str) -> Option<&'static str> {
    use super::{WSDD_CORE_PROJECT, WSDD_TOOLS_PROJECT, WSDD_WEB_PROJECT};
    match name {
        "WSDD-Proxy-Server" | "WSDD-MySql-Server" | "WSDD-phpMyAdmin-Server" => {
            Some(WSDD_CORE_PROJECT)
        }
        "WSDD-Redis-Server"
        | "WSDD-Memcached-Server"
        | "WSDD-Mailpit-Server"
        | "WSDD-Ngrok-Server" => Some(WSDD_TOOLS_PROJECT),
        "WSDD-MySql-Server-Test" => Some("wsdd-mysql-test"),
        _ if crate::models::project::PhpVersion::all()
            .into_iter()
            .any(|version| {
                name == format!("WSDD-Web-Server-{}", version.runtime_container_tag())
            }) =>
        {
            Some(WSDD_WEB_PROJECT)
        }
        _ => None,
    }
}

/// Refuses to operate on a container owned by another Compose project.
/// An existing container cannot acquire a different Compose project label.
pub fn require_container_project(
    runner: &impl ScriptRunner,
    container_name: &str,
    expected_project: &str,
) -> Result<(), InfraError> {
    let out = runner.run_direct_sync(
        env_config().docker_exe(),
        &[
            "ps",
            "-a",
            "--format",
            r#"{{.Names}}|{{.Label "com.docker.compose.project"}}"#,
            "--filter",
            &format!("name={container_name}"),
        ],
        None,
        None,
    )?;
    if !out.success {
        return Err(InfraError::DockerUnreachable(out.text));
    }
    check_project_label(&out.text, container_name, expected_project)
}

fn check_project_label(
    inventory: &str,
    container_name: &str,
    expected_project: &str,
) -> Result<(), InfraError> {
    for line in inventory.lines() {
        let Some((name, project)) = line.trim().split_once('|') else {
            continue;
        };
        if name != container_name {
            continue;
        }
        if project == expected_project {
            return Ok(());
        }
        let owner = if project.is_empty() {
            "sin proyecto Compose"
        } else {
            project
        };
        return Err(InfraError::PrerequisiteNotMet(format!(
            "{container_name} pertenece a {owner}; WSDD esperaba {expected_project}. No se modifico el contenedor. Tras respaldar sus datos, retire manualmente el contenedor heredado y vuelva a desplegar con WSDD."
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::check_project_label;

    #[test]
    fn exact_container_with_expected_project_is_accepted() {
        let inventory = "WSDD-Proxy-Server-old|docker-structure\nWSDD-Proxy-Server|wsdd-core\n";
        assert!(check_project_label(inventory, "WSDD-Proxy-Server", "wsdd-core").is_ok());
    }

    #[test]
    fn legacy_project_is_rejected_without_mutation() {
        let error = check_project_label(
            "WSDD-Proxy-Server|docker-structure\n",
            "WSDD-Proxy-Server",
            "wsdd-core",
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("docker-structure"));
        assert!(error.contains("No se modifico"));
    }

    #[test]
    fn absent_container_is_accepted() {
        assert!(check_project_label("", "WSDD-Proxy-Server", "wsdd-core").is_ok());
    }
}
