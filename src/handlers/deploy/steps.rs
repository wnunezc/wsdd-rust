use crate::config::environment::{env_config, path_config, path_to_string};
use crate::errors::InfraError;
use crate::handlers::hosts;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::ps_script::{PsRunner, ScriptRunner};
use crate::handlers::yml;
use crate::models::project::Project;

use super::paths;

pub(super) fn update_options_yml(project: &Project, tx: &LogSender) -> Result<(), InfraError> {
    let options_file = paths::options_yml_path(project);
    yml::add_project_to_options_yml(
        &options_file,
        &project.domain,
        project.php_version.compose_tag(),
    )?;
    let _ = tx.send(LogLine::success("[Deploy] options.yml actualizado ✓"));
    Ok(())
}

pub(super) fn setup_ssl(
    project: &Project,
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    let paths = path_config();
    std::fs::create_dir_all(paths.ssl_dir()).map_err(InfraError::Io)?;

    let cert_file = path_to_string(paths.ssl_cert_file(&project.domain));
    let key_file = path_to_string(paths.ssl_key_file(&project.domain));
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
    runner.run_direct_sync(
        env_config().docker_exe(),
        &["restart", "WSDD-Proxy-Server"],
        None,
        None,
    )?;
    let _ = tx.send(LogLine::success("[Deploy] WSDD-Proxy-Server reiniciado ✓"));

    Ok(())
}

pub(super) fn update_hosts(project: &Project, tx: &LogSender) -> Result<(), InfraError> {
    let base_domains = project.php_version.base_container_domains();
    let mut domains: Vec<&str> = base_domains.iter().map(String::as_str).collect();
    domains.push(project.domain.as_str());

    hosts::update_host(Some(&domains), tx)
        .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))
}

pub(super) fn remove_options_yml(project: &Project, tx: &LogSender) -> Result<(), InfraError> {
    let options_file = paths::options_yml_path(project);
    yml::remove_project_from_options_yml(
        &options_file,
        &project.domain,
        project.php_version.compose_tag(),
    )?;
    let _ = tx.send(LogLine::success("[Remove] options.yml actualizado ✓"));
    Ok(())
}

pub(super) fn remove_hosts(project: &Project, tx: &LogSender) -> Result<(), InfraError> {
    hosts::remove_domains(&[project.domain.as_str()], tx)
        .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?;
    let _ = tx.send(LogLine::success("[Remove] Archivo hosts actualizado ✓"));
    Ok(())
}

pub(super) fn remove_ssl(project: &Project, tx: &LogSender) -> Result<(), InfraError> {
    for path in paths::ssl_file_paths(project) {
        if !path.exists() {
            continue;
        }
        std::fs::remove_file(&path).map_err(InfraError::Io)?;
        let _ = tx.send(LogLine::success(format!(
            "[Remove] Certificado removido ✓ {}",
            path.display()
        )));
    }
    Ok(())
}
