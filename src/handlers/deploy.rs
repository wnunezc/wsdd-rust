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
//! Despliegue y eliminación de proyectos WSDD.
//!
//! Equivalente a `HandlerProject.DeployNewProjectAsync` y el flujo de remove en C#.
//! Separado por SRP: este módulo orquesta las operaciones de infra (volumen, yml,
//! docker-compose, vhost, SSL, hosts) y la persistencia del proyecto.
//!
//! # Funciones públicas
//!
//! - [`deploy_project`]: despliega un proyecto completo (guarda en disco + infra).
//! - [`remove_project`]: elimina un proyecto completo (infra + borra de disco).
//!
//! Todas las funciones son sincrónicas — llamar desde `std::thread::spawn`.

use std::path::Path;

use crate::errors::InfraError;
use crate::handlers::docker::WSDD_PROJECT;
use crate::handlers::docker_deploy;
use crate::handlers::docker_deploy::{make_docker_progress_bridge, make_log_bridge};
use crate::handlers::hosts;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::project as project_handler;
use crate::handlers::ps_script::{PsRunner, ScriptRunner};
use crate::handlers::setting::AppSettings;
use crate::handlers::yml;
use crate::models::project::Project;

// ─── Rutas del entorno ────────────────────────────────────────────────────────

const SSL_DIR: &str = r"C:\WSDD-Environment\Docker-Structure\ssl";

fn php_bin_dir(php_dir_name: &str) -> String {
    format!(r"C:\WSDD-Environment\Docker-Structure\bin\{}", php_dir_name)
}

fn vhost_conf_path(php_dir_name: &str) -> String {
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

// ─── API pública ──────────────────────────────────────────────────────────────

/// Despliega un proyecto WSDD completo.
///
/// # Flujo
/// 1. Guarda el proyecto en disco (`project::save`).
/// 2. Crea el volumen Docker para el `WorkPath`.
/// 3. Agrega el dominio al `options.phpXX.yml`.
/// 4. Reconstruye el contenedor PHP (stop → rm → create --build → up -d).
/// 5. Inserta el bloque vhost en `vhost.conf`.
/// 6. Si `ssl=true`: genera certificado con mkcert y reinicia el proxy.
/// 7. Actualiza el archivo `hosts`.
///
/// Retorna `Ok(())` si todos los pasos completan sin error.
/// Si falla un paso, los anteriores ya están aplicados — se puede reintentar.
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

    // 1. Persistir proyecto
    project_handler::save(project)?;
    let _ = tx.send(LogLine::success("[Deploy] Proyecto guardado ✓"));

    // 2. Volumen Docker
    step_create_volume(project, runner, tx)?;

    // 3. options.yml
    step_update_options_yml(project, tx)?;

    // 4. Sincronizar recursos administrados de la versión PHP
    docker_deploy::sync_php_version_resources_sync(settings, &project.php_version)?;
    let _ = tx.send(LogLine::success(
        "[Deploy] Recursos gestionados de PHP/Webmin sincronizados ✓",
    ));

    // 5. Reconstruir contenedor PHP
    step_rebuild_php_container(project, runner, tx)?;

    // 6. vhost.conf
    step_update_vhost(project, tx)?;

    // 7. SSL (opcional)
    if project.ssl {
        step_setup_ssl(project, runner, tx)?;
    }

    // 8. Hosts
    step_update_hosts(project, tx)?;

    let _ = tx.send(LogLine::success(format!(
        "[Deploy] '{}' desplegado correctamente ✓",
        project.name
    )));
    Ok(())
}

/// Elimina un proyecto WSDD completo.
///
/// # Flujo
/// 1. Elimina el dominio de `options.phpXX.yml` (antes de borrar el proyecto).
/// 2. Reconstruye el contenedor PHP sin el proyecto.
/// 3. Elimina el volumen Docker.
/// 4. Elimina el bloque vhost de `vhost.conf`.
/// 5. Borra el proyecto del disco (`project::delete`).
///
/// Los dominios NO se eliminan de `hosts` (limitación conocida).
pub fn remove_project(
    project: &Project,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let _ = tx.send(LogLine::info(format!(
        "[Remove] Eliminando proyecto '{}'...",
        project.name
    )));

    // 1. Quitar de options.yml (antes de perder los datos del proyecto)
    step_remove_options_yml(project, tx);

    // 2. Reconstruir contenedor PHP (aplica el yml actualizado)
    step_rebuild_php_container(project, runner, tx)?;

    // 3. Eliminar volumen
    step_remove_volume(project, runner, tx);

    // 4. Limpiar vhost.conf
    step_remove_vhost(project, tx);

    // 5. Borrar de disco
    project_handler::delete(&project.name)?;

    let _ = tx.send(LogLine::success(format!(
        "[Remove] '{}' eliminado ✓",
        project.name
    )));
    Ok(())
}

// ─── Pasos de Deploy ──────────────────────────────────────────────────────────

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

/// Stop → rm → create --build → up -d para el contenedor PHP del proyecto.
///
/// El stop y rm son best-effort (ignorados si el contenedor no existe).
fn step_rebuild_php_container(
    project: &Project,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let container_name = format!("WSDD-Web-Server-{}", project.php_version.container_tag());
    let php_dir_name = project.php_version.dir_name();
    let compose_tag = project.php_version.compose_tag();

    let bin_dir_str = php_bin_dir(php_dir_name);
    let bin_dir = Path::new(&bin_dir_str);
    let webserver_yml = webserver_yml_path(php_dir_name, compose_tag);
    let options_yml = yml::options_path(php_dir_name, compose_tag);

    // Stop (best-effort)
    let _ = tx.send(LogLine::info(format!(
        "[Deploy] Deteniendo {}...",
        container_name
    )));
    let _ = runner.run_direct_sync("docker", &["stop", &container_name], None, None);
    let _ = runner.run_direct_sync("docker", &["rm", &container_name], None, None);

    // docker-compose create --build
    let _ = tx.send(LogLine::info(
        "[Deploy] Construyendo contenedor PHP (puede tardar)...",
    ));
    let bridge = make_docker_progress_bridge(tx);
    runner.run_ps_sync(
        &format!(
            "docker-compose -p {WSDD_PROJECT} -f \"{webserver_yml}\" -f \"{options_yml}\" create --build"
        ),
        Some(bin_dir),
        Some(&bridge),
    )?;

    // docker-compose up -d
    let _ = tx.send(LogLine::info("[Deploy] Iniciando contenedor PHP..."));
    let bridge2 = make_docker_progress_bridge(tx);
    runner.run_ps_sync(
        &format!(
            "docker-compose -p {WSDD_PROJECT} -f \"{webserver_yml}\" -f \"{options_yml}\" up -d"
        ),
        Some(bin_dir),
        Some(&bridge2),
    )?;

    let _ = tx.send(LogLine::success(format!(
        "[Deploy] {} reconstruido ✓",
        container_name
    )));
    Ok(())
}

/// Inserta el bloque vhost del proyecto en `vhost.conf`.
///
/// Usa `tpl.vhost.conf` como plantilla. Idempotente: omite si el dominio ya existe.
fn step_update_vhost(project: &Project, tx: &LogSender) -> Result<(), InfraError> {
    let php_dir_name = project.php_version.dir_name();
    let template_path = vhost_template_path(php_dir_name);
    let vhost_path = vhost_conf_path(php_dir_name);

    let template = std::fs::read_to_string(&template_path).map_err(InfraError::Io)?;

    // Reemplazar placeholders
    let protocol = if project.ssl {
        "Protocols h2 h2c http/1.1"
    } else {
        ""
    };
    let block = template
        .replace("{CustomUrl}", &project.domain)
        .replace("{EntryPoint}", project.entry_point.as_path())
        .replace("{PROTOCOL}", protocol);

    // Leer vhost.conf existente (o crear vacío si no existe)
    let vhost_content = std::fs::read_to_string(&vhost_path).unwrap_or_default();

    // Idempotencia: si el dominio ya tiene entrada, no duplicar
    if vhost_content.contains(&format!("ServerName {}", project.domain)) {
        let _ = tx.send(LogLine::info(
            "[Deploy] vhost.conf: entrada ya existe (idempotente)",
        ));
        return Ok(());
    }

    // Insertar tras el marcador ### PERSONAL PROJECTS ###
    const MARKER: &str = "### PERSONAL PROJECTS ###";
    let new_content = if let Some(pos) = vhost_content.find(MARKER) {
        let end_of_line = vhost_content[pos..]
            .find('\n')
            .map(|n| pos + n + 1)
            .unwrap_or(pos + MARKER.len());
        let mut result = String::from(&vhost_content[..end_of_line]);
        result.push('\n');
        result.push_str(&block);
        result.push('\n');
        result.push_str(&vhost_content[end_of_line..]);
        result
    } else {
        // Sin marcador: agregar al final
        format!("{}\n\n{}\n", vhost_content, block)
    };

    std::fs::write(&vhost_path, new_content).map_err(InfraError::Io)?;
    let _ = tx.send(LogLine::success("[Deploy] vhost.conf actualizado ✓"));
    Ok(())
}

/// Genera el certificado SSL con mkcert y reinicia el proxy.
fn step_setup_ssl(project: &Project, runner: &PsRunner, tx: &LogSender) -> Result<(), InfraError> {
    std::fs::create_dir_all(SSL_DIR).map_err(InfraError::Io)?;

    let cert_file = format!("{}\\{}.crt", SSL_DIR, project.domain);
    let key_file = format!("{}\\{}.key", SSL_DIR, project.domain);
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

// ─── Pasos de Remove ──────────────────────────────────────────────────────────

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
    let vhost_path = vhost_conf_path(project.php_version.dir_name());
    let content = match std::fs::read_to_string(&vhost_path) {
        Ok(c) => c,
        Err(_) => return, // Si no existe, no hay nada que limpiar
    };
    let cleaned = strip_vhost_block(&content, &project.domain);
    match std::fs::write(&vhost_path, cleaned) {
        Ok(()) => {
            let _ = tx.send(LogLine::success("[Remove] vhost.conf limpiado ✓"));
        }
        Err(e) => {
            let _ = tx.send(LogLine::warn(format!(
                "[Remove] Advertencia vhost.conf: {e}"
            )));
        }
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Elimina el bloque `<VirtualHost>` correspondiente al dominio de `content`.
///
/// Parser manual línea a línea — no requiere crate regex.
/// Idempotente: si el bloque no existe, retorna el contenido sin modificar.
fn strip_vhost_block(content: &str, domain: &str) -> String {
    let server_name_marker = format!("ServerName {}", domain);
    let mut result: Vec<&str> = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if trimmed.eq_ignore_ascii_case("<VirtualHost *:80>") {
            // Buscar si este bloque contiene nuestro ServerName
            let mut block_end = i;
            let mut is_target = false;

            for (j, line_j) in lines.iter().enumerate().skip(i + 1) {
                if line_j.trim().eq_ignore_ascii_case("</VirtualHost>") {
                    block_end = j;
                    break;
                }
                if line_j.contains(&server_name_marker) {
                    is_target = true;
                }
            }

            if is_target && block_end > i {
                // Saltar el bloque completo (incluyendo posible línea vacía posterior)
                i = block_end + 1;
                // Saltar línea vacía de separación si la hay
                if i < lines.len() && lines[i].trim().is_empty() {
                    i += 1;
                }
                continue;
            }
        }

        result.push(lines[i]);
        i += 1;
    }

    result.join("\n")
}
