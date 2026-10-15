use std::time::{Duration, Instant};

use crate::config::environment::{env_config, path_config};
use crate::errors::InfraError;
use crate::handlers::docker::WSDD_PROJECT;
use crate::handlers::docker_deploy::make_docker_progress_bridge;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::ps_script::ScriptRunner;
use crate::models::project::PhpVersion;

pub(super) fn deploy(
    version: &PhpVersion,
    runner: &impl ScriptRunner,
    tx: &LogSender,
    prepare: impl FnOnce() -> Result<(), InfraError>,
) -> Result<(), InfraError> {
    let name = format!("WSDD-Web-Server-{}", version.runtime_container_tag());
    let inventory = runner.run_direct_sync(
        env_config().docker_exe(),
        &["ps", "-a", "--format", "{{.Names}}"],
        None,
        None,
    )?;
    if !inventory.success {
        return Err(InfraError::DockerUnreachable(inventory.text));
    }
    if inventory.text.lines().any(|line| line.trim() == name) {
        return Err(InfraError::PrerequisiteNotMet(format!(
            "{name} ya existe. No se modifico el contenedor; use su toolbox."
        )));
    }

    let _ = tx.send(LogLine::info(format!("[Deploy PHP] Preparando {name}...")));
    prepare()?;
    let dir = path_config().php_dir(version.dir_name());
    let webserver = format!("webserver.{}.yml", version.compose_tag());
    let options = format!("options.{}.yml", version.compose_tag());
    let service = format!("webserver{}", version.host_suffix());
    // Build before up: a failed build never removes an existing runtime.
    for command in compose_commands(&webserver, &options, &service) {
        let bridge = make_docker_progress_bridge(tx);
        let output = runner.run_ps_sync(&command, Some(&dir), Some(&bridge));
        bridge.complete(output, "docker-compose PHP")?;
    }

    let deadline = Instant::now() + Duration::from_secs(90);
    let apache_probe = format!(
        "apache2ctl -t && curl --fail --silent --max-time 5 --output /dev/null --header 'Host: php{}.wsdd.dock' http://127.0.0.1/",
        version.host_suffix()
    );
    loop {
        let state = runner.run_direct_sync(
            env_config().docker_exe(),
            &[
                "inspect",
                "--format",
                "{{.State.Running}} {{.State.Restarting}}",
                &name,
            ],
            None,
            None,
        )?;
        if !state.success {
            return Err(InfraError::UnexpectedOutput(
                "docker inspect PHP".into(),
                state.text,
            ));
        }
        if state.text.trim() == "true false" {
            let check = runner.run_direct_sync(
                env_config().docker_exe(),
                &["exec", &name, "sh", "-c", &apache_probe],
                None,
                None,
            )?;
            if check.success {
                break;
            }
        }
        if Instant::now() >= deadline {
            return Err(InfraError::PrerequisiteNotMet(format!(
                "{name} no alcanzo disponibilidad Apache en 90 segundos. Revise sus logs; no se eliminaron datos."
            )));
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    let _ = tx.send(LogLine::success(format!(
        "[Deploy PHP] {name} en ejecucion; Apache responde. No se crearon proyectos."
    )));
    Ok(())
}

fn compose_commands(webserver: &str, options: &str, service: &str) -> [String; 2] {
    let prefix = format!(
        "{} -p {WSDD_PROJECT} -f \"{webserver}\" -f \"{options}\"",
        env_config().docker_compose_exe()
    );
    [
        format!("{prefix} build {service}"),
        format!("{prefix} up -d --no-build --no-recreate --no-deps {service}"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::ps_script::{OutputSender, ProcOutput};
    use std::cell::Cell;
    use std::path::Path;
    use std::sync::Mutex;

    struct Runner {
        inventory: ProcOutput,
        build_ok: bool,
        commands: Mutex<Vec<String>>,
    }

    impl ScriptRunner for Runner {
        fn run_script_sync(
            &self,
            _: &str,
            _: Option<&Path>,
            _: Option<&OutputSender>,
        ) -> Result<ProcOutput, InfraError> {
            panic!("No scripts or project operations expected")
        }
        fn run_ps_sync(
            &self,
            cmd: &str,
            _: Option<&Path>,
            _: Option<&OutputSender>,
        ) -> Result<ProcOutput, InfraError> {
            self.commands.lock().unwrap().push(cmd.into());
            Ok(ProcOutput {
                text: String::new(),
                success: self.build_ok,
            })
        }
        fn run_direct_sync(
            &self,
            _: &str,
            args: &[&str],
            _: Option<&Path>,
            _: Option<&OutputSender>,
        ) -> Result<ProcOutput, InfraError> {
            self.commands.lock().unwrap().push(args.join(" "));
            Ok(match args[0] {
                "ps" => self.inventory.clone(),
                "inspect" => ProcOutput {
                    text: "true false".into(),
                    success: true,
                },
                "exec" => ProcOutput {
                    text: String::new(),
                    success: true,
                },
                _ => panic!("Unexpected operation: {args:?}"),
            })
        }
    }

    fn runner(text: &str, success: bool, build_ok: bool) -> Runner {
        Runner {
            inventory: ProcOutput {
                text: text.into(),
                success,
            },
            build_ok,
            commands: Mutex::new(Vec::new()),
        }
    }

    #[test]
    fn existing_runtime_or_failed_inventory_never_prepares_or_builds() {
        for r in [
            runner("WSDD-Web-Server-PHP8.3\n", true, true),
            runner("", false, true),
        ] {
            let (tx, _) = std::sync::mpsc::channel();
            assert!(deploy(&PhpVersion::Php83, &r, &tx, || panic!("Must not prepare")).is_err());
            assert_eq!(r.commands.lock().unwrap().len(), 1);
        }
    }

    #[test]
    fn failed_build_does_not_run_up_or_remove_anything() {
        let r = runner("", true, false);
        let (tx, _) = std::sync::mpsc::channel();
        assert!(deploy(&PhpVersion::Php56, &r, &tx, || Ok(())).is_err());
        let commands = r.commands.lock().unwrap();
        assert_eq!(commands.len(), 2);
        assert!(commands[1].ends_with("build webserver56"));
    }

    #[test]
    fn all_versions_deploy_only_their_service_without_project_operations() {
        for version in PhpVersion::all() {
            let r = runner("WSDD-MySql-Server\nWSDD-Web-Server-PHP8.30", true, true);
            let (tx, _) = std::sync::mpsc::channel();
            let prepared = Cell::new(false);
            deploy(&version, &r, &tx, || {
                prepared.set(true);
                Ok(())
            })
            .unwrap();
            assert!(prepared.get());
            let commands = r.commands.lock().unwrap();
            assert_eq!(commands.len(), 5);
            assert!(commands[1].contains(&format!("webserver.{}.yml", version.compose_tag())));
            assert!(commands[2].ends_with(&format!(
                "up -d --no-build --no-recreate --no-deps webserver{}",
                version.host_suffix()
            )));
        }
    }

    #[test]
    fn preparation_failure_never_builds() {
        let r = runner("", true, true);
        let (tx, _) = std::sync::mpsc::channel();
        assert!(deploy(&PhpVersion::Php83, &r, &tx, || Err(
            InfraError::PrerequisiteNotMet("prepare failed".into())
        ))
        .is_err());
        assert_eq!(r.commands.lock().unwrap().len(), 1);
    }
}
