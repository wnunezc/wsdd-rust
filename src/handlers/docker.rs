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
//! Gestión de Docker Desktop — detección, configuración y operaciones de contenedores.
//!
//! Equivalente a `Handlers/HandlerDocker.cs` en la versión C#.
//!
//! # Evaluación individual de scripts
//!
//! Cada función `probe_*` ejecuta exactamente un script PS1 y retorna
//! un resultado tipado. Se pueden llamar en aislamiento para validar
//! el entorno sin ejecutar el flujo completo de instalación:
//!
//! ```rust,ignore
//! let runner = PsRunner::new();
//! let installed = docker::probe_installed(&runner).await?;
//! let configured = docker::probe_configured(&runner).await?;
//! let running = docker::probe_running(&runner).await?;
//! ```

use crate::errors::InfraError;
use crate::handlers::ps_script::{OutputSender, ProcOutput, PsRunner, ScriptRunner, run_docker};

/// Nombre de la red Docker compartida entre todos los contenedores WSDD.
pub const WSDD_NETWORK: &str = "wsdd-network";

/// Proyecto Docker Compose de WSDD.
pub const WSDD_PROJECT: &str = "wsdd-projects";

// ─── Sondas de estado (probe_*) ───────────────────────────────────────────────
// Cada función ejecuta exactamente un script y retorna bool o Result.
// Permiten evaluación individual sin ejecutar el flujo completo.

/// Verifica si Docker Desktop está instalado en el sistema.
///
/// Ejecuta: `dd-isinstalled.ps1`
/// Keyword de éxito: `"Installed"`
///
/// # Errors
/// [`InfraError::ScriptFailed`] si PowerShell no puede ejecutarse.
pub async fn probe_installed(runner: &PsRunner) -> Result<bool, InfraError> {
    let runner = runner.clone();
    let result = tokio::task::spawn_blocking(move || {
        runner.run_script_sync("dd-isinstalled.ps1", None, None)
    })
    .await
    .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))??;

    Ok(result.contains("Installed"))
}

/// Verifica si Docker Desktop está configurado con los flags necesarios para WSDD.
///
/// Ejecuta: `dd-issettingup.ps1`
/// Keyword de éxito: `"Updated"` (indica que settings.json ya tiene los flags)
///
/// # Flags verificados en settings.json:
/// - `exposeDockerAPIOnTCP2375 = true`
/// - `updateHostsFile = true`
/// - `runWinServiceInWslMode = true`
/// - `useResourceSaver = false`
///
/// # Errors
/// [`InfraError::ScriptFailed`] si PowerShell no puede ejecutarse.
pub async fn probe_configured(runner: &PsRunner) -> Result<bool, InfraError> {
    let runner = runner.clone();
    let result = tokio::task::spawn_blocking(move || {
        runner.run_script_sync("dd-issettingup.ps1", None, None)
    })
    .await
    .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))??;

    Ok(result.contains("Updated"))
}

/// Verifica si Docker Desktop está corriendo y responde a comandos.
///
/// Ejecuta: `dd-isrunning.ps1`
/// Keyword de éxito: `"Running"`
///
/// # Errors
/// [`InfraError::ScriptFailed`] si PowerShell no puede ejecutarse.
pub async fn probe_running(runner: &PsRunner) -> Result<bool, InfraError> {
    let runner = runner.clone();
    let result = tokio::task::spawn_blocking(move || {
        runner.run_script_sync("dd-isrunning.ps1", None, None)
    })
    .await
    .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))??;

    Ok(result.contains("Running"))
}

// ─── Operaciones de control ───────────────────────────────────────────────────

/// Aplica la configuración necesaria en `settings.json` de Docker Desktop
/// y lo reinicia completamente.
///
/// Ejecuta: `dd-setting.ps1`
/// Keyword de éxito: `"Continue"`
///
/// Este script: para el servicio, parchea settings.json, inicia el servicio,
/// lanza Docker Desktop y espera que el pipe y la API estén disponibles.
///
/// # Errors
/// [`InfraError::ScriptFailed`] si PowerShell falla.
/// [`InfraError::UnexpectedOutput`] si Docker no arrancó correctamente.
pub async fn apply_settings(
    runner: &PsRunner,
    tx: Option<OutputSender>,
) -> Result<(), InfraError> {
    let runner = runner.clone();
    let result = tokio::task::spawn_blocking(move || {
        runner.run_script_sync("dd-setting.ps1", None, tx.as_ref())
    })
    .await
    .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))??;

    if result.contains("Continue") {
        Ok(())
    } else {
        Err(InfraError::UnexpectedOutput(
            "dd-setting.ps1".to_string(),
            result.text.clone(),
        ))
    }
}

/// Inicia Docker Desktop y espera a que esté listo.
///
/// Ejecuta: `dd-start.ps1`
///
/// # Errors
/// [`InfraError::ScriptFailed`] si el script falla.
pub async fn start(runner: &PsRunner, tx: Option<OutputSender>) -> Result<(), InfraError> {
    let runner = runner.clone();
    tokio::task::spawn_blocking(move || {
        runner.run_script_sync("dd-start.ps1", None, tx.as_ref())
    })
    .await
    .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?
    .map(|_| ())
}

/// Detiene Docker Desktop y todos sus procesos.
///
/// Ejecuta: `dd-stop.ps1`
///
/// # Errors
/// [`InfraError::ScriptFailed`] si el script falla.
pub async fn stop(runner: &PsRunner, tx: Option<OutputSender>) -> Result<(), InfraError> {
    let runner = runner.clone();
    tokio::task::spawn_blocking(move || {
        runner.run_script_sync("dd-stop.ps1", None, tx.as_ref())
    })
    .await
    .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?
    .map(|_| ())
}

/// Apaga WSL completamente (`wsl --shutdown`).
///
/// Al reabrir Docker Desktop, WSL se reiniciará automáticamente.
///
/// # Errors
/// [`InfraError::ScriptFailed`] si el script falla.
pub async fn stop_wsl(runner: &PsRunner, tx: Option<OutputSender>) -> Result<(), InfraError> {
    let runner = runner.clone();
    tokio::task::spawn_blocking(move || {
        runner.run_script_sync("wsl-shutdown.ps1", None, tx.as_ref())
    })
    .await
    .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?
    .map(|_| ())
}

/// Inicia un contenedor Docker específico.
///
/// # Errors
/// [`InfraError::DockerUnreachable`] si docker no responde o el contenedor no existe.
pub async fn start_container(
    container_name: &str,
    tx: Option<OutputSender>,
) -> Result<(), InfraError> {
    let args = vec!["start".to_string(), container_name.to_string()];
    let out = run_docker(args, tx).await?;
    if out.success {
        Ok(())
    } else {
        Err(InfraError::DockerUnreachable(format!(
            "No se pudo iniciar {container_name}: {}",
            out.text
        )))
    }
}

/// Detiene un contenedor Docker específico.
///
/// # Errors
/// [`InfraError::DockerUnreachable`] si docker no responde o el contenedor no existe.
pub async fn stop_container(
    container_name: &str,
    tx: Option<OutputSender>,
) -> Result<(), InfraError> {
    let args = vec!["stop".to_string(), container_name.to_string()];
    let out = run_docker(args, tx).await?;
    if out.success {
        Ok(())
    } else {
        Err(InfraError::DockerUnreachable(format!(
            "No se pudo detener {container_name}: {}",
            out.text
        )))
    }
}

/// Reinicia un contenedor Docker específico.
///
/// # Errors
/// [`InfraError::DockerUnreachable`] si docker no responde.
pub async fn restart_container(
    container_name: &str,
    tx: Option<OutputSender>,
) -> Result<(), InfraError> {
    let args = vec!["restart".to_string(), container_name.to_string()];
    let out = run_docker(args, tx).await?;
    if out.success {
        Ok(())
    } else {
        Err(InfraError::DockerUnreachable(format!(
            "No se pudo reiniciar {container_name}: {}",
            out.text
        )))
    }
}

// ─── Infraestructura de red y volúmenes ───────────────────────────────────────

/// Verifica si la red `wsdd-network` existe en Docker.
///
/// # Errors
/// [`InfraError::DockerUnreachable`] si docker no responde.
pub async fn network_exists() -> Result<bool, InfraError> {
    let out = run_docker(vec!["network".to_string(), "ls".to_string()], None).await?;
    if out.text.contains("Error") {
        return Err(InfraError::DockerUnreachable(out.text.clone()));
    }
    Ok(out.contains(WSDD_NETWORK))
}

/// Crea la red bridge `wsdd-network` si no existe.
///
/// # Errors
/// [`InfraError::ScriptFailed`] si la creación falla.
pub async fn ensure_network(
    runner: &PsRunner,
    tx: Option<OutputSender>,
) -> Result<bool, InfraError> {
    if network_exists().await? {
        return Ok(true); // ya existe
    }
    let cmd = format!("docker network create --driver bridge {WSDD_NETWORK}");
    let runner = runner.clone();
    let result = tokio::task::spawn_blocking(move || {
        runner.run_ps_sync(&cmd, None, tx.as_ref())
    })
    .await
    .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))??;

    // Verificar que se creó correctamente
    network_exists().await.inspect(|&exists| {
        if !exists {
            tracing::warn!(output = %result.text, "La red no se creó correctamente");
        }
    })
}

/// Verifica si el volumen `pma-code` (phpMyAdmin) existe.
///
/// # Errors
/// [`InfraError::DockerUnreachable`] si docker no responde.
pub async fn pma_volume_exists() -> Result<bool, InfraError> {
    let out = run_docker(vec!["volume".to_string(), "ls".to_string()], None).await?;
    if out.text.contains("Error") {
        return Err(InfraError::DockerUnreachable(out.text.clone()));
    }
    Ok(out.contains("pma-code"))
}

/// Verifica si los contenedores base de WSDD están desplegados.
///
/// Verifica la presencia de: `WSDD-Proxy-Server`, `WSDD-MySql-Server`, `phpMyAdmin-Server`
///
/// # Errors
/// [`InfraError::DockerUnreachable`] si docker no responde.
pub async fn base_containers_exist() -> Result<bool, InfraError> {
    let out = run_docker(vec!["ps".to_string(), "-a".to_string()], None).await?;
    if out.text.contains("Error") {
        return Err(InfraError::DockerUnreachable(out.text.clone()));
    }
    Ok(out.contains("WSDD-Proxy-Server")
        && out.contains("WSDD-MySql-Server")
        && out.contains("phpMyAdmin-Server"))
}

/// Verifica si existe un contenedor PHP de una versión específica.
///
/// # Errors
/// [`InfraError::DockerUnreachable`] si docker no responde.
pub async fn php_container_exists(php_container_tag: &str) -> Result<bool, InfraError> {
    let out = run_docker(vec!["ps".to_string(), "-a".to_string()], None).await?;
    if out.text.contains("Error") {
        return Err(InfraError::DockerUnreachable(out.text.clone()));
    }
    Ok(out.contains(php_container_tag))
}

// ─── Variables de entorno ─────────────────────────────────────────────────────

/// Establece `DOCKER_HOST=tcp://localhost:2375` para usuario actual y sistema.
///
/// Necesario para que los comandos `docker` funcionen sin Docker Desktop
/// como proceso principal del usuario.
///
/// # Errors
/// [`InfraError::Io`] si PowerShell falla.
pub async fn set_docker_host_env(runner: &PsRunner) -> Result<(), InfraError> {
    let cmds = [
        r#"[Environment]::SetEnvironmentVariable("DOCKER_HOST", "tcp://localhost:2375", "User")"#,
        r#"[Environment]::SetEnvironmentVariable("DOCKER_HOST", "tcp://localhost:2375", "Machine")"#,
    ];

    let runner_ref = runner.clone();
    for cmd in &cmds {
        let cmd = cmd.to_string();
        let r = runner_ref.clone();
        tokio::task::spawn_blocking(move || r.run_ps_sync(&cmd, None, None))
            .await
            .map_err(|e| {
                InfraError::Io(std::io::Error::other(e.to_string()))
            })??;
    }
    Ok(())
}

// ─── Listado de contenedores ──────────────────────────────────────────────────

/// Obtiene la lista de contenedores WSDD activos.
///
/// Parsea el output de `docker ps -a` en formato pipe-separado.
/// Equivalente a `GetDockerContainersAsync()` en C#.
///
/// # Errors
/// [`InfraError::ScriptFailed`] si el script falla.
pub async fn list_containers(
    runner: &PsRunner,
    tx: Option<OutputSender>,
) -> Result<Vec<ContainerInfo>, InfraError> {
    // Comando docker directo — sin PS1 generado en runtime.
    // Los templates {{.ID}}, {{.Names}}, etc. se pasan como arg directo a docker.exe
    // sin pasar por PowerShell, por lo que no hay interpolación de llaves.
    // (Patrón dd-grepcontainerview.ps1 eliminado: ver wsdd-rust.md § Scripts PS1 eliminados)
    let args = vec![
        "ps".to_string(),
        "-a".to_string(),
        "--format".to_string(),
        "{{.ID}}|{{.Names}}|{{.Image}}|{{.Ports}}|{{.Status}}".to_string(),
        "--filter".to_string(),
        "name=WSDD-".to_string(),
    ];
    let result = run_docker(args, tx).await?;
    let containers = parse_container_list(&result.text, runner.clone()).await;
    Ok(containers)
}

async fn parse_container_list(output: &str, runner: PsRunner) -> Vec<ContainerInfo> {
    let mut containers = Vec::new();

    for line in output.lines() {
        if line.contains("error") || line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 5 {
            continue;
        }

        let name = parts[1].to_string();
        let r = runner.clone();
        let cmd = format!("docker exec {name} printenv VIRTUAL_HOST");
        let urls_raw = tokio::task::spawn_blocking(move || r.run_ps_sync(&cmd, None, None))
            .await
            .ok()
            .and_then(|r| r.ok())
            .map(|o| o.text)
            .unwrap_or_default();

        let urls: Vec<String> = urls_raw
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        containers.push(ContainerInfo {
            id: parts[0].to_string(),
            name,
            image: parts[2].to_string(),
            ports: parts[3].to_string(),
            status: parts[4].to_string(),
            urls,
        });
    }

    containers
}

// ─── Tipos de datos ───────────────────────────────────────────────────────────

/// Información de un contenedor Docker activo.
///
/// Equivalente a `DockerContainer` en C#.
#[derive(Debug, Clone)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub ports: String,
    pub status: String,
    /// URLs del contenedor obtenidas de la env var `VIRTUAL_HOST`.
    pub urls: Vec<String>,
}

impl ContainerInfo {
    /// Retorna `true` si el contenedor está en estado Running/Up/Started.
    pub fn is_running(&self) -> bool {
        let status_lower = self.status.to_lowercase();
        status_lower.contains("up")
            || status_lower.contains("running")
            || status_lower.contains("started")
    }
}

/// Resultado de la verificación de requisitos de Docker.
#[derive(Debug, Clone)]
pub struct RequirementStatus {
    pub installed: bool,
    pub configured: bool,
    pub running: bool,
}

impl RequirementStatus {
    /// Retorna `true` si Docker está completamente listo para WSDD.
    pub fn is_ready(&self) -> bool {
        self.installed && self.configured && self.running
    }
}

/// Resultado del despliegue del entorno base.
#[derive(Debug, Clone)]
pub struct DeployStatus {
    pub network_ok: bool,
    pub volume_ok: bool,
    pub containers_ok: bool,
}

impl DeployStatus {
    pub fn is_complete(&self) -> bool {
        self.network_ok && self.volume_ok && self.containers_ok
    }
}

// ─── Fix de permisos MySQL ────────────────────────────────────────────────────

/// Aplica permisos FullControl en el directorio de datos MySQL.
///
/// Ejecuta: `dd-fixmysqlpermission.ps1`
///
/// Nota: el script C# original usaba `C:\ProgramData\WSDD-Environment\` (bug).
/// Esta implementación usa la ruta correcta: `C:\WSDD-Environment\`.
///
/// # Errors
/// [`InfraError::ScriptFailed`] si el script falla.
pub async fn fix_mysql_permissions(
    runner: &PsRunner,
    tx: Option<OutputSender>,
) -> Result<ProcOutput, InfraError> {
    let runner = runner.clone();
    tokio::task::spawn_blocking(move || {
        runner.run_script_sync("dd-fixmysqlpermission.ps1", None, tx.as_ref())
    })
    .await
    .map_err(|e| InfraError::Io(std::io::Error::other(e.to_string())))?
}

// ─── Wrappers síncronos para background threads ──────────────────────────────
//
// Las funciones async de este módulo usan `run_docker` → `spawn_blocking` y
// necesitan un runtime de tokio. Estos wrappers usan `run_direct_sync`
// directamente para poder llamarse desde `std::thread::spawn` sin runtime.

/// Lista los contenedores WSDD de forma síncrona.
///
/// Equivalente a `list_containers` pero usable desde threads no-async.
pub fn list_containers_sync(runner: &PsRunner) -> Result<Vec<ContainerInfo>, InfraError> {
    let args = [
        "ps",
        "-a",
        "--format",
        "{{.ID}}|{{.Names}}|{{.Image}}|{{.Ports}}|{{.Status}}",
        "--filter",
        "name=WSDD-",
    ];
    let result = runner.run_direct_sync("docker", &args, None, None)?;
    Ok(parse_container_list_sync(&result.text))
}

fn parse_container_list_sync(output: &str) -> Vec<ContainerInfo> {
    output
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.to_lowercase().contains("error"))
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() < 5 {
                return None;
            }
            Some(ContainerInfo {
                id: parts[0].to_string(),
                name: parts[1].to_string(),
                image: parts[2].to_string(),
                ports: parts[3].to_string(),
                status: parts[4].to_string(),
                urls: vec![],
            })
        })
        .collect()
}

/// Inicia un contenedor por nombre de forma síncrona.
pub fn start_container_sync(runner: &PsRunner, name: &str) -> Result<(), InfraError> {
    let out = runner.run_direct_sync("docker", &["start", name], None, None)?;
    if out.success {
        Ok(())
    } else {
        Err(InfraError::DockerUnreachable(format!(
            "No se pudo iniciar {name}: {}",
            out.text
        )))
    }
}

/// Detiene un contenedor por nombre de forma síncrona.
pub fn stop_container_sync(runner: &PsRunner, name: &str) -> Result<(), InfraError> {
    let out = runner.run_direct_sync("docker", &["stop", name], None, None)?;
    if out.success {
        Ok(())
    } else {
        Err(InfraError::DockerUnreachable(format!(
            "No se pudo detener {name}: {}",
            out.text
        )))
    }
}

/// Reinicia un contenedor por nombre de forma síncrona.
pub fn restart_container_sync(runner: &PsRunner, name: &str) -> Result<(), InfraError> {
    let out = runner.run_direct_sync("docker", &["restart", name], None, None)?;
    if out.success {
        Ok(())
    } else {
        Err(InfraError::DockerUnreachable(format!(
            "No se pudo reiniciar {name}: {}",
            out.text
        )))
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_info_is_running() {
        let c = ContainerInfo {
            id: "abc".to_string(),
            name: "WSDD-Proxy-Server".to_string(),
            image: "nginx".to_string(),
            ports: "80/tcp".to_string(),
            status: "Up 2 hours".to_string(),
            urls: vec![],
        };
        assert!(c.is_running());
    }

    #[test]
    fn container_info_not_running() {
        let c = ContainerInfo {
            id: "abc".to_string(),
            name: "WSDD-MySql-Server".to_string(),
            image: "mysql".to_string(),
            ports: String::new(),
            status: "Exited (0) 1 hour ago".to_string(),
            urls: vec![],
        };
        assert!(!c.is_running());
    }

    #[test]
    fn requirement_status_is_ready() {
        let s = RequirementStatus {
            installed: true,
            configured: true,
            running: true,
        };
        assert!(s.is_ready());
    }

    #[test]
    fn requirement_status_not_ready_if_not_running() {
        let s = RequirementStatus {
            installed: true,
            configured: true,
            running: false,
        };
        assert!(!s.is_ready());
    }
}
