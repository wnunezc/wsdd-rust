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
//! Base container deployment and readiness checks.

use std::time::{Duration, Instant};

use super::super::progress::{deploy_log_path, make_docker_progress_bridge};
use crate::config::environment::{env_config, path_config};
use crate::errors::InfraError;
use crate::handlers::docker::WSDD_PROJECT;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::ps_script::{PsRunner, ScriptRunner};

const CREATE_READY_TIMEOUT: Duration = Duration::from_secs(30);
const START_READY_TIMEOUT: Duration = Duration::from_secs(90);
const READY_POLL_INTERVAL: Duration = Duration::from_secs(1);

// Dependency order, independent of the Compose project that owns each container.
const BASE_SERVICES: [(&str, &str); 3] = [
    ("WSDD-MySql-Server", "database"),
    ("WSDD-phpMyAdmin-Server", "phpmyadmin"),
    ("WSDD-Proxy-Server", "reverse-proxy"),
];

#[derive(Debug, PartialEq)]
enum RecoveryAction {
    Start(&'static str),
    Create(&'static str),
}

fn base_states_sync(runner: &impl ScriptRunner) -> Result<Vec<(String, String)>, InfraError> {
    let out = runner.run_direct_sync(
        env_config().docker_exe(),
        &["ps", "-a", "--format", "{{.Names}}|{{.State}}"],
        None,
        None,
    )?;
    if !out.success {
        return Err(InfraError::DockerUnreachable(out.text));
    }
    Ok(out
        .text
        .lines()
        .filter_map(|line| {
            let (name, state) = line.trim().split_once('|')?;
            BASE_SERVICES
                .iter()
                .any(|(expected, _)| name == *expected)
                .then(|| (name.to_string(), state.to_string()))
        })
        .collect())
}

fn recovery_actions(states: &[(String, String)]) -> Result<Vec<RecoveryAction>, InfraError> {
    let mut actions = Vec::new();
    for (name, service) in BASE_SERVICES {
        match states
            .iter()
            .find(|(candidate, _)| candidate == name)
            .map(|(_, state)| state.as_str())
        {
            None => actions.push(RecoveryAction::Create(service)),
            Some("created" | "exited") => actions.push(RecoveryAction::Start(name)),
            Some("running" | "restarting") => {}
            Some(state) => {
                return Err(InfraError::PrerequisiteNotMet(format!(
                "{name} esta en estado {state}; no se eliminara ni reemplazara automaticamente."
            )))
            }
        }
    }
    Ok(actions)
}

fn create_command(prefix: &str, service: &str) -> String {
    format!("{prefix} up -d --build --no-recreate --no-deps {service}")
}

fn apply_recovery_actions(
    actions: &[RecoveryAction],
    mut execute: impl FnMut(&RecoveryAction) -> Result<(), InfraError>,
) -> Result<(), InfraError> {
    for action in actions {
        execute(action)?;
    }
    Ok(())
}

/// Deploys or recovers the base WSDD Docker containers.
pub(super) fn deploy_base_containers_sync(
    runner: &PsRunner,
    tx: &LogSender,
) -> Result<(), InfraError> {
    if check_base_containers_running_sync(runner)? {
        let _ = tx.send(LogLine::success(
            "✓ Contenedores WSDD ya están desplegados y activos",
        ));
        return Ok(());
    }

    let init_yml = path_config().init_yml();
    let docker_dir = path_config().docker_structure_dir();
    let log_path = deploy_log_path();
    let actions = recovery_actions(&base_states_sync(runner)?)?;
    let prefix = format!(
        "{} -p {WSDD_PROJECT} -f \"{}\"",
        env_config().docker_compose_exe(),
        init_yml.display()
    );
    if actions
        .iter()
        .any(|action| matches!(action, RecoveryAction::Create(_)))
    {
        let validation =
            runner.run_ps_sync(&format!("{prefix} config --quiet"), Some(&docker_dir), None)?;
        if !validation.success {
            return Err(InfraError::UnexpectedOutput(
                "docker-compose config".into(),
                validation.text,
            ));
        }
    }

    let command_label = "base container recovery";
    let _ = tx.send(LogLine::info(
        "Recuperando servicios base sin reemplazar contenedores existentes...",
    ));
    let _ = tx.send(LogLine::info(format!(
        "  (output detallado en: {})",
        log_path.display()
    )));
    super::super::progress::write_deploy_log_header(command_label);
    apply_recovery_actions(&actions, |action| {
        let bridge = make_docker_progress_bridge(tx);
        match action {
            RecoveryAction::Start(name) => {
                let _ = tx.send(LogLine::info(format!(
                    "[Runtime] Iniciando {name} existente..."
                )));
                let result = runner.run_direct_sync(
                    env_config().docker_exe(),
                    &["start", name],
                    None,
                    Some(&bridge),
                );
                bridge.complete(result, "docker start")?;
            }
            RecoveryAction::Create(service) => {
                let _ = tx.send(LogLine::info(format!(
                    "[Runtime] Creando solamente servicio base ausente: {service}..."
                )));
                let result = runner.run_ps_sync(
                    &create_command(&prefix, service),
                    Some(&docker_dir),
                    Some(&bridge),
                );
                bridge.complete(result, "docker-compose up missing service")?;
            }
        }
        Ok(())
    })?;

    let _ = tx.send(LogLine::info(
        "Esperando confirmacion real de contenedores creados...",
    ));
    if !wait_until_sync(CREATE_READY_TIMEOUT, READY_POLL_INTERVAL, || {
        check_base_containers_sync(runner)
    })? {
        let _ = tx.send(LogLine::error(
            "Los contenedores no se crearon correctamente",
        ));
        return Err(InfraError::UnexpectedOutput(
            command_label.to_string(),
            "contenedores no encontrados tras la creacion".to_string(),
        ));
    }

    if !check_base_containers_sync(runner)? {
        let _ = tx.send(LogLine::error(
            "✗ Los contenedores no se crearon correctamente",
        ));
        return Err(InfraError::UnexpectedOutput(
            command_label.to_string(),
            "contenedores no encontrados tras la creación".to_string(),
        ));
    }

    let _ = tx.send(LogLine::info(
        "Esperando disponibilidad real de servicios base...",
    ));
    if !wait_until_sync(START_READY_TIMEOUT, READY_POLL_INTERVAL, || {
        check_base_containers_running_sync(runner)
    })? {
        log_final_container_status(runner, tx);
        return Err(InfraError::UnexpectedOutput(
            command_label.to_string(),
            "los servicios base no alcanzaron estado running".to_string(),
        ));
    }

    let _ = tx.send(LogLine::success(
        "✓ Contenedores WSDD desplegados correctamente",
    ));
    Ok(())
}

/// Logs active WSDD containers after provisioning completes.
pub(super) fn show_running_containers_sync(runner: &PsRunner, tx: &LogSender) {
    let _ = tx.send(LogLine::info("Servicios activos:"));
    match runner.run_direct_sync(
        env_config().docker_exe(),
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

fn check_base_containers_sync(runner: &PsRunner) -> Result<bool, InfraError> {
    Ok(base_states_sync(runner)?.len() == BASE_SERVICES.len())
}

fn check_base_containers_running_sync(runner: &PsRunner) -> Result<bool, InfraError> {
    let states = base_states_sync(runner)?;
    Ok(states.len() == BASE_SERVICES.len() && states.iter().all(|(_, state)| state == "running"))
}

fn wait_until_sync<F>(
    timeout: Duration,
    interval: Duration,
    mut check: F,
) -> Result<bool, InfraError>
where
    F: FnMut() -> Result<bool, InfraError>,
{
    let deadline = Instant::now() + timeout;

    loop {
        if check()? {
            return Ok(true);
        }

        if Instant::now() >= deadline {
            return Ok(false);
        }

        std::thread::sleep(interval);
    }
}

fn base_container_status_lines_sync(runner: &PsRunner) -> Result<Vec<String>, InfraError> {
    let out = runner.run_direct_sync(
        env_config().docker_exe(),
        &[
            "ps",
            "-a",
            "--format",
            "{{.Names}}|{{.Status}}",
            "--filter",
            "name=WSDD-",
        ],
        None,
        None,
    )?;

    if out.text.contains("Error") {
        return Err(InfraError::DockerUnreachable(out.text));
    }

    Ok(out
        .text
        .lines()
        .filter(|line| {
            line.contains("WSDD-Proxy-Server")
                || line.contains("WSDD-MySql-Server")
                || line.contains("WSDD-phpMyAdmin-Server")
        })
        .map(|line| line.trim().to_string())
        .collect())
}

fn log_final_container_status(runner: &PsRunner, tx: &LogSender) {
    let _ = tx.send(LogLine::warn(
        "Estado detectado de contenedores base al expirar la espera:",
    ));
    match base_container_status_lines_sync(runner) {
        Ok(lines) if !lines.is_empty() => {
            for line in lines {
                let _ = tx.send(LogLine::warn(format!("  {line}")));
            }
        }
        Ok(_) => {
            let _ = tx.send(LogLine::warn(
                "  No se pudo obtener estado visible de los contenedores base",
            ));
        }
        Err(e) => {
            let _ = tx.send(LogLine::warn(format!(
                "  No se pudo consultar estado final de contenedores: {e}"
            )));
        }
    }
    let _ = tx.send(LogLine::error(
        "Los servicios base no quedaron disponibles a tiempo",
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::ps_script::{OutputSender, ProcOutput};
    use std::path::Path;

    fn states(values: [&str; 3]) -> Vec<(String, String)> {
        BASE_SERVICES
            .iter()
            .zip(values)
            .filter(|(_, state)| *state != "absent")
            .map(|((name, _), state)| (name.to_string(), state.to_string()))
            .collect()
    }

    #[test]
    fn stopped_mixed_project_containers_are_started_not_recreated() {
        // Project labels are deliberately irrelevant to docker start by exact name.
        let plan = recovery_actions(&states(["created", "created", "exited"])).unwrap();
        assert_eq!(
            plan,
            vec![
                RecoveryAction::Start("WSDD-MySql-Server"),
                RecoveryAction::Start("WSDD-phpMyAdmin-Server"),
                RecoveryAction::Start("WSDD-Proxy-Server"),
            ]
        );
    }

    #[test]
    fn missing_services_alone_are_created_without_changing_present_containers() {
        for a in ["absent", "running", "exited"] {
            for b in ["absent", "running", "exited"] {
                for c in ["absent", "running", "exited"] {
                    let input = [a, b, c];
                    let plan = recovery_actions(&states(input)).unwrap();
                    let expected: Vec<RecoveryAction> = BASE_SERVICES
                        .iter()
                        .zip(input)
                        .filter_map(|((name, service), state)| match state {
                            "absent" => Some(RecoveryAction::Create(service)),
                            "exited" => Some(RecoveryAction::Start(name)),
                            _ => None,
                        })
                        .collect();
                    assert_eq!(plan, expected);
                }
            }
        }
    }

    #[test]
    fn missing_service_compose_command_never_removes_orphans_or_recreates_dependencies() {
        for (_, service) in BASE_SERVICES {
            let command = create_command("docker-compose -p wsdd-projects -f init.yml", service);
            assert!(command.ends_with(&format!("--no-recreate --no-deps {service}")));
            assert!(!command.contains("--remove-orphans"));
            assert!(!command.contains("--force-recreate"));
        }
    }

    #[test]
    fn restarting_is_not_running_and_is_not_automatically_replaced() {
        assert!(
            recovery_actions(&states(["restarting", "running", "running"]))
                .unwrap()
                .is_empty()
        );
        for state in ["paused", "dead", "removing", "unknown"] {
            assert!(recovery_actions(&states([state, "running", "running"])).is_err());
        }
    }

    #[test]
    fn failed_recovery_does_not_continue_to_other_services() {
        let plan = recovery_actions(&states(["exited", "created", "exited"])).unwrap();
        let mut calls = 0;
        assert!(apply_recovery_actions(&plan, |_| {
            calls += 1;
            Err(InfraError::PrerequisiteNotMet("start failed".into()))
        })
        .is_err());
        assert_eq!(calls, 1);
    }

    struct InventoryRunner(bool);

    impl ScriptRunner for InventoryRunner {
        fn run_script_sync(
            &self,
            _: &str,
            _: Option<&Path>,
            _: Option<&OutputSender>,
        ) -> Result<ProcOutput, InfraError> {
            panic!("No script execution permitted")
        }
        fn run_ps_sync(
            &self,
            _: &str,
            _: Option<&Path>,
            _: Option<&OutputSender>,
        ) -> Result<ProcOutput, InfraError> {
            panic!("No Compose execution permitted")
        }
        fn run_direct_sync(
            &self,
            _: &str,
            args: &[&str],
            _: Option<&Path>,
            _: Option<&OutputSender>,
        ) -> Result<ProcOutput, InfraError> {
            assert_eq!(args, ["ps", "-a", "--format", "{{.Names}}|{{.State}}"]);
            Ok(ProcOutput { success: self.0, text: "WSDD-MySql-Server|created\nWSDD-MySql-Server-copy|running\nWSDD-Web-Server-PHP8.4|exited".into() })
        }
    }

    #[test]
    fn inventory_checks_exit_status_and_matches_only_exact_base_names() {
        assert!(base_states_sync(&InventoryRunner(false)).is_err());
        assert_eq!(
            base_states_sync(&InventoryRunner(true)).unwrap(),
            vec![("WSDD-MySql-Server".into(), "created".into())]
        );
    }
}
