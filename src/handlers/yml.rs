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
//! Manipulación de archivos docker-compose options para proyectos WSDD.
//!
//! Equivalente a `Handlers/HandlerYml.cs` en la versión C#.
//!
//! # Estrategia de edición
//!
//! Se usa manipulación line-based (no serde_yaml) para preservar el formato
//! original del archivo, comentarios y orden de las claves — crítico para que
//! docker-compose no rompa al reordenar keys.
//!
//! # Convenciones de naming
//!
//! Dado un proyecto con dominio `"myapp.dock"` y PHP 8.3:
//! - `project_ref` = `"myapp.dock"` (el dominio — identificador único del proyecto)
//! - `php_compose_tag` = `"php83"` (tag sin puntos — ver `PhpVersion::compose_tag()`)
//! - `volume_name` = `"php83-myapp.dock"` (nombre del Docker volume externo)

use crate::config::environment::{path_config, path_to_string};
use crate::errors::InfraError;

// ─── Agregar proyecto a options.yml ──────────────────────────────────────────

/// Agrega un proyecto al archivo `options.phpXX.yml` del contenedor PHP.
///
/// Equivalente a `HandlerYml.AddOptions()` en C#.
///
/// # Operaciones
/// 1. Si `project_ref` ya está en el archivo → no hace nada (idempotente).
/// 2. Agrega `project_ref` a la línea `VIRTUAL_HOST:` (separado por coma).
/// 3. Agrega `project_ref:host-gateway` a `extra_hosts` para self-calls HTTPS.
/// 4. Agrega el volume mount `- php83-myapp.dock:/var/www/html/myapp.dock`
///    al final de la sección `volumes:` del servicio.
/// 5. Agrega la declaración global del volume con `external: true`.
///
/// # Errors
/// [`InfraError::Io`] si el archivo no se puede leer o escribir.
/// [`InfraError::UnexpectedOutput`] si el archivo no tiene la estructura esperada.
pub fn add_project_to_options_yml(
    path: &str,
    project_ref: &str,
    php_compose_tag: &str,
) -> Result<(), InfraError> {
    ensure_client_max_body_size(path)?;

    let volume_name = format!("{php_compose_tag}-{project_ref}");
    let mut lines: Vec<String> = read_lines(path)?;
    let mount_line = format!("- {volume_name}:/var/www/html/{project_ref}");
    let global_entry = format!("  {volume_name}:");
    let mut changed = false;

    // ── 2. VIRTUAL_HOST ────────────────────────────────────────────────────
    let vh_idx = lines
        .iter()
        .position(|l| l.trim_start().starts_with("VIRTUAL_HOST:"))
        .ok_or_else(|| {
            InfraError::UnexpectedOutput(
                path.to_string(),
                "no se encontró la línea VIRTUAL_HOST:".to_string(),
            )
        })?;

    // Preservar indentación original
    let indent: String = lines[vh_idx]
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect();
    let current_hosts = lines[vh_idx]
        .split_once(':')
        .map(|x| x.1)
        .unwrap_or("")
        .trim()
        .to_string();
    let mut hosts: Vec<String> = current_hosts
        .split(',')
        .map(str::trim)
        .filter(|host| !host.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    if !hosts.iter().any(|host| host == project_ref) {
        hosts.push(project_ref.to_string());
        lines[vh_idx] = format!("{indent}VIRTUAL_HOST: {}", hosts.join(","));
        changed = true;
    }

    changed |= ensure_project_extra_host(&mut lines, project_ref)?;

    // ── 3. Volume mount en la sección volumes: del servicio ───────────────
    // Buscar `    volumes:` (sección del servicio, no la global al fondo)
    let svc_vol_idx = lines
        .iter()
        .position(|l| {
            let t = l.trim_start();
            t.starts_with("volumes:") && l.starts_with("    ") // 4 espacios = nivel servicio
        })
        .ok_or_else(|| {
            InfraError::UnexpectedOutput(
                path.to_string(),
                "no se encontró la sección volumes: del servicio".to_string(),
            )
        })?;

    // Avanzar hasta el final de los entries existentes (líneas con `      -`)
    let mut insert_vol_at = svc_vol_idx + 1;
    while insert_vol_at < lines.len() && lines[insert_vol_at].trim_start().starts_with('-') {
        insert_vol_at += 1;
    }
    if !lines.iter().any(|l| l.trim() == mount_line) {
        lines.insert(insert_vol_at, format!("      {mount_line}"));
        changed = true;
    }

    // ── 4. Declaración global de volumes ──────────────────────────────────
    // Buscar `volumes:` al nivel raíz (sin indentación)
    if let Some(global_idx) = lines
        .iter()
        .position(|l| l.trim() == "volumes:" && !l.starts_with(' '))
    {
        if let Some(existing_idx) = lines.iter().position(|l| l.trim_end() == global_entry) {
            if existing_idx + 1 >= lines.len() || lines[existing_idx + 1].trim() != "external: true"
            {
                lines.insert(existing_idx + 1, "    external: true".to_string());
                changed = true;
            }
        } else {
            let mut insert_at = global_idx + 1;
            while insert_at < lines.len()
                && (lines[insert_at].starts_with("  ") || lines[insert_at].trim().is_empty())
            {
                insert_at += 1;
            }
            lines.insert(insert_at, "    external: true".to_string());
            lines.insert(insert_at, global_entry);
            changed = true;
        }
    } else {
        // No existe la sección global — agregarla al final
        lines.push(String::new());
        lines.push("volumes:".to_string());
        lines.push(global_entry);
        lines.push("    external: true".to_string());
        changed = true;
    }

    if changed {
        write_lines(path, &lines)?;
    }

    Ok(())
}

/// Ensures existing PHP options files expose nginx-proxy upload limit metadata.
///
/// The migration is line-based so existing `VIRTUAL_HOST` entries for already
/// registered projects remain untouched.
pub fn ensure_client_max_body_size(path: &str) -> Result<(), InfraError> {
    let mut lines: Vec<String> = read_lines(path)?;
    if lines
        .iter()
        .any(|line| line.trim_start().starts_with("CLIENT_MAX_BODY_SIZE:"))
    {
        return Ok(());
    }

    let virtual_port_idx = lines
        .iter()
        .position(|line| line.trim_start().starts_with("VIRTUAL_PORT:"))
        .ok_or_else(|| {
            InfraError::UnexpectedOutput(
                path.to_string(),
                "no se encontró la línea VIRTUAL_PORT:".to_string(),
            )
        })?;
    let indent: String = lines[virtual_port_idx]
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect();
    lines.insert(
        virtual_port_idx + 1,
        format!("{indent}CLIENT_MAX_BODY_SIZE: 16m"),
    );
    write_lines(path, &lines)
}

/// Ensures PHP-Admin's live options entries are present without touching
/// project-specific state.
pub fn ensure_php_admin_options_yml(
    path: &str,
    php_admin_domain: &str,
    php_compose_tag: &str,
) -> Result<(), InfraError> {
    ensure_client_max_body_size(path)?;

    let mut lines = read_lines(path)?;
    let mut changed = false;
    changed |= ensure_virtual_host(&mut lines, path, php_admin_domain)?;
    changed |= ensure_service_volume(
        &mut lines,
        &format!("- ../../state/php-admin/{php_compose_tag}:/var/lib/wsdd-php-admin"),
    )?;
    changed |= ensure_service_volume(
        &mut lines,
        &format!("- ../../log/php-admin/{php_compose_tag}:/var/log/wsdd-php-admin"),
    )?;

    if changed {
        write_lines(path, &lines)?;
    }

    Ok(())
}

/// Ensures the PHP-Admin Apache proxy block exists in the WSDD-managed section.
pub fn ensure_php_admin_vhost_conf(path: &str, php_admin_domain: &str) -> Result<(), InfraError> {
    let mut lines = read_lines(path)?;
    if lines
        .iter()
        .any(|line| line.trim() == format!("ServerName {php_admin_domain}"))
    {
        return Ok(());
    }

    let insert_at = lines
        .iter()
        .rposition(|line| line.trim() == "### DO NOT TOUCH ###")
        .ok_or_else(|| {
            InfraError::UnexpectedOutput(
                path.to_string(),
                "no se encontró el marcador DO NOT TOUCH final".to_string(),
            )
        })?;

    let block = [
        "<VirtualHost *:80>".to_string(),
        format!("    ServerName {php_admin_domain}"),
        "    Protocols h2 h2c http/1.1".to_string(),
        "    ProxyPass / http://127.0.0.1:8989/".to_string(),
        "    ProxyPassReverse / http://127.0.0.1:8989/".to_string(),
        "    ProxyPreserveHost On".to_string(),
        "    <Proxy *>".to_string(),
        "        Allow from all".to_string(),
        "    </Proxy>".to_string(),
        format!("    ErrorLog /var/log/wsdd-php-admin/{php_admin_domain}_error.log"),
        format!("    CustomLog /var/log/wsdd-php-admin/{php_admin_domain}_access.log combined"),
        "</VirtualHost>".to_string(),
    ];

    for (offset, line) in block.into_iter().enumerate() {
        lines.insert(insert_at + offset, line);
    }
    write_lines(path, &lines)
}

// ─── Eliminar proyecto de options.yml ────────────────────────────────────────

/// Elimina un proyecto del archivo `options.phpXX.yml` del contenedor PHP.
///
/// # Operaciones (inversas a `add_project_to_options_yml`)
/// 1. Si `project_ref` no está en el archivo → no hace nada (idempotente).
/// 2. Elimina `project_ref` de la línea `VIRTUAL_HOST:`.
/// 3. Elimina `project_ref:host-gateway` de `extra_hosts`.
/// 4. Elimina el volume mount del proyecto.
/// 5. Elimina la declaración global del volume (entrada + `external: true`).
///
/// # Errors
/// [`InfraError::Io`] si el archivo no se puede leer o escribir.
pub fn remove_project_from_options_yml(
    path: &str,
    project_ref: &str,
    php_compose_tag: &str,
) -> Result<(), InfraError> {
    let volume_name = format!("{php_compose_tag}-{project_ref}");
    let mut lines: Vec<String> = read_lines(path)?;
    let mount_line = format!("- {volume_name}:/var/www/html/{project_ref}");

    // ── 2. VIRTUAL_HOST — eliminar project_ref de la lista ────────────────
    if let Some(idx) = lines
        .iter()
        .position(|l| l.trim_start().starts_with("VIRTUAL_HOST:"))
    {
        let indent: String = lines[idx]
            .chars()
            .take_while(|c| c.is_whitespace())
            .collect();
        let hosts_str = lines[idx]
            .split_once(':')
            .map(|x| x.1)
            .unwrap_or("")
            .trim()
            .to_string();
        let updated: Vec<&str> = hosts_str
            .split(',')
            .map(|h| h.trim())
            .filter(|h| *h != project_ref)
            .collect();
        lines[idx] = format!("{indent}VIRTUAL_HOST: {}", updated.join(","));
    }

    remove_project_extra_host(&mut lines, project_ref);

    // ── 3. Volume mount del proyecto ──────────────────────────────────────
    lines.retain(|l| l.trim() != mount_line);

    // ── 4. Declaración global del volume (2 líneas: nombre + external) ────
    // Buscar `  volume_name:` (2 espacios = nivel global volumes)
    let global_entry = format!("  {volume_name}:");
    if let Some(idx) = lines.iter().position(|l| l.trim_end() == global_entry) {
        // Eliminar también la línea `    external: true` siguiente si existe
        if idx + 1 < lines.len() && lines[idx + 1].trim() == "external: true" {
            lines.remove(idx + 1);
        }
        lines.remove(idx);
    }

    remove_empty_global_volumes(&mut lines, path)?;
    write_lines(path, &lines)
}

fn remove_empty_global_volumes(lines: &mut Vec<String>, path: &str) -> Result<(), InfraError> {
    let document: serde_yaml::Value = serde_yaml::from_str(&lines.join("\n"))
        .map_err(|e| InfraError::UnexpectedOutput(path.into(), e.to_string()))?;
    match document.get("volumes") {
        Some(serde_yaml::Value::Null) => {}
        None | Some(serde_yaml::Value::Mapping(_)) => return Ok(()),
        Some(_) => {
            return Err(InfraError::UnexpectedOutput(
                path.into(),
                "La seccion global volumes debe ser un mapping".into(),
            ));
        }
    }
    // Remove only the empty root key; service mounts and comments stay intact.
    if let Some(index) = lines.iter().position(|line| {
        !line.starts_with(char::is_whitespace)
            && line
                .split_once(':')
                .is_some_and(|(key, _)| key == "volumes")
    }) {
        lines.remove(index);
    }
    Ok(())
}

// ─── Rutas de archivos ───────────────────────────────────────────────────────

/// Retorna la ruta completa al archivo `options.phpXX.yml` para la versión PHP dada.
///
/// # Ejemplo
/// ```
/// let path = options_path("php8.3", "php83");
/// // → r"C:\WSDD-Environment\Docker-Structure\bin\php8.3\options.php83.yml"
/// ```
pub fn options_path(php_dir_name: &str, compose_tag: &str) -> String {
    path_to_string(path_config().options_yml(php_dir_name, compose_tag))
}

// ─── Helpers privados ─────────────────────────────────────────────────────────

fn read_lines(path: &str) -> Result<Vec<String>, InfraError> {
    std::fs::read_to_string(path)
        .map(|s| s.lines().map(|l| l.to_string()).collect())
        .map_err(InfraError::Io)
}

fn write_lines(path: &str, lines: &[String]) -> Result<(), InfraError> {
    let content = lines.join("\n");
    std::fs::write(path, content).map_err(InfraError::Io)
}

fn ensure_virtual_host(lines: &mut [String], path: &str, host: &str) -> Result<bool, InfraError> {
    let vh_idx = lines
        .iter()
        .position(|l| l.trim_start().starts_with("VIRTUAL_HOST:"))
        .ok_or_else(|| {
            InfraError::UnexpectedOutput(
                path.to_string(),
                "no se encontró la línea VIRTUAL_HOST:".to_string(),
            )
        })?;
    let indent: String = lines[vh_idx]
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect();
    let current_hosts = lines[vh_idx]
        .split_once(':')
        .map(|x| x.1)
        .unwrap_or("")
        .trim()
        .to_string();
    let mut hosts: Vec<String> = current_hosts
        .split(',')
        .map(str::trim)
        .filter(|host| !host.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    if hosts.iter().any(|candidate| candidate == host) {
        return Ok(false);
    }

    hosts.push(host.to_string());
    lines[vh_idx] = format!("{indent}VIRTUAL_HOST: {}", hosts.join(","));
    Ok(true)
}

fn ensure_service_volume(lines: &mut Vec<String>, mount_line: &str) -> Result<bool, InfraError> {
    if lines.iter().any(|line| line.trim() == mount_line) {
        return Ok(false);
    }

    let svc_vol_idx = lines
        .iter()
        .position(|l| {
            let t = l.trim_start();
            t.starts_with("volumes:") && l.starts_with("    ")
        })
        .ok_or_else(|| {
            InfraError::UnexpectedOutput(
                mount_line.to_string(),
                "no se encontró la sección volumes: del servicio".to_string(),
            )
        })?;
    let mut insert_at = svc_vol_idx + 1;
    while insert_at < lines.len() && lines[insert_at].trim_start().starts_with('-') {
        insert_at += 1;
    }
    lines.insert(insert_at, format!("      {mount_line}"));
    Ok(true)
}

fn ensure_project_extra_host(
    lines: &mut Vec<String>,
    project_ref: &str,
) -> Result<bool, InfraError> {
    let host_entry = format!("\"{project_ref}:host-gateway\"");
    if lines
        .iter()
        .any(|line| line.trim() == format!("- {host_entry}"))
    {
        return Ok(false);
    }

    let service_idx = lines
        .iter()
        .position(|line| line.starts_with("  ") && line.trim_end().ends_with(':'))
        .ok_or_else(|| {
            InfraError::UnexpectedOutput(
                project_ref.to_string(),
                "no se encontró el servicio webserver en options.yml".to_string(),
            )
        })?;

    if let Some(extra_hosts_idx) = lines
        .iter()
        .enumerate()
        .skip(service_idx + 1)
        .take_while(|(_, line)| line.starts_with("    ") || line.trim().is_empty())
        .find_map(|(idx, line)| {
            (line.trim_start().starts_with("extra_hosts:") && line.starts_with("    "))
                .then_some(idx)
        })
    {
        let mut insert_at = extra_hosts_idx + 1;
        while insert_at < lines.len() && lines[insert_at].trim_start().starts_with('-') {
            insert_at += 1;
        }
        lines.insert(insert_at, format!("      - {host_entry}"));
        return Ok(true);
    }

    let volumes_idx = lines
        .iter()
        .enumerate()
        .skip(service_idx + 1)
        .take_while(|(_, line)| line.starts_with("    ") || line.trim().is_empty())
        .find_map(|(idx, line)| {
            (line.trim_start().starts_with("volumes:") && line.starts_with("    ")).then_some(idx)
        })
        .ok_or_else(|| {
            InfraError::UnexpectedOutput(
                project_ref.to_string(),
                "no se encontró la sección volumes: del servicio".to_string(),
            )
        })?;

    lines.insert(volumes_idx, format!("      - {host_entry}"));
    lines.insert(volumes_idx, "    extra_hosts:".to_string());
    Ok(true)
}

fn remove_project_extra_host(lines: &mut Vec<String>, project_ref: &str) {
    let host_entry = format!("- \"{project_ref}:host-gateway\"");
    lines.retain(|line| line.trim() != host_entry);

    let mut idx = 0;
    while idx < lines.len() {
        if lines[idx].trim_start().starts_with("extra_hosts:") && lines[idx].starts_with("    ") {
            let next_has_entry =
                idx + 1 < lines.len() && lines[idx + 1].trim_start().starts_with('-');
            if !next_has_entry {
                lines.remove(idx);
                continue;
            }
        }
        idx += 1;
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_yml() -> String {
        "\
services:
  webserver83:
    environment:
      VIRTUAL_HOST: php83.wsdd.dock,cron83.wsdd.dock,wm83.wsdd.dock
      VIRTUAL_PORT: 80
    volumes:
      - ./dev:/var/www/html/dev
      - ./vhost:/etc/apache2/sites-enabled"
            .to_string()
    }

    #[test]
    fn removing_last_project_keeps_valid_root_volumes_for_every_php_version() {
        for version in crate::models::project::PhpVersion::all() {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("options.yml");
            let tag = version.compose_tag();
            let original = format!(
                "services:\n  webserver{}:\n    environment:\n      VIRTUAL_HOST: base.dock\n      VIRTUAL_PORT: 80\n    volumes:\n      - ./dev:/var/www/html/dev\n      - ../../state/php-admin/{tag}:/var/lib/wsdd-php-admin\nnetworks:\n  retained:\n    external: true\n",
                version.host_suffix()
            );
            std::fs::write(&path, original).unwrap();
            let p = path.to_str().unwrap();
            for domain in ["first.dock", "second.dock"] {
                add_project_to_options_yml(p, domain, tag).unwrap();
            }
            remove_project_from_options_yml(p, "first.dock", tag).unwrap();
            let remaining: serde_yaml::Value =
                serde_yaml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
            assert!(remaining["volumes"].is_mapping());
            assert_eq!(remaining["volumes"].as_mapping().unwrap().len(), 1);
            remove_project_from_options_yml(p, "second.dock", tag).unwrap();
            let after = std::fs::read_to_string(&path).unwrap();
            let document: serde_yaml::Value = serde_yaml::from_str(&after).unwrap();
            assert!(document.get("volumes").is_none());
            assert!(document["networks"]["retained"].is_mapping());
            assert!(after.contains(&format!(
                "../../state/php-admin/{tag}:/var/lib/wsdd-php-admin"
            )));
            remove_project_from_options_yml(p, "second.dock", tag).unwrap();
            assert_eq!(
                std::fs::read_to_string(&path).unwrap().trim_end(),
                after.trim_end()
            );
            add_project_to_options_yml(p, "third.dock", tag).unwrap();
            let document: serde_yaml::Value =
                serde_yaml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
            assert_eq!(document["volumes"].as_mapping().unwrap().len(), 1);
        }
    }

    #[test]
    fn remove_repairs_preexisting_empty_global_volumes_without_touching_other_sections() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("options.yml");
        std::fs::write(
            &path,
            format!(
                "{}\n\nvolumes:\n# keep this comment\nnetworks:\n  retained: {{}}\n",
                sample_yml()
            ),
        )
        .unwrap();
        remove_project_from_options_yml(path.to_str().unwrap(), "absent.dock", "php83").unwrap();
        let after = std::fs::read_to_string(path).unwrap();
        let doc: serde_yaml::Value = serde_yaml::from_str(&after).unwrap();
        assert!(doc.get("volumes").is_none());
        assert!(doc["networks"].is_mapping());
        assert!(after.contains("# keep this comment"));
    }

    #[test]
    fn invalid_global_volumes_are_rejected_without_writing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("options.yml");
        let original = format!("{}\nvolumes: [invalid]\n", sample_yml());
        std::fs::write(&path, &original).unwrap();
        assert!(
            remove_project_from_options_yml(path.to_str().unwrap(), "absent.dock", "php83")
                .is_err()
        );
        assert_eq!(std::fs::read_to_string(path).unwrap(), original);
    }

    #[test]
    fn add_project_appends_virtual_host_and_volume() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_options.yml");
        std::fs::write(&path, sample_yml()).unwrap();

        add_project_to_options_yml(path.to_str().unwrap(), "myapp.dock", "php83").unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(
            content.contains("myapp.dock"),
            "VIRTUAL_HOST debe incluir myapp.dock"
        );
        assert!(
            content.contains("php83-myapp.dock:/var/www/html/myapp.dock"),
            "volume mount"
        );
        assert!(
            content.contains("      - \"myapp.dock:host-gateway\""),
            "extra_hosts debe permitir self-calls HTTPS al vhost local"
        );
        assert!(content.contains("volumes:"), "sección global volumes");
        assert!(content.contains("external: true"), "volume externo");
    }

    #[test]
    fn add_project_is_idempotent() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_options_idem.yml");
        std::fs::write(&path, sample_yml()).unwrap();

        add_project_to_options_yml(path.to_str().unwrap(), "myapp.dock", "php83").unwrap();
        let after_first = std::fs::read_to_string(&path).unwrap();

        add_project_to_options_yml(path.to_str().unwrap(), "myapp.dock", "php83").unwrap();
        let after_second = std::fs::read_to_string(&path).unwrap();

        assert_eq!(
            after_first, after_second,
            "segunda llamada no debe cambiar nada"
        );
    }

    #[test]
    fn remove_project_cleans_all_entries() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_options_remove.yml");
        std::fs::write(&path, sample_yml()).unwrap();

        add_project_to_options_yml(path.to_str().unwrap(), "myapp.dock", "php83").unwrap();
        remove_project_from_options_yml(path.to_str().unwrap(), "myapp.dock", "php83").unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(
            !content.contains("myapp.dock"),
            "no debe quedar rastro del proyecto"
        );
        assert!(
            !content.contains("php83-myapp.dock"),
            "volume mount eliminado"
        );
        assert!(
            !content.contains("external: true\n    external: true"),
            "no debe duplicar external"
        );
    }

    #[test]
    fn add_project_repairs_partial_state() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_options_partial.yml");
        std::fs::write(
            &path,
            "\
services:
  webserver83:
    environment:
      VIRTUAL_HOST: php83.wsdd.dock,cron83.wsdd.dock,wm83.wsdd.dock,myapp.dock
      VIRTUAL_PORT: 80
    volumes:
      - ./dev:/var/www/html/dev
      - ./vhost:/etc/apache2/sites-enabled
volumes:
",
        )
        .unwrap();

        add_project_to_options_yml(path.to_str().unwrap(), "myapp.dock", "php83").unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let virtual_host_line = content
            .lines()
            .find(|line| line.trim_start().starts_with("VIRTUAL_HOST:"))
            .unwrap();
        assert_eq!(
            virtual_host_line
                .split_once(':')
                .map(|x| x.1)
                .unwrap_or("")
                .split(',')
                .filter(|host| host.trim() == "myapp.dock")
                .count(),
            1,
            "el dominio no debe duplicarse en VIRTUAL_HOST"
        );
        assert_eq!(
            content
                .lines()
                .filter(|line| line.trim() == "- \"myapp.dock:host-gateway\"")
                .count(),
            1,
            "el extra_host del proyecto debe existir una sola vez"
        );
        assert_eq!(
            content
                .lines()
                .filter(|line| line.trim() == "- php83-myapp.dock:/var/www/html/myapp.dock")
                .count(),
            1,
            "el mount del proyecto debe existir una sola vez"
        );
        assert_eq!(
            content
                .lines()
                .filter(|line| line.trim() == "php83-myapp.dock:")
                .count(),
            1,
            "la declaracion global del volume debe existir una sola vez"
        );
        assert!(content.contains("php83-myapp.dock:/var/www/html/myapp.dock"));
        assert!(content.contains("  php83-myapp.dock:"));
        assert!(content.contains("    external: true"));
    }

    #[test]
    fn ensure_client_max_body_size_preserves_existing_virtual_hosts() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_options_client_size.yml");
        std::fs::write(
            &path,
            "\
services:
  webserver83:
    environment:
      VIRTUAL_HOST: php83.wsdd.dock,cron83.wsdd.dock,wm83.wsdd.dock,existing.dock
      VIRTUAL_PORT: 80
    volumes:
      - ./dev:/var/www/html/dev
",
        )
        .unwrap();

        ensure_client_max_body_size(path.to_str().unwrap()).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains(
            "VIRTUAL_HOST: php83.wsdd.dock,cron83.wsdd.dock,wm83.wsdd.dock,existing.dock"
        ));
        assert!(content.contains("      CLIENT_MAX_BODY_SIZE: 16m"));
    }

    #[test]
    fn remove_project_drops_external_line_for_removed_volume() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_options_remove_external.yml");
        std::fs::write(
            &path,
            "\
services:
  webserver83:
    environment:
      VIRTUAL_HOST: php83.wsdd.dock,myapp.dock
    volumes:
      - php83-myapp.dock:/var/www/html/myapp.dock
volumes:
  php83-myapp.dock:
    external: true
",
        )
        .unwrap();

        remove_project_from_options_yml(path.to_str().unwrap(), "myapp.dock", "php83").unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(!content.contains("php83-myapp.dock:/var/www/html/myapp.dock"));
        assert!(!content.contains("  php83-myapp.dock:"));
        assert!(!content.contains("    external: true"));
    }

    #[test]
    fn add_project_preserves_existing_extra_hosts() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_options_extra_hosts.yml");
        std::fs::write(
            &path,
            "\
services:
  webserver83:
    environment:
      VIRTUAL_HOST: php83.wsdd.dock
      VIRTUAL_PORT: 80
    extra_hosts:
      - \"existing.dock:host-gateway\"
    volumes:
      - ./dev:/var/www/html/dev
",
        )
        .unwrap();

        add_project_to_options_yml(path.to_str().unwrap(), "myapp.dock", "php83").unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("      - \"existing.dock:host-gateway\""));
        assert!(content.contains("      - \"myapp.dock:host-gateway\""));
        assert_eq!(
            content
                .lines()
                .filter(|line| line.trim() == "- \"myapp.dock:host-gateway\"")
                .count(),
            1
        );
    }

    #[test]
    fn add_project_preserves_existing_project_state() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_options_existing_project_state.yml");
        std::fs::write(
            &path,
            "\
services:
  webserver83:
    environment:
      VIRTUAL_HOST: php83.wsdd.dock,cron83.wsdd.dock,wm83.wsdd.dock,existing.dock
      VIRTUAL_PORT: 80
      CLIENT_MAX_BODY_SIZE: 16m
    extra_hosts:
      - \"existing.dock:host-gateway\"
    volumes:
      - ./dev:/var/www/html/dev
      - php83-existing.dock:/var/www/html/existing.dock
      - ./vhost:/etc/apache2/sites-enabled
volumes:
  php83-existing.dock:
    external: true
",
        )
        .unwrap();

        add_project_to_options_yml(path.to_str().unwrap(), "newapp.dock", "php83").unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(
            content.contains("existing.dock"),
            "existing project domain must be preserved"
        );
        assert!(content.contains("      - \"existing.dock:host-gateway\""));
        assert!(content.contains("      - php83-existing.dock:/var/www/html/existing.dock"));
        assert!(content.contains("  php83-existing.dock:"));
        assert!(content.contains("      - \"newapp.dock:host-gateway\""));
        assert!(content.contains("      - php83-newapp.dock:/var/www/html/newapp.dock"));
        assert!(content.contains("  php83-newapp.dock:"));
    }

    #[test]
    fn remove_project_keeps_other_extra_hosts() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_options_remove_extra_hosts.yml");
        std::fs::write(
            &path,
            "\
services:
  webserver83:
    environment:
      VIRTUAL_HOST: php83.wsdd.dock,myapp.dock,other.dock
    extra_hosts:
      - \"myapp.dock:host-gateway\"
      - \"other.dock:host-gateway\"
    volumes:
      - php83-myapp.dock:/var/www/html/myapp.dock
volumes:
  php83-myapp.dock:
    external: true
",
        )
        .unwrap();

        remove_project_from_options_yml(path.to_str().unwrap(), "myapp.dock", "php83").unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(!content.contains("\"myapp.dock:host-gateway\""));
        assert!(content.contains("      - \"other.dock:host-gateway\""));
    }

    #[test]
    fn ensure_php_admin_options_adds_domain_and_persistent_mounts() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_options_php_admin.yml");
        std::fs::write(&path, sample_yml()).unwrap();

        ensure_php_admin_options_yml(path.to_str().unwrap(), "phpadmin83.wsdd.dock", "php83")
            .unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains(
            "VIRTUAL_HOST: php83.wsdd.dock,cron83.wsdd.dock,wm83.wsdd.dock,phpadmin83.wsdd.dock"
        ));
        assert!(content.contains("../../state/php-admin/php83:/var/lib/wsdd-php-admin"));
        assert!(content.contains("../../log/php-admin/php83:/var/log/wsdd-php-admin"));
    }

    #[test]
    fn ensure_php_admin_options_is_idempotent() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_options_php_admin_idem.yml");
        std::fs::write(&path, sample_yml()).unwrap();

        ensure_php_admin_options_yml(path.to_str().unwrap(), "phpadmin83.wsdd.dock", "php83")
            .unwrap();
        let after_first = std::fs::read_to_string(&path).unwrap();
        ensure_php_admin_options_yml(path.to_str().unwrap(), "phpadmin83.wsdd.dock", "php83")
            .unwrap();
        let after_second = std::fs::read_to_string(&path).unwrap();

        assert_eq!(after_first, after_second);
    }

    #[test]
    fn remove_project_preserves_php_admin_virtual_host() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_options_remove_keeps_php_admin.yml");
        std::fs::write(&path, sample_yml()).unwrap();

        ensure_php_admin_options_yml(path.to_str().unwrap(), "phpadmin83.wsdd.dock", "php83")
            .unwrap();
        add_project_to_options_yml(path.to_str().unwrap(), "myapp.dock", "php83").unwrap();
        remove_project_from_options_yml(path.to_str().unwrap(), "myapp.dock", "php83").unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("phpadmin83.wsdd.dock"));
        assert!(!content.contains("myapp.dock"));
    }

    #[test]
    fn ensure_php_admin_vhost_adds_proxy_inside_managed_block() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_vhost_php_admin.conf");
        std::fs::write(
            &path,
            "\
### DO NOT TOUCH ###
<VirtualHost *:80>
    ServerName wm83.wsdd.dock
</VirtualHost>
### DO NOT TOUCH ###
### PERSONAL PROJECTS ###
### PERSONAL PROJECTS ###
",
        )
        .unwrap();

        ensure_php_admin_vhost_conf(path.to_str().unwrap(), "phpadmin83.wsdd.dock").unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("ServerName phpadmin83.wsdd.dock"));
        assert!(content.contains("ProxyPass / http://127.0.0.1:8989/"));
        assert!(content.contains("/var/log/wsdd-php-admin/phpadmin83.wsdd.dock_access.log"));
        assert!(
            content.find("ServerName phpadmin83.wsdd.dock").unwrap()
                < content.find("### PERSONAL PROJECTS ###").unwrap()
        );
    }
}
