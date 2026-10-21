//! Explicit, isolated MySQL replication experiment.

use std::fs;
use std::time::{Duration, Instant};

use crate::config::environment::{env_config, path_config};
use crate::errors::InfraError;
use crate::handlers::deploy_trace::TraceHandle;
use crate::handlers::docker;
use crate::handlers::docker_deploy::make_docker_progress_bridge;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::ps_script::{PsRunner, ScriptRunner};

pub const CONTAINER: &str = "WSDD-MySql-Server-Test";
const PROJECT: &str = "wsdd-mysql-test";
const SERVICE: &str = "database-test";
const COMPOSE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/bin/mysql-test/compose.yml"
));
const DOCKERFILE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/bin/mysql-test/Dockerfile"
));
const PRIMARY_CONFIG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/bin/mysql-test/primary.cnf"
));
const REPLICA_CONFIG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/bin/mysql-test/replica.cnf"
));
const CONTROL: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/bin/mysql-test/wsdd-mysql-test"
));

fn test_dir() -> std::path::PathBuf {
    path_config()
        .docker_structure_dir()
        .join("bin")
        .join("mysql-test")
}

pub fn data_dir() -> std::path::PathBuf {
    test_dir().join("data")
}

fn prepared_marker() -> std::path::PathBuf {
    data_dir().join(".wsdd-test-prepared")
}

fn sync_resources() -> Result<(), InfraError> {
    let root = test_dir();
    fs::create_dir_all(&root)?;
    for (name, bytes) in [
        ("compose.yml", COMPOSE),
        ("Dockerfile", DOCKERFILE),
        ("primary.cnf", PRIMARY_CONFIG),
        ("replica.cnf", REPLICA_CONFIG),
        ("wsdd-mysql-test", CONTROL),
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
    Ok(bridge.complete(result, "docker mysql-test")?.text)
}

fn require_current_protocol(runner: &PsRunner, tx: &LogSender) -> Result<(), InfraError> {
    let version = docker_run(
        runner,
        &[
            "exec",
            CONTAINER,
            "/usr/local/bin/wsdd-mysql-test",
            "version",
        ],
        tx,
    );
    if !matches!(version, Ok(ref value) if value.trim() == "wsdd-mysql-test:state-v2") {
        return Err(InfraError::PrerequisiteNotMet(
            "El contenedor MySQL Test usa el protocolo anterior. Retíralo desde Docker conservando bin/mysql-test/data y pulsa Deploy para construir la versión nueva".into(),
        ));
    }
    Ok(())
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

fn wait_healthy(runner: &PsRunner, tx: &LogSender) -> Result<(), InfraError> {
    let deadline = Instant::now() + Duration::from_secs(180);
    let mut last_report = Instant::now() - Duration::from_secs(10);
    while Instant::now() < deadline {
        let out = runner.run_direct_sync(
            env_config().docker_exe(),
            &["inspect", "--format", "{{.State.Health.Status}}", CONTAINER],
            None,
            None,
        )?;
        if out.success && out.text.trim() == "healthy" {
            let _ = docker_run(runner, &["logs", "--tail", "25", CONTAINER], tx);
            let _ = tx.send(LogLine::success(
                "MySQL Test: principal y réplica saludables",
            ));
            return Ok(());
        }
        if container_state(runner)?.as_deref() != Some("running") {
            let _ = docker_run(runner, &["logs", "--tail", "80", CONTAINER], tx);
            return Err(InfraError::PrerequisiteNotMet(
                "MySQL Test terminó antes de alcanzar el healthcheck; revisa el log anterior"
                    .into(),
            ));
        }
        if last_report.elapsed() >= Duration::from_secs(10) {
            let _ = tx.send(LogLine::info(format!(
                "MySQL Test: healthcheck {}",
                out.text.trim()
            )));
            last_report = Instant::now();
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    let _ = docker_run(runner, &["logs", "--tail", "80", CONTAINER], tx);
    Err(InfraError::PrerequisiteNotMet(
        "MySQL Test no alcanzó estado healthy en 180 segundos".into(),
    ))
}

pub fn deploy(runner: &PsRunner, tx: &LogSender, trace: &TraceHandle) -> Result<(), InfraError> {
    trace.stage("Aislamiento y recursos de prueba", tx, || {
        docker::require_container_project(runner, CONTAINER, PROJECT)?;
        sync_resources()
    })?;
    trace.stage("Construcción e inicio de MySQL Test", tx, || {
        let compose = test_dir().join("compose.yml");
        let file = compose.to_string_lossy();
        match container_state(runner)? {
            Some(state) if state == "running" && prepared_marker().is_file() => {
                require_current_protocol(runner, tx)?;
                let _ = tx.send(LogLine::warn(
                    "MySQL Test quedó preparado tras una recreación interrumpida; reiniciando su contenedor",
                ));
                docker_run(runner, &["restart", "-t", "120", CONTAINER], tx)?;
            }
            Some(state) if state == "running" => {
                require_current_protocol(runner, tx)?;
                let _ = tx.send(LogLine::info(
                    "El contenedor de prueba ya existe; no se recreará",
                ));
            }
            Some(_) => {
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

pub fn recreate(runner: &PsRunner, tx: &LogSender, trace: &TraceHandle) -> Result<(), InfraError> {
    trace.stage("Comprobar propietario y estado", tx, || {
        docker::require_container_project(runner, CONTAINER, PROJECT)?;
        if container_state(runner)?.as_deref() != Some("running") {
            return Err(InfraError::PrerequisiteNotMet(
                "Despliega y valida MySQL Test antes de recrearlo".into(),
            ));
        }
        require_current_protocol(runner, tx)?;
        sync_resources()
    })?;
    trace.stage(
        "Construir imagen de reemplazo sin detener MySQL",
        tx,
        || {
            let compose = test_dir().join("compose.yml");
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
            &[
                "exec",
                CONTAINER,
                "/usr/local/bin/wsdd-mysql-test",
                "prepare",
            ],
            tx,
        )?;
        if !output.contains("READY:") {
            return Err(InfraError::PrerequisiteNotMet(
                "MySQL Test no confirmó un apagado consistente".into(),
            ));
        }
        Ok(())
    })?;
    trace.stage("Recrear solo el contenedor de prueba", tx, || {
        let compose = test_dir().join("compose.yml");
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

pub fn validate(runner: &PsRunner, tx: &LogSender, trace: &TraceHandle) -> Result<(), InfraError> {
    trace.stage("Comprobar propietario y estado", tx, || {
        docker::require_container_project(runner, CONTAINER, PROJECT)?;
        if container_state(runner)?.as_deref() != Some("running") {
            return Err(InfraError::PrerequisiteNotMet(
                "El contenedor MySQL Test no está en ejecución".into(),
            ));
        }
        require_current_protocol(runner, tx)?;
        Ok(())
    })?;
    trace.stage("Prueba SQL de extremo a extremo", tx, || {
        docker_run(
            runner,
            &[
                "exec",
                CONTAINER,
                "/usr/local/bin/wsdd-mysql-test",
                "validate",
            ],
            tx,
        )?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn experiment_does_not_share_production_mount_or_ports() {
        let compose = std::str::from_utf8(COMPOSE).unwrap();
        assert!(compose.contains("WSDD-MySql-Server-Test"));
        assert!(compose.contains("./data:/var/lib/mysql:rw"));
        assert!(!compose.contains("ports:"));
        assert!(!compose.contains("../mysql/data"));
        assert!(!compose.contains("WSDD-MySql-Server\n"));
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
            .contains("FROM mysql:8.4.11"));
        let control = std::str::from_utf8(CONTROL).unwrap();
        assert!(control.contains("wsdd-mysql-test:state-v2"));
        assert!(control.contains("digest=$(compare_state)"));
        assert!(control.contains("expected=$(cat \"$state_file\")"));
    }
}
