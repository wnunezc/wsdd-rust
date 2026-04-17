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
//! Apache vhost snapshot and regeneration helpers for project restore flows.

use std::fs;
use std::path::PathBuf;

use crate::config::environment::path_config;
use crate::errors::InfraError;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::project as project_handler;
use crate::handlers::yml;
use crate::models::project::{PhpVersion, Project};

const PERSONAL_PROJECTS_MARKER: &str = "### PERSONAL PROJECTS ###";

/// Captures the existing vhost block for a project, when present.
pub(super) fn capture_vhost_block(project: &Project) -> Result<Option<String>, InfraError> {
    let vhost_path = vhost_conf_path(project);
    if !vhost_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(vhost_path).map_err(InfraError::Io)?;
    Ok(extract_vhost_block(&content, &project.domain))
}

/// Captures the project-related lines from `options.yml`, when present.
pub(super) fn capture_options_snapshot(project: &Project) -> Result<Option<String>, InfraError> {
    let options_path = yml::options_path(
        project.php_version.dir_name(),
        project.php_version.compose_tag(),
    );
    let path = std::path::Path::new(&options_path);
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(path).map_err(InfraError::Io)?;
    let snapshot = content
        .lines()
        .filter(|line| {
            line.contains(&project.domain)
                || line.contains(&format!(
                    "{}-{}",
                    project.php_version.compose_tag(),
                    project.domain
                ))
        })
        .collect::<Vec<_>>()
        .join("\n");

    if snapshot.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(snapshot))
    }
}

/// Rebuilds the active vhost file for all projects that share a PHP version.
pub(super) fn sync_active_vhost_for_php(
    php_version: &PhpVersion,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let mut projects: Vec<Project> = project_handler::list_all()?
        .into_iter()
        .filter(|project| project.php_version == *php_version)
        .collect();
    projects.sort_by(|a, b| a.name.cmp(&b.name));

    let path = vhost_conf_path_for_php(php_version);
    let existing = fs::read_to_string(&path).map_err(InfraError::Io)?;
    let blocks = projects
        .iter()
        .map(render_vhost_block)
        .collect::<Result<Vec<_>, _>>()?;
    let rewritten = rewrite_personal_projects_section(&existing, &blocks);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(InfraError::Io)?;
    }
    fs::write(&path, rewritten).map_err(InfraError::Io)?;
    cleanup_legacy_vhost_file(php_version, tx);
    Ok(())
}

fn render_vhost_block(project: &Project) -> Result<String, InfraError> {
    let template = fs::read_to_string(vhost_template_path(project)).map_err(InfraError::Io)?;
    let protocol = if project.ssl {
        "Protocols h2 h2c http/1.1"
    } else {
        ""
    };
    Ok(template
        .replace("{CustomUrl}", &project.domain)
        .replace("{EntryPoint}", project.entry_point.as_path())
        .replace("{PROTOCOL}", protocol))
}

fn extract_vhost_block(content: &str, domain: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let marker = format!("ServerName {}", domain);
    let mut start_idx: Option<usize> = None;

    for (idx, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("<VirtualHost") {
            start_idx = Some(idx);
        }
        if line.contains(&marker) {
            if let Some(start) = start_idx {
                for end in idx..lines.len() {
                    if lines[end].trim_start().starts_with("</VirtualHost>") {
                        return Some(lines[start..=end].join("\n"));
                    }
                }
            }
        }
    }

    None
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

fn cleanup_legacy_vhost_file(php_version: &PhpVersion, tx: &LogSender) {
    let legacy_path = legacy_vhost_conf_path(php_version);
    if !legacy_path.exists() {
        return;
    }

    match fs::remove_file(&legacy_path) {
        Ok(()) => {
            let _ = tx.send(LogLine::info(
                "[Restore] Archivo legacy vhost.conf removido del flujo activo",
            ));
        }
        Err(e) => {
            let _ = tx.send(LogLine::warn(format!(
                "[Restore] Advertencia al remover vhost.conf legacy: {e}"
            )));
        }
    }
}

fn vhost_conf_path(project: &Project) -> PathBuf {
    vhost_conf_path_for_php(&project.php_version)
}

fn vhost_conf_path_for_php(php_version: &PhpVersion) -> PathBuf {
    path_config().active_vhost_conf(php_version.dir_name())
}

fn legacy_vhost_conf_path(php_version: &PhpVersion) -> PathBuf {
    path_config().legacy_vhost_conf(php_version.dir_name())
}

fn vhost_template_path(project: &Project) -> PathBuf {
    path_config().vhost_template(project.php_version.dir_name())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_vhost_block_for_domain() {
        let content = "\
### PERSONAL PROJECTS ###
<VirtualHost *:80>
    ServerName demo.dock
</VirtualHost>

<VirtualHost *:80>
    ServerName other.dock
</VirtualHost>
";

        let block = extract_vhost_block(content, "demo.dock").expect("block");
        assert!(block.contains("demo.dock"));
        assert!(!block.contains("other.dock"));
    }

    #[test]
    fn rewrite_personal_projects_section_preserves_marker() {
        let content = "### PERSONAL PROJECTS ###\n### PERSONAL PROJECTS ###\n";
        let result = rewrite_personal_projects_section(
            content,
            &[String::from(
                "<VirtualHost *:80>\nServerName demo.dock\n</VirtualHost>",
            )],
        );
        assert!(result.contains("### PERSONAL PROJECTS ###"));
        assert!(result.contains("demo.dock"));
    }
}
