//! Managed MySQL core with an ephemeral primary and persistent replica.

use std::fs;
use std::time::{Duration, Instant};

use crate::config::environment::{env_config, path_config};
use crate::errors::InfraError;
use crate::handlers::deploy_trace::TraceHandle;
use crate::handlers::docker;
use crate::handlers::docker_deploy::{make_docker_progress_bridge, sync_prerequisite_compose_sync};
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::ps_script::{PsRunner, ScriptRunner};
use crate::handlers::setting::AppSettings;

pub const CONTAINER: &str = "WSDD-MySql-Server";
const PROJECT: &str = docker::WSDD_CORE_PROJECT;
const SERVICE: &str = "database";
#[cfg(test)]
const COMPOSE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/init.yml"
));
const DOCKERFILE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/bin/mysql/Dockerfile"
));
const PRIMARY_CONFIG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/bin/mysql/primary.cnf"
));
const REPLICA_CONFIG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/bin/mysql/replica.cnf"
));
const CONTROL: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/bin/mysql/wsdd-mysql"
));

fn core_dir() -> std::path::PathBuf {
    path_config()
        .docker_structure_dir()
        .join("bin")
        .join("mysql")
}

/// Returns the persistent replica directory used by the core Compose service.
pub fn data_dir() -> std::path::PathBuf {
    core_dir().join("data")
}

fn prepared_marker() -> std::path::PathBuf {
    data_dir().join(".wsdd-core-prepared")
}

/// Reconciles only WSDD-owned MySQL build files, preserving all database data.
pub fn sync_resources() -> Result<(), InfraError> {
    let root = core_dir();
    fs::create_dir_all(&root)?;
    for (name, bytes) in [
        ("Dockerfile", DOCKERFILE),
        ("primary.cnf", PRIMARY_CONFIG),
        ("replica.cnf", REPLICA_CONFIG),
        ("wsdd-mysql", CONTROL),
    ] {
        let path = root.join(name);
        if fs::read(&path).ok().as_deref() != Some(bytes) {
            fs::write(path, bytes)?;
        }
    }
    Ok(())
}

fn docker_run(runner: &PsRunner, args: &[&str], tx: &LogSender) -> Result<String, InfraError> {
    let bridge = make_docker_progress_bridge(tx);
    let result = runner.run_direct_sync(env_config().docker_exe(), args, None, Some(&bridge));
    Ok(bridge.complete(result, "docker mysql core")?.text)
}

fn require_current_protocol(runner: &PsRunner, tx: &LogSender) -> Result<(), InfraError> {
    let version = docker_run(
        runner,
        &["exec", CONTAINER, "/usr/local/bin/wsdd-mysql", "version"],
        tx,
    );
    if !matches!(version, Ok(ref value) if value.trim() == "wsdd-mysql-core:state-v1") {
        return Err(InfraError::PrerequisiteNotMet(
            "El contenedor MySQL existente no usa el protocolo de dos instancias. WSDD no lo reemplazará automáticamente; conserva bin/mysql/data y retira el contenedor antiguo antes de Deploy".into(),
        ));
    }
    Ok(())
}

/// Rejects a stopped legacy single-process image before it can touch the new datadir.
pub fn require_managed_image(runner: &PsRunner) -> Result<(), InfraError> {
    let out = runner.run_direct_sync(
        env_config().docker_exe(),
        &[
            "inspect",
            "--format",
            "{{json .Config.Entrypoint}}",
            CONTAINER,
        ],
        None,
        None,
    )?;
    if out.success && is_managed_entrypoint(&out.text) {
        return Ok(());
    }
    Err(unmanaged_image_error())
}

fn is_managed_entrypoint(output: &str) -> bool {
    output.trim() == r#"["/usr/local/bin/wsdd-mysql"]"#
}

fn unmanaged_image_error() -> InfraError {
    InfraError::PrerequisiteNotMet(
        "El contenedor MySQL conserva la imagen anterior. WSDD no lo iniciará sobre el datadir convertido; retira solo ese contenedor conservando bin/mysql/data y vuelve a desplegar".into(),
    )
}

pub async fn require_managed_image_async() -> Result<(), InfraError> {
    let out = crate::handlers::ps_script::run_docker(
        vec![
            "inspect".into(),
            "--format".into(),
            "{{json .Config.Entrypoint}}".into(),
            CONTAINER.into(),
        ],
        None,
    )
    .await?;
    if out.success && is_managed_entrypoint(&out.text) {
        Ok(())
    } else {
        Err(unmanaged_image_error())
    }
}

fn container_state(runner: &PsRunner) -> Result<Option<String>, InfraError> {
    let out = runner.run_direct_sync(
        env_config().docker_exe(),
        &[
            "ps",
            "-a",
            "--filter",
            &format!("name={CONTAINER}"),
            "--format",
            "{{.Names}}|{{.State}}",
        ],
        None,
        None,
    )?;
    if !out.success {
        return Err(InfraError::DockerUnreachable(out.text));
    }
    Ok(out.text.lines().find_map(|line| {
        let (name, state) = line.trim().split_once('|')?;
        (name == CONTAINER).then(|| state.to_owned())
    }))
}

/// Waits for both processes and replication to pass the container healthcheck.
pub fn wait_healthy(runner: &PsRunner, tx: &LogSender) -> Result<(), InfraError> {
    let deadline = Instant::now() + Duration::from_secs(600);
    let mut last_report = Instant::now() - Duration::from_secs(10);
    while Instant::now() < deadline {
        let out = runner.run_direct_sync(
            env_config().docker_exe(),
            &[
                "inspect",
                "--format",
                "{{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}",
                CONTAINER,
            ],
            None,
            None,
        )?;
        if out.success && out.text.trim() == "healthy" {
            let _ = docker_run(runner, &["logs", "--tail", "25", CONTAINER], tx);
            let _ = tx.send(LogLine::success(
                "MySQL Core: principal y réplica saludables",
            ));
            return Ok(());
        }
        if out.success && out.text.trim() == "none" {
            return Err(InfraError::PrerequisiteNotMet(
                "El contenedor MySQL existente no tiene el healthcheck del nuevo protocolo; no se reemplazará automáticamente".into(),
            ));
        }
        if container_state(runner)?.as_deref() != Some("running") {
            let _ = docker_run(runner, &["logs", "--tail", "80", CONTAINER], tx);
            return Err(InfraError::PrerequisiteNotMet(
                "MySQL Core terminó antes de alcanzar el healthcheck; revisa el log anterior"
                    .into(),
            ));
        }
        if last_report.elapsed() >= Duration::from_secs(10) {
            let _ = tx.send(LogLine::info(format!(
                "MySQL Core: healthcheck {}",
                out.text.trim()
            )));
            last_report = Instant::now();
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    let _ = docker_run(runner, &["logs", "--tail", "80", CONTAINER], tx);
    Err(InfraError::PrerequisiteNotMet(
        "MySQL Core no alcanzó estado healthy en 600 segundos".into(),
    ))
}

/// Creates or starts the core database, then waits for both MySQL instances.
pub fn deploy(
    runner: &PsRunner,
    settings: &AppSettings,
    tx: &LogSender,
    trace: &TraceHandle,
) -> Result<(), InfraError> {
    trace.stage("Recursos y propietario de MySQL Core", tx, || {
        docker::require_container_project(runner, CONTAINER, PROJECT)?;
        sync_resources()?;
        sync_prerequisite_compose_sync(settings)
    })?;
    trace.stage("Construcción e inicio de MySQL Core", tx, || {
        let compose = path_config().init_yml();
        let file = compose.to_string_lossy();
        match container_state(runner)? {
            Some(state) if state == "running" && prepared_marker().is_file() => {
                require_current_protocol(runner, tx)?;
                let _ = tx.send(LogLine::warn(
                    "MySQL Core quedó preparado tras una recreación interrumpida; reiniciando su contenedor",
                ));
                docker_run(runner, &["restart", "-t", "300", CONTAINER], tx)?;
            }
            Some(state) if state == "running" => {
                require_current_protocol(runner, tx)?;
                let _ = tx.send(LogLine::info(
                    "El contenedor MySQL Core ya existe; no se recreará",
                ));
            }
            Some(_) => {
                require_managed_image(runner)?;
                docker_run(runner, &["start", CONTAINER], tx)?;
            }
            None => {
                docker_run(
                    runner,
                    &[
                        "compose",
                        "-p",
                        PROJECT,
                        "-f",
                        &file,
                        "up",
                        "-d",
                        "--build",
                        "--no-recreate",
                        "--no-deps",
                        SERVICE,
                    ],
                    tx,
                )?;
            }
        }
        Ok(())
    })?;
    trace.stage("Verificación de ambas instancias", tx, || {
        wait_healthy(runner, tx)?;
        require_current_protocol(runner, tx)
    })
}

/// Recreates only the core database after a verified replication barrier.
pub fn recreate(
    runner: &PsRunner,
    settings: &AppSettings,
    tx: &LogSender,
    trace: &TraceHandle,
) -> Result<(), InfraError> {
    trace.stage("Comprobar propietario y estado", tx, || {
        docker::require_container_project(runner, CONTAINER, PROJECT)?;
        if container_state(runner)?.as_deref() != Some("running") {
            return Err(InfraError::PrerequisiteNotMet(
                "Despliega y valida MySQL Core antes de recrearlo".into(),
            ));
        }
        require_current_protocol(runner, tx)?;
        sync_resources()?;
        sync_prerequisite_compose_sync(settings)
    })?;
    trace.stage(
        "Construir imagen de reemplazo sin detener MySQL",
        tx,
        || {
            let compose = path_config().init_yml();
            let file = compose.to_string_lossy();
            docker_run(
                runner,
                &["compose", "-p", PROJECT, "-f", &file, "build", SERVICE],
                tx,
            )?;
            Ok(())
        },
    )?;
    trace.stage("Esperar réplica y apagar MySQL", tx, || {
        let output = docker_run(
            runner,
            &["exec", CONTAINER, "/usr/local/bin/wsdd-mysql", "prepare"],
            tx,
        )?;
        if !output.contains("READY:") {
            return Err(InfraError::PrerequisiteNotMet(
                "MySQL Core no confirmó un apagado consistente".into(),
            ));
        }
        Ok(())
    })?;
    trace.stage("Recrear solo MySQL Core", tx, || {
        let compose = path_config().init_yml();
        let file = compose.to_string_lossy();
        docker_run(
            runner,
            &[
                "compose",
                "-p",
                PROJECT,
                "-f",
                &file,
                "up",
                "-d",
                "--no-build",
                "--force-recreate",
                "--no-deps",
                SERVICE,
            ],
            tx,
        )?;
        Ok(())
    })?;
    trace.stage("Validar restauración y replicación", tx, || {
        wait_healthy(runner, tx)
    })
}

/// Runs the end-to-end SQL witness and replica comparison.
pub fn validate(runner: &PsRunner, tx: &LogSender, trace: &TraceHandle) -> Result<(), InfraError> {
    trace.stage("Comprobar propietario y estado", tx, || {
        docker::require_container_project(runner, CONTAINER, PROJECT)?;
        if container_state(runner)?.as_deref() != Some("running") {
            return Err(InfraError::PrerequisiteNotMet(
                "El contenedor MySQL Core no está en ejecución".into(),
            ));
        }
        require_current_protocol(runner, tx)?;
        Ok(())
    })?;
    trace.stage("Prueba SQL de extremo a extremo", tx, || {
        docker_run(
            runner,
            &["exec", CONTAINER, "/usr/local/bin/wsdd-mysql", "validate"],
            tx,
        )?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_keeps_existing_identity_and_uses_persistent_replica() {
        let compose = COMPOSE;
        assert!(compose.contains("WSDD-MySql-Server"));
        assert!(compose.contains("./bin/mysql/data:/var/lib/mysql:rw"));
        assert!(compose.contains("127.0.0.1:3306:3306"));
        assert!(compose.contains("PMA_HOST: database"));
        let primary = std::str::from_utf8(PRIMARY_CONFIG).unwrap();
        let replica = std::str::from_utf8(REPLICA_CONFIG).unwrap();
        assert!(primary.contains("datadir=/opt/wsdd-mysql-primary"));
        assert!(!primary.contains("datadir=/var/lib/mysql"));
        assert!(primary.contains("lower-case-table-names=1"));
        assert!(primary.contains("relay-log=relay-bin"));
        assert!(primary.contains("relay-log-recovery=ON"));
        assert!(replica.contains("lower-case-table-names=1"));
        assert!(std::str::from_utf8(DOCKERFILE)
            .unwrap()
            .contains("FROM mysql:26.7.0@sha256:"));
        let control = std::str::from_utf8(CONTROL).unwrap();
        assert!(control.contains("wsdd-mysql-core:state-v1"));
        assert!(control.contains("digest=$(compare_state)"));
        assert!(control.contains("expected=$(cat \"$state_file\")"));
    }
}
