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
//! Despliegue del entorno Docker base para WSDD.
//!
//! Equivalente a `HandlerDocker.DeployEnvironment()` + verificación de requisitos
//! en la versión C#. Separado de `docker.rs` por SRP — este módulo es responsable
//! exclusivamente del proceso de inicialización del entorno, no de las operaciones
//! cotidianas con contenedores.
//!
//! # Funciones principales
//!
//! - [`process_requirements_sync`]: verifica e inicializa Docker Desktop (Loader).
//! - [`deploy_environment_sync`]: despliega red, volúmenes y contenedores base.
//!
//! Todas las funciones son sincrónicas — deben llamarse desde `spawn_blocking`.

use std::collections::HashMap;
use std::io::Write;

use crate::errors::InfraError;
use crate::handlers::docker::{WSDD_NETWORK, WSDD_PROJECT};
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::ps_script::{OutputSender, PsRunner, ScriptRunner};

// ─── Resultado del check de requisitos de Docker ──────────────────────────────

/// Resultado del check de requisitos de Docker.
#[derive(Debug, Clone, PartialEq)]
pub enum DockerRequirementOutcome {
    /// Docker está instalado, configurado y en ejecución.
    Ready,
    /// Docker no está instalado — error bloqueante, la app no puede continuar.
    NotInstalled,
    /// Docker está instalado pero no pudo arrancar en el tiempo esperado.
    StartupFailed,
}

// ─── Requirements (llamado desde Loader) ─────────────────────────────────────

/// Verifica e inicializa Docker Desktop de forma sincrónica.
///
/// Ejecuta los scripts probe en orden y aplica configuración si es necesario.
/// Debe llamarse desde un hilo separado (no en el render loop de egui).
///
/// # Flujo
/// 1. `dd-isinstalled.ps1` — si falla: retorna `NotInstalled`
/// 2. `dd-issettingup.ps1` + `dd-isrunning.ps1`
///    - configured && running → `Ready`
///    - configured && !running → `dd-start.ps1`
///    - !configured → `dd-setting.ps1`
/// 3. Retorna `Ready`
pub fn process_requirements_sync(runner: &PsRunner, tx: &LogSender) -> DockerRequirementOutcome {
    let _ = tx.send(LogLine::info("Verificando Docker Desktop..."));

    // ── 1. ¿Instalado? ────────────────────────────────────────────────────
    let installed = runner
        .run_script_sync("dd-isinstalled.ps1", None, None)
        .map(|o| o.contains("Installed"))
        .unwrap_or(false);

    if !installed {
        let _ = tx.send(LogLine::error("✗ Docker Desktop no está instalado."));
        let _ = tx.send(LogLine::error(
            "  Descárgalo desde https://www.docker.com/products/docker-desktop",
        ));
        return DockerRequirementOutcome::NotInstalled;
    }
    let _ = tx.send(LogLine::success("✓ Docker Desktop está instalado"));

    // ── 2. ¿Configurado y corriendo? ──────────────────────────────────────
    let configured = runner
        .run_script_sync("dd-issettingup.ps1", None, None)
        .map(|o| o.contains("Updated"))
        .unwrap_or(false);

    let running = runner
        .run_script_sync("dd-isrunning.ps1", None, None)
        .map(|o| o.contains("Running"))
        .unwrap_or(false);

    if configured && running {
        let _ = tx.send(LogLine::success(
            "✓ Docker Desktop está configurado y en ejecución",
        ));
        return DockerRequirementOutcome::Ready;
    }

    // ── 3a. Settings correctos pero Docker no corre → solo iniciar ────────
    if configured {
        let _ = tx.send(LogLine::warn(
            "Docker Desktop no está en ejecución — iniciando...",
        ));
        return run_script_outcome(runner, tx, "dd-start.ps1", "iniciar");
    }

    // ── 3b. Settings no aplicados → configurar y reiniciar ────────────────
    let _ = tx.send(LogLine::warn(
        "Aplicando configuración de Docker Desktop...",
    ));
    let _ = tx.send(LogLine::info(
        "  (Docker se reiniciará — puede tardar hasta 2 minutos)",
    ));
    run_script_outcome(runner, tx, "dd-setting.ps1", "configurar")
}

/// Ejecuta un script PS1 y mapea el resultado a [`DockerRequirementOutcome`].
///
/// Retorna `Ready` si el script emite `"Continue"`, `StartupFailed` en cualquier
/// otro caso (timeout, error de pipe, fallo de PowerShell, etc.).
fn run_script_outcome(
    runner: &PsRunner,
    tx: &LogSender,
    script: &str,
    action: &str,
) -> DockerRequirementOutcome {
    let out_tx = make_log_bridge(tx);
    match runner.run_script_sync(script, None, Some(&out_tx)) {
        Ok(o) if o.contains("Continue") => {
            let _ = tx.send(LogLine::success("✓ Docker Desktop listo para WSDD"));
            DockerRequirementOutcome::Ready
        }
        Ok(o) => {
            tracing::warn!(script, output = %o.text, "script no emitió Continue");
            let _ = tx.send(LogLine::error(format!(
                "✗ Docker Desktop no pudo {action} en el tiempo esperado"
            )));
            let _ = tx.send(LogLine::error(
                "  Inicia Docker Desktop manualmente y vuelve a abrir WSDD",
            ));
            DockerRequirementOutcome::StartupFailed
        }
        Err(e) => {
            tracing::error!(script, error = %e, "falló la ejecución del script");
            let _ = tx.send(LogLine::error(format!("✗ Error al {action} Docker: {e}")));
            DockerRequirementOutcome::StartupFailed
        }
    }
}

// ─── Deploy Environment (inicialización del entorno base) ────────────────────
//
// Equivalente a `HandlerDocker.DeployEnvironment()` en la versión C#.
// Todas las funciones son sincrónicas — deben llamarse desde spawn_blocking.

/// Inicializa el entorno Docker mínimo para WSDD (sincrónico).
///
/// # Flujo
/// 1. `DOCKER_HOST=tcp://localhost:2375` (User + Machine)
/// 2. Red `wsdd-network`
/// 3. Volumen `pma-code`
/// 4. Contenedores base via docker-compose (create --build + up -d)
/// 5. Mostrar contenedores activos
///
/// Retorna `Ok(())` solo si todos los pasos completan sin error.
pub fn deploy_environment_sync(runner: &PsRunner, tx: &LogSender) -> Result<(), InfraError> {
    set_docker_host_env_sync(runner, tx)?;
    ensure_network_sync(runner, tx)?;
    create_pma_volume_sync(runner, tx)?;
    deploy_base_containers_sync(runner, tx)?;
    show_running_containers_sync(runner, tx);
    Ok(())
}

/// Aplica permisos FullControl en el directorio de datos MySQL.
///
/// Ejecuta: `dd-fixmysqlpermission.ps1`
/// Usa la ruta correcta `C:\WSDD-Environment\` (el C# original tenía un bug con ProgramData).
///
/// # Errors
/// [`InfraError::ScriptFailed`] si el script falla.
pub fn fix_mysql_permissions_sync(
    runner: &PsRunner,
    tx: Option<&OutputSender>,
) -> Result<(), InfraError> {
    runner
        .run_script_sync("dd-fixmysqlpermission.ps1", None, tx)
        .map(|_| ())
}

// ─── Helpers de deploy (privados) ────────────────────────────────────────────

/// Establece `DOCKER_HOST=tcp://localhost:2375` en dos scopes Windows.
///
/// - `"User"` → HKCU: variable disponible solo para el usuario actual.
/// - `"Machine"` → HKLM: variable disponible para todos los usuarios y procesos de sistema.
///   Ambos son necesarios — no son duplicados.
fn set_docker_host_env_sync(runner: &PsRunner, tx: &LogSender) -> Result<(), InfraError> {
    let _ = tx.send(LogLine::info(
        "Configurando DOCKER_HOST=tcp://localhost:2375...",
    ));
    let cmds = [
        r#"[Environment]::SetEnvironmentVariable("DOCKER_HOST", "tcp://localhost:2375", "User")"#,
        r#"[Environment]::SetEnvironmentVariable("DOCKER_HOST", "tcp://localhost:2375", "Machine")"#,
    ];
    for cmd in cmds {
        runner.run_ps_sync(cmd, None, None)?;
    }
    let _ = tx.send(LogLine::success(
        "✓ DOCKER_HOST configurado (User + Machine)",
    ));
    Ok(())
}

/// Verifica si la red wsdd-network existe (sincrónico).
fn check_network_sync(runner: &PsRunner) -> Result<bool, InfraError> {
    let out = runner.run_direct_sync("docker", &["network", "ls"], None, None)?;
    if out.text.contains("Error") {
        return Err(InfraError::DockerUnreachable(out.text));
    }
    Ok(out.contains(WSDD_NETWORK))
}

/// Crea la red `wsdd-network` si no existe (sincrónico).
fn ensure_network_sync(runner: &PsRunner, tx: &LogSender) -> Result<(), InfraError> {
    let _ = tx.send(LogLine::info("Verificando red Docker wsdd-network..."));
    if check_network_sync(runner)? {
        let _ = tx.send(LogLine::success("✓ Red wsdd-network ya existe"));
        return Ok(());
    }
    let _ = tx.send(LogLine::warn("Red wsdd-network no encontrada — creando..."));
    let bridge = make_log_bridge(tx);
    runner.run_direct_sync(
        "docker",
        &["network", "create", "--driver", "bridge", WSDD_NETWORK],
        None,
        Some(&bridge),
    )?;
    if check_network_sync(runner)? {
        let _ = tx.send(LogLine::success("✓ Red wsdd-network creada correctamente"));
        Ok(())
    } else {
        let _ = tx.send(LogLine::error("✗ No se pudo crear la red wsdd-network"));
        Err(InfraError::UnexpectedOutput(
            "docker network create wsdd-network".to_string(),
            "red no encontrada tras la creación".to_string(),
        ))
    }
}

/// Verifica si el volumen pma-code existe (sincrónico).
fn check_pma_volume_sync(runner: &PsRunner) -> Result<bool, InfraError> {
    let out = runner.run_direct_sync("docker", &["volume", "ls"], None, None)?;
    if out.text.contains("Error") {
        return Err(InfraError::DockerUnreachable(out.text));
    }
    Ok(out.contains("pma-code"))
}

/// Crea el volumen `pma-code` si no existe (sincrónico).
fn create_pma_volume_sync(runner: &PsRunner, tx: &LogSender) -> Result<(), InfraError> {
    let _ = tx.send(LogLine::info("Verificando volumen pma-code..."));
    if check_pma_volume_sync(runner)? {
        let _ = tx.send(LogLine::success("✓ Volumen pma-code ya existe"));
        return Ok(());
    }
    let _ = tx.send(LogLine::warn("Volumen pma-code no encontrado — creando..."));
    let device = r"C:\WSDD-Environment\Docker-Structure\bin\pma\app";
    let device_opt = format!("device={device}");
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
            "pma-code",
        ],
        None,
        Some(&bridge),
    )?;
    if check_pma_volume_sync(runner)? {
        let _ = tx.send(LogLine::success("✓ Volumen pma-code creado correctamente"));
        Ok(())
    } else {
        let _ = tx.send(LogLine::error("✗ No se pudo crear el volumen pma-code"));
        Err(InfraError::UnexpectedOutput(
            "docker volume create pma-code".to_string(),
            "volumen no encontrado tras la creación".to_string(),
        ))
    }
}

/// Verifica si los tres contenedores base WSDD existen (sincrónico).
fn check_base_containers_sync(runner: &PsRunner) -> Result<bool, InfraError> {
    let out = runner.run_direct_sync("docker", &["ps", "-a"], None, None)?;
    if out.text.contains("Error") {
        return Err(InfraError::DockerUnreachable(out.text));
    }
    Ok(out.contains("WSDD-Proxy-Server")
        && out.contains("WSDD-MySql-Server")
        && out.contains("phpMyAdmin-Server"))
}

/// Despliega los contenedores base WSDD via docker-compose (sincrónico).
///
/// # Flujo
/// 1. `docker-compose create --build` — construye imágenes y crea contenedores
/// 2. Espera 15s (settle de layers internos)
/// 3. Verifica que los contenedores existan
/// 4. `docker-compose up -d` — inicia los contenedores
/// 5. Espera 15s (inicialización de servicios)
fn deploy_base_containers_sync(runner: &PsRunner, tx: &LogSender) -> Result<(), InfraError> {
    if check_base_containers_sync(runner)? {
        let _ = tx.send(LogLine::success("✓ Contenedores WSDD ya están desplegados"));
        return Ok(());
    }

    let init_yml = r"C:\WSDD-Environment\Docker-Structure\init.yml";
    let docker_dir = std::path::Path::new(r"C:\WSDD-Environment\Docker-Structure");
    let log_path = deploy_log_path();

    // Paso 1: create --build
    let _ = tx.send(LogLine::info(
        "Construyendo contenedores WSDD — puede tardar varios minutos en la primera ejecución...",
    ));
    let _ = tx.send(LogLine::info(format!(
        "  (output detallado en: {})",
        log_path.display()
    )));
    write_deploy_log_header("docker-compose create --build");
    let bridge = make_docker_progress_bridge(tx);
    runner.run_ps_sync(
        &format!("docker-compose -p {WSDD_PROJECT} -f \"{init_yml}\" create --build"),
        Some(docker_dir),
        Some(&bridge),
    )?;
    let _ = tx.send(LogLine::success(
        "✓ Imágenes construidas y contenedores creados",
    ));

    // Paso 2: esperar 15s
    let _ = tx.send(LogLine::info(
        "Esperando que Docker procese los contenedores creados (15s)...",
    ));
    std::thread::sleep(std::time::Duration::from_secs(15));

    // Paso 3: verificar
    if !check_base_containers_sync(runner)? {
        let _ = tx.send(LogLine::error(
            "✗ Los contenedores no se crearon correctamente",
        ));
        return Err(InfraError::UnexpectedOutput(
            "docker-compose create --build".to_string(),
            "contenedores no encontrados tras la creación".to_string(),
        ));
    }

    // Paso 4: up -d
    let _ = tx.send(LogLine::info("Iniciando contenedores WSDD..."));
    write_deploy_log_header("docker-compose up -d");
    let bridge2 = make_docker_progress_bridge(tx);
    runner.run_ps_sync(
        &format!("docker-compose -p {WSDD_PROJECT} -f \"{init_yml}\" up -d"),
        Some(docker_dir),
        Some(&bridge2),
    )?;

    // Paso 5: esperar 15s
    let _ = tx.send(LogLine::info(
        "Esperando que los servicios estén disponibles (15s)...",
    ));
    std::thread::sleep(std::time::Duration::from_secs(15));

    let _ = tx.send(LogLine::success(
        "✓ Contenedores WSDD desplegados correctamente",
    ));
    Ok(())
}

/// Muestra los contenedores WSDD activos en el terminal (feedback visual post-deploy).
///
/// No falla — si docker no responde, loguea un warning y continúa.
fn show_running_containers_sync(runner: &PsRunner, tx: &LogSender) {
    let _ = tx.send(LogLine::info("Servicios activos:"));
    match runner.run_direct_sync(
        "docker",
        &[
            "ps",
            "-a",
            "--format",
            "{{.Names}} - {{.Status}}",
            "--filter",
            "name=WSDD-",
        ],
        None,
        None,
    ) {
        Ok(out) => {
            for line in out.text.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    let _ = tx.send(LogLine::info(format!("  {trimmed}")));
                }
            }
        }
        Err(e) => {
            let _ = tx.send(LogLine::warn(format!(
                "No se pudo listar contenedores: {e}"
            )));
        }
    }
}

// ─── Helpers de bridges y progreso ───────────────────────────────────────────

/// Crea un `OutputSender` que reenvía cada línea como `LogLine::info` al `LogSender`.
pub fn make_log_bridge(tx: &LogSender) -> OutputSender {
    let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
    let log_tx = tx.clone();
    std::thread::spawn(move || {
        while let Ok(line) = out_rx.recv() {
            let _ = log_tx.send(LogLine::info(line));
        }
    });
    out_tx
}

/// Ruta del archivo de log de deploy con rotación diaria.
fn deploy_log_path() -> std::path::PathBuf {
    let log_dir = std::path::Path::new(r"C:\WSDD-Environment\logs");
    let _ = std::fs::create_dir_all(log_dir);
    let day = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() / 86400)
        .unwrap_or(0);
    log_dir.join(format!("wsdd-deploy-d{day}.log"))
}

/// Crea un `OutputSender` que procesa el output de `docker-compose` con progress in-place.
///
/// - Líneas de capa (`<12-hex> <status>`): actualización in-place por hash.
/// - Líneas normales: append normal y escritura en log.
/// - Log en disco: solo escribe cuando cambia la categoría de estado de la capa.
fn make_docker_progress_bridge(tx: &LogSender) -> OutputSender {
    let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
    let log_tx = tx.clone();
    let log_path = deploy_log_path();

    std::thread::spawn(move || {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .ok();
        let mut layer_cats: HashMap<String, String> = HashMap::new();

        while let Ok(line) = out_rx.recv() {
            // Docker puede emitir múltiples actualizaciones separadas por \r
            for segment in line.split('\r') {
                let trimmed = segment.trim_end();
                if trimmed.is_empty() {
                    continue;
                }

                if let Some((hash, rest)) = parse_docker_layer_line(trimmed) {
                    let _ = log_tx.send(
                        LogLine::info(format!("  {hash}  {rest}")).with_key(hash.to_string()),
                    );
                    let cat = layer_status_category(rest).to_string();
                    let prev = layer_cats.entry(hash.to_string()).or_default();
                    if *prev != cat {
                        *prev = cat;
                        if let Some(ref mut f) = file {
                            let _ = writeln!(f, "{trimmed}");
                        }
                    }
                } else {
                    let _ = log_tx.send(LogLine::info(trimmed.to_string()));
                    if let Some(ref mut f) = file {
                        let _ = writeln!(f, "{trimmed}");
                    }
                }
            }
        }
    });

    out_tx
}

/// Detecta si una línea es de progreso de capa Docker.
///
/// Formato Docker Compose V2: espacios + icono Unicode + `<12-hex> <status>`
/// Retorna `(hash, rest)` si coincide, `None` si no es línea de capa.
fn parse_docker_layer_line(line: &str) -> Option<(&str, &str)> {
    let stripped = line.trim_start_matches(|c: char| !c.is_ascii_alphanumeric());
    let (hash, rest) = stripped.split_once(' ')?;
    if hash.len() == 12 && hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        Some((hash, rest.trim()))
    } else {
        None
    }
}

/// Clasifica el estado de una capa Docker para deduplicar el log.
fn layer_status_category(status: &str) -> &str {
    match status.split_whitespace().next().unwrap_or("") {
        "Pulling" | "Waiting" => "waiting",
        "Downloading" => "downloading",
        "Download" => "downloaded",
        "Verifying" => "verifying",
        "Extracting" => "extracting",
        "Pull" | "Extract" => "done",
        "Already" => "exists",
        _ => "other",
    }
}

/// Escribe una línea de cabecera con timestamp en el log de deploy.
fn write_deploy_log_header(label: &str) {
    let log_path = deploy_log_path();
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let _ = writeln!(f, "\n=== {label} [t={secs}] ===");
    }
}
