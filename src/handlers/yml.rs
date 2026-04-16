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
/// 3. Agrega el volume mount `- php83-myapp.dock:/var/www/html/myapp.dock`
///    al final de la sección `volumes:` del servicio.
/// 4. Agrega la declaración global del volume con `external: true`.
///
/// # Errors
/// [`InfraError::Io`] si el archivo no se puede leer o escribir.
/// [`InfraError::UnexpectedOutput`] si el archivo no tiene la estructura esperada.
pub fn add_project_to_options_yml(
    path: &str,
    project_ref: &str,
    php_compose_tag: &str,
) -> Result<(), InfraError> {
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

// ─── Eliminar proyecto de options.yml ────────────────────────────────────────

/// Elimina un proyecto del archivo `options.phpXX.yml` del contenedor PHP.
///
/// # Operaciones (inversas a `add_project_to_options_yml`)
/// 1. Si `project_ref` no está en el archivo → no hace nada (idempotente).
/// 2. Elimina `project_ref` de la línea `VIRTUAL_HOST:`.
/// 3. Elimina el volume mount del proyecto.
/// 4. Elimina la declaración global del volume (entrada + `external: true`).
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

    write_lines(path, &lines)
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
}
