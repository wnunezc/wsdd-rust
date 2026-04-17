use std::path::Path;

use crate::errors::InfraError;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::project as project_handler;
use crate::models::project::{PhpVersion, Project};

use super::paths;

const PERSONAL_PROJECTS_MARKER: &str = "### PERSONAL PROJECTS ###";

pub(super) fn update(project: &Project, tx: &LogSender) -> Result<(), InfraError> {
    let projects = php_projects(&project.php_version, None)?;
    sync_active_vhost(project.php_version.dir_name(), &projects, tx)?;
    let _ = tx.send(LogLine::success("[Deploy] vhost activo regenerado ✓"));
    Ok(())
}

pub(super) fn remove(project: &Project, tx: &LogSender) -> Result<(), InfraError> {
    let projects = php_projects(&project.php_version, Some(&project.name))?;
    sync_active_vhost(project.php_version.dir_name(), &projects, tx)?;
    let _ = tx.send(LogLine::success("[Remove] vhost activo regenerado ✓"));
    Ok(())
}

fn php_projects(
    php_version: &PhpVersion,
    exclude_project_name: Option<&str>,
) -> Result<Vec<Project>, InfraError> {
    let mut projects: Vec<Project> = project_handler::list_all()?
        .into_iter()
        .filter(|project| project.php_version == *php_version)
        .collect();

    if let Some(name) = exclude_project_name {
        projects.retain(|project| project.name != name);
    }

    Ok(projects)
}

fn sync_active_vhost(
    php_dir_name: &str,
    projects: &[Project],
    tx: &LogSender,
) -> Result<(), InfraError> {
    let template_path = paths::vhost_template_path(php_dir_name);
    let active_path = paths::active_vhost_conf_path(php_dir_name);

    let template = std::fs::read_to_string(&template_path).map_err(InfraError::Io)?;
    let active_content = std::fs::read_to_string(&active_path).map_err(InfraError::Io)?;

    let blocks: Vec<String> = projects
        .iter()
        .map(|project| render_project_vhost_block(&template, project))
        .collect();

    let rewritten = rewrite_personal_projects_section(&active_content, &blocks);
    std::fs::write(&active_path, rewritten).map_err(InfraError::Io)?;
    cleanup_legacy_vhost_file(php_dir_name, tx);
    Ok(())
}

fn render_project_vhost_block(template: &str, project: &Project) -> String {
    let protocol = if project.ssl {
        "Protocols h2 h2c http/1.1"
    } else {
        ""
    };

    template
        .replace("{CustomUrl}", &project.domain)
        .replace("{EntryPoint}", project.entry_point.as_path())
        .replace("{PROTOCOL}", protocol)
}

fn rewrite_personal_projects_section(content: &str, blocks: &[String]) -> String {
    let marker_positions: Vec<usize> = content
        .match_indices(PERSONAL_PROJECTS_MARKER)
        .map(|(idx, _)| idx)
        .collect();

    let rendered_blocks = if blocks.is_empty() {
        String::new()
    } else {
        format!("\n\n{}\n", blocks.join("\n\n"))
    };

    if marker_positions.len() >= 2 {
        let first_marker_end = marker_positions[0] + PERSONAL_PROJECTS_MARKER.len();
        let second_marker_start = marker_positions[1];
        format!(
            "{}{}{}",
            &content[..first_marker_end],
            rendered_blocks,
            &content[second_marker_start..]
        )
    } else if let Some(first_marker_start) = marker_positions.first().copied() {
        let first_marker_end = first_marker_start + PERSONAL_PROJECTS_MARKER.len();
        format!(
            "{}{}{}",
            &content[..first_marker_end],
            rendered_blocks,
            &content[first_marker_end..]
        )
    } else if blocks.is_empty() {
        content.to_string()
    } else {
        format!("{content}\n\n{}\n", blocks.join("\n\n"))
    }
}

fn cleanup_legacy_vhost_file(php_dir_name: &str, tx: &LogSender) {
    let legacy_path = paths::legacy_vhost_conf_path(php_dir_name);
    let legacy_file = Path::new(&legacy_path);
    if !legacy_file.exists() {
        return;
    }

    match std::fs::remove_file(legacy_file) {
        Ok(()) => {
            let _ = tx.send(LogLine::info(
                "[VHost] Archivo legacy vhost.conf removido del flujo activo",
            ));
        }
        Err(e) => {
            let _ = tx.send(LogLine::warn(format!(
                "[VHost] Advertencia al remover vhost.conf legacy: {e}"
            )));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::project::{EntryPoint, ProjectStatus};

    fn test_project(name: &str, php_version: PhpVersion, entry_point: EntryPoint) -> Project {
        Project {
            name: name.to_string(),
            domain: format!("{name}.dock"),
            php_version,
            work_path: format!(r"D:\Projects\{name}"),
            entry_point,
            ssl: true,
            status: ProjectStatus::Unknown,
        }
    }

    #[test]
    fn rewrite_personal_projects_section_replaces_existing_blocks() {
        let base = "\
### DO NOT TOUCH ###
### PERSONAL PROJECTS ###

<VirtualHost *:80>
    ServerName old-app.dock
</VirtualHost>
### PERSONAL PROJECTS ###
";
        let rewritten = rewrite_personal_projects_section(
            base,
            &[String::from(
                "<VirtualHost *:80>\n    ServerName new-app.dock\n</VirtualHost>",
            )],
        );

        assert!(rewritten.contains("ServerName new-app.dock"));
        assert!(!rewritten.contains("ServerName old-app.dock"));
    }

    #[test]
    fn render_project_vhost_block_uses_entry_point_and_ssl_protocol() {
        let template = "\
<VirtualHost *:80>
    ServerName {CustomUrl}
    DocumentRoot /var/www/html/{CustomUrl}{EntryPoint}
    {PROTOCOL}
</VirtualHost>";
        let block = render_project_vhost_block(
            template,
            &test_project("evangeline-shop", PhpVersion::Php84, EntryPoint::Public),
        );

        assert!(block.contains("ServerName evangeline-shop.dock"));
        assert!(block.contains("DocumentRoot /var/www/html/evangeline-shop.dock/public"));
        assert!(block.contains("Protocols h2 h2c http/1.1"));
    }

    #[test]
    fn php_projects_can_be_filtered_by_name() {
        let projects = vec![
            test_project("alpha", PhpVersion::Php84, EntryPoint::Root),
            test_project("beta", PhpVersion::Php84, EntryPoint::Root),
            test_project("gamma", PhpVersion::Php83, EntryPoint::Root),
        ];

        let filtered: Vec<Project> = projects
            .into_iter()
            .filter(|project| project.php_version == PhpVersion::Php84)
            .filter(|project| project.name != "beta")
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "alpha");
    }
}
