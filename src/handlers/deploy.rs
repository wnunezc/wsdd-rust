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
//! Despliegue y eliminacion de proyectos WSDD.
//!
//! Equivalente a `HandlerProject.DeployNewProjectAsync` y al flujo de remove en C#.
//! Este modulo orquesta la persistencia del proyecto y los cambios de infra:
//! volumen, options.yml, vhost activo, SSL y hosts.
//!
//! Todas las funciones son sincronicas; llamar desde `std::thread::spawn`.

use std::path::Path;

use crate::errors::InfraError;
use crate::handlers::docker;
use crate::handlers::docker::WSDD_PROJECT;
use crate::handlers::docker_deploy;
use crate::handlers::docker_deploy::{make_docker_progress_bridge, make_log_bridge};
use crate::handlers::hosts;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::project as project_handler;
use crate::handlers::ps_script::{PsRunner, ScriptRunner};
use crate::handlers::setting::AppSettings;
use crate::handlers::yml;
use crate::models::project::{PhpVersion, Project};

const SSL_DIR: &str = r"C:\WSDD-Environment\Docker-Structure\ssl";
const PERSONAL_PROJECTS_MARKER: &str = "### PERSONAL PROJECTS ###";

fn php_bin_dir(php_dir_name: &str) -> String {
    format!(r"C:\WSDD-Environment\Docker-Structure\bin\{}", php_dir_name)
}

fn active_vhost_conf_path(php_dir_name: &str) -> String {
    format!(
        r"C:\WSDD-Environment\Docker-Structure\bin\{}\vhost\vhost.conf",
        php_dir_name
    )
}

fn legacy_vhost_conf_path(php_dir_name: &str) -> String {
    format!(
        r"C:\WSDD-Environment\Docker-Structure\bin\{}\vhost.conf",
        php_dir_name
    )
}

fn vhost_template_path(php_dir_name: &str) -> String {
    format!(
        r"C:\WSDD-Environment\Docker-Structure\bin\{}\tpl.vhost.conf",
        php_dir_name
    )
}

fn webserver_yml_path(php_dir_name: &str, compose_tag: &str) -> String {
    format!(
        r"C:\WSDD-Environment\Docker-Structure\bin\{}\webserver.{}.yml",
        php_dir_name, compose_tag
    )
}

/// Despliega un proyecto WSDD completo.
///
/// Flujo:
/// 1. Guarda el proyecto en disco.
/// 2. Crea el volumen Docker del proyecto.
/// 3. Agrega el dominio al `options.phpXX.yml`.
/// 4. Sincroniza los recursos base de PHP/Webmin.
/// 5. Regenera el vhost activo desde `projects/*.json`.
/// 6. Reconstruye el contenedor PHP para que Apache levante con el vhost actualizado.
/// 7. Si `ssl=true`, genera certificado y reinicia el proxy.
/// 8. Actualiza el archivo `hosts`.
pub fn deploy_project(
    project: &Project,
    settings: &AppSettings,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let _ = tx.send(LogLine::info(format!(
        "[Deploy] Iniciando despliegue de '{}'...",
        project.name
    )));

    project_handler::save(project)?;
    let _ = tx.send(LogLine::success("[Deploy] Proyecto guardado ✓"));

    step_create_volume(project, runner, tx)?;
    step_update_options_yml(project, tx)?;

    docker_deploy::sync_php_version_resources_sync(settings, &project.php_version)?;
    let _ = tx.send(LogLine::success(
        "[Deploy] Recursos gestionados de PHP/Webmin sincronizados ✓",
    ));

    step_update_vhost(project, tx)?;
    step_rebuild_php_container(project, runner, tx)?;

    if project.ssl {
        step_setup_ssl(project, runner, tx)?;
    }

    step_update_hosts(project, tx)?;

    let _ = tx.send(LogLine::success(format!(
        "[Deploy] '{}' desplegado correctamente ✓",
        project.name
    )));
    Ok(())
}

/// Elimina un proyecto WSDD completo.
///
/// Flujo:
/// 1. Elimina el dominio de `options.phpXX.yml`.
/// 2. Regenera el vhost activo excluyendo el proyecto objetivo.
/// 3. Reconstruye el contenedor PHP.
/// 4. Elimina el volumen Docker.
/// 5. Borra el proyecto del disco.
///
/// Los dominios no se eliminan de `hosts` (limitacion conocida).
pub fn remove_project(
    project: &Project,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let _ = tx.send(LogLine::info(format!(
        "[Remove] Eliminando proyecto '{}'...",
        project.name
    )));

    step_remove_options_yml(project, tx);
    step_remove_vhost(project, tx);
    step_rebuild_php_container(project, runner, tx)?;
    step_remove_volume(project, runner, tx);
    project_handler::delete(&project.name)?;

    let _ = tx.send(LogLine::success(format!(
        "[Remove] '{}' eliminado ✓",
        project.name
    )));
    Ok(())
}

fn step_create_volume(
    project: &Project,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let compose_tag = project.php_version.compose_tag();
    let volume_name = format!("{}-{}", compose_tag, project.domain);
    let device_opt = format!("device={}", project.work_path);

    let _ = tx.send(LogLine::info(format!(
        "[Deploy] Creando volumen Docker '{}'...",
        volume_name
    )));

    let bridge = make_log_bridge(tx);
    runner.run_direct_sync(
        "docker",
        &[
            "volume",
            "create",
            "--driver",
            "local",
            "--opt",
            "type=none",
            "--opt",
            &device_opt,
            "--opt",
            "o=bind",
            &volume_name,
        ],
        None,
        Some(&bridge),
    )?;

    let _ = tx.send(LogLine::success(format!(
        "[Deploy] Volumen '{}' creado ✓",
        volume_name
    )));
    Ok(())
}

fn step_update_options_yml(project: &Project, tx: &LogSender) -> Result<(), InfraError> {
    let options_file = yml::options_path(
        project.php_version.dir_name(),
        project.php_version.compose_tag(),
    );
    yml::add_project_to_options_yml(
        &options_file,
        &project.domain,
        project.php_version.compose_tag(),
    )?;
    let _ = tx.send(LogLine::success("[Deploy] options.yml actualizado ✓"));
    Ok(())
}

/// Stop -> rm -> recreate del contenedor PHP del proyecto.
///
/// Solo fuerza `--build` cuando el contenedor PHP aún no existe.
/// En altas/bajas normales de proyectos basta recrear el contenedor para que
/// Apache recoja `options.phpXX.yml` y `vhost.conf`, evitando builds basura.
fn step_rebuild_php_container(
    project: &Project,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let container_name = format!("WSDD-Web-Server-{}", project.php_version.container_tag());
    let php_dir_name = project.php_version.dir_name();
    let compose_tag = project.php_version.compose_tag();
    let should_build =
        docker::php_container_exists_sync(runner, project.php_version.container_tag())
            .map(|exists| !exists)
            .unwrap_or(true);

    let bin_dir_str = php_bin_dir(php_dir_name);
    let bin_dir = Path::new(&bin_dir_str);
    let webserver_yml = webserver_yml_path(php_dir_name, compose_tag);
    let options_yml = yml::options_path(php_dir_name, compose_tag);

    let _ = tx.send(LogLine::info(format!(
        "[Deploy] Deteniendo {}...",
        container_name
    )));
    let _ = runner.run_direct_sync("docker", &["stop", &container_name], None, None);
    let _ = runner.run_direct_sync("docker", &["rm", &container_name], None, None);

    let _ = tx.send(LogLine::info(if should_build {
        "[Deploy] Construyendo y creando contenedor PHP (puede tardar)..."
    } else {
        "[Deploy] Recreando contenedor PHP..."
    }));
    let bridge = make_docker_progress_bridge(tx);
    runner.run_ps_sync(
        &format!(
            "docker-compose -p {WSDD_PROJECT} -f \"{webserver_yml}\" -f \"{options_yml}\" up -d {}--force-recreate",
            if should_build { "--build " } else { "" }
        ),
        Some(bin_dir),
        Some(&bridge),
    )?;

    let _ = tx.send(LogLine::success(format!(
        "[Deploy] {} reconstruido ✓",
        container_name
    )));
    Ok(())
}

/// Regenera la seccion PERSONAL PROJECTS del vhost activo para la version PHP.
///
/// La fuente de verdad son los `projects/*.json` persistidos por WSDD.
fn step_update_vhost(project: &Project, tx: &LogSender) -> Result<(), InfraError> {
    let projects = php_projects(&project.php_version, None)?;
    sync_active_vhost(project.php_version.dir_name(), &projects, tx)?;
    let _ = tx.send(LogLine::success("[Deploy] vhost activo regenerado ✓"));
    Ok(())
}

/// Genera el certificado SSL con mkcert y reinicia el proxy.
fn step_setup_ssl(project: &Project, runner: &PsRunner, tx: &LogSender) -> Result<(), InfraError> {
    std::fs::create_dir_all(SSL_DIR).map_err(InfraError::Io)?;

    let cert_file = format!(r"{}\{}.crt", SSL_DIR, project.domain);
    let key_file = format!(r"{}\{}.key", SSL_DIR, project.domain);
    let wildcard = format!("*.{}", project.domain);

    let _ = tx.send(LogLine::info(
        "[Deploy] Generando certificado SSL (mkcert)...",
    ));
    runner.run_ps_sync(
        &format!(
            "mkcert -cert-file \"{cert_file}\" -key-file \"{key_file}\" \"{domain}\" \"{wildcard}\"",
            cert_file = cert_file,
            key_file = key_file,
            domain = project.domain,
            wildcard = wildcard
        ),
        None,
        None,
    )?;
    let _ = tx.send(LogLine::success("[Deploy] Certificado SSL generado ✓"));

    let _ = tx.send(LogLine::info("[Deploy] Reiniciando WSDD-Proxy-Server..."));
    runner.run_direct_sync("docker", &["restart", "WSDD-Proxy-Server"], None, None)?;
    let _ = tx.send(LogLine::success("[Deploy] WSDD-Proxy-Server reiniciado ✓"));

    Ok(())
}

fn step_update_hosts(project: &Project, tx: &LogSender) -> Result<(), InfraError> {
    let base_domains = project.php_version.base_container_domains();
    let mut domains: Vec<&str> = base_domains.iter().map(String::as_str).collect();
    domains.push(project.domain.as_str());

    hosts::update_host(Some(&domains), tx)
        .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))
}

fn step_remove_options_yml(project: &Project, tx: &LogSender) {
    let options_file = yml::options_path(
        project.php_version.dir_name(),
        project.php_version.compose_tag(),
    );
    match yml::remove_project_from_options_yml(
        &options_file,
        &project.domain,
        project.php_version.compose_tag(),
    ) {
        Ok(()) => {
            let _ = tx.send(LogLine::success("[Remove] options.yml actualizado ✓"));
        }
        Err(e) => {
            let _ = tx.send(LogLine::warn(format!(
                "[Remove] Advertencia options.yml: {e}"
            )));
        }
    }
}

fn step_remove_volume(project: &Project, runner: &PsRunner, tx: &LogSender) {
    let volume_name = format!("{}-{}", project.php_version.compose_tag(), project.domain);
    let _ = tx.send(LogLine::info(format!(
        "[Remove] Eliminando volumen '{}'...",
        volume_name
    )));
    match runner.run_direct_sync("docker", &["volume", "rm", &volume_name], None, None) {
        Ok(_) => {
            let _ = tx.send(LogLine::success(format!(
                "[Remove] Volumen '{}' eliminado ✓",
                volume_name
            )));
        }
        Err(e) => {
            let _ = tx.send(LogLine::warn(format!("[Remove] Advertencia volumen: {e}")));
        }
    }
}

fn step_remove_vhost(project: &Project, tx: &LogSender) {
    let projects = match php_projects(&project.php_version, Some(&project.name)) {
        Ok(projects) => projects,
        Err(e) => {
            let _ = tx.send(LogLine::warn(format!(
                "[Remove] Advertencia vhost.conf: {e}"
            )));
            return;
        }
    };

    match sync_active_vhost(project.php_version.dir_name(), &projects, tx) {
        Ok(()) => {
            let _ = tx.send(LogLine::success("[Remove] vhost activo regenerado ✓"));
        }
        Err(e) => {
            let _ = tx.send(LogLine::warn(format!(
                "[Remove] Advertencia vhost.conf: {e}"
            )));
        }
    }
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
    let template_path = vhost_template_path(php_dir_name);
    let active_path = active_vhost_conf_path(php_dir_name);

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
    let legacy_path = legacy_vhost_conf_path(php_dir_name);
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
