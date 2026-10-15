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
//! Docker deploy log bridges and progress parsing.

use std::io::Write;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::config::environment::path_config;
use crate::errors::InfraError;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::ps_script::{OutputSender, ProcOutput};

static NEXT_OPERATION: AtomicU64 = AtomicU64::new(1);

pub struct LogBridge {
    sender: Option<OutputSender>,
    worker: Option<std::thread::JoinHandle<()>>,
    log_path: Option<PathBuf>,
}

impl Deref for LogBridge {
    type Target = OutputSender;
    fn deref(&self) -> &Self::Target {
        self.sender.as_ref().expect("active log bridge")
    }
}

impl LogBridge {
    fn drain(&mut self) {
        self.sender.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }

    /// Drain all output before returning success or propagating a concise error.
    pub fn complete(
        mut self,
        result: Result<ProcOutput, InfraError>,
        command: &str,
    ) -> Result<ProcOutput, InfraError> {
        self.drain();
        let output = result?;
        if output.success {
            return Ok(output);
        }
        let details = match &self.log_path {
            Some(path) => format!("Salida ya mostrada. Log completo: {}", path.display()),
            None => "Salida ya mostrada; no se pudo guardar el archivo de diagnostico.".into(),
        };
        Err(InfraError::StreamedProcessFailed {
            command: command.into(),
            details,
        })
    }
}

impl Drop for LogBridge {
    fn drop(&mut self) {
        self.drain();
    }
}

/// Creates an output bridge that forwards each process line into the UI log.
pub fn make_log_bridge(tx: &LogSender) -> LogBridge {
    create_bridge(tx, deploy_log_path(), false)
}

/// Returns the daily deploy log path.
pub(super) fn deploy_log_path() -> std::path::PathBuf {
    let log_dir = path_config().logs_dir();
    let _ = std::fs::create_dir_all(log_dir);
    let day = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() / 86400)
        .unwrap_or(0);
    path_config().deploy_log_file(day)
}

/// Creates an output bridge that compacts Docker layer progress in the UI.
pub fn make_docker_progress_bridge(tx: &LogSender) -> LogBridge {
    create_bridge(tx, deploy_log_path(), true)
}

fn create_bridge(tx: &LogSender, path: PathBuf, compact: bool) -> LogBridge {
    let (out_tx, out_rx) = std::sync::mpsc::channel::<String>();
    let log_tx = tx.clone();
    let scope = NEXT_OPERATION.fetch_add(1, Ordering::Relaxed);
    let mut file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        Ok(file) => Some(file),
        Err(error) => {
            let _ = tx.send(LogLine::warn(format!(
                "No se pudo guardar log {}: {error}",
                path.display()
            )));
            None
        }
    };
    let log_path = file.as_ref().map(|_| path);
    if let Some(file) = &mut file {
        let _ = writeln!(file, "\n=== process operation {scope} ===");
    }

    let worker = std::thread::spawn(move || {
        while let Ok(line) = out_rx.recv() {
            for segment in line.split(['\r', '\n']) {
                let trimmed = segment.trim_end();
                if trimmed.is_empty() {
                    continue;
                }

                if let Some(file) = &mut file {
                    let _ = writeln!(file, "{trimmed}");
                }
                if let Some((hash, rest)) =
                    compact.then(|| parse_docker_layer_line(trimmed)).flatten()
                {
                    let _ = log_tx.send(
                        LogLine::raw(format!("  {hash}  {rest}"))
                            .with_key(format!("operation:{scope}:{}", docker_layer_key(hash))),
                    );
                } else if let Some(key) = compact
                    .then(|| parse_buildkit_progress_key(trimmed))
                    .flatten()
                {
                    let _ = log_tx.send(
                        LogLine::raw(trimmed.to_string())
                            .with_key(format!("operation:{scope}:{key}")),
                    );
                } else {
                    let _ = log_tx.send(LogLine::raw(trimmed.to_string()));
                }
            }
        }
    });

    LogBridge {
        sender: Some(out_tx),
        worker: Some(worker),
        log_path,
    }
}

/// Writes a timestamped deploy log header.
pub(super) fn write_deploy_log_header(label: &str) {
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

fn parse_docker_layer_line(line: &str) -> Option<(&str, &str)> {
    let stripped = line.trim_start_matches(|c: char| !c.is_ascii_alphanumeric());
    let (hash, rest) = stripped.split_once(' ')?;
    if hash.len() == 12 && hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        Some((hash, rest.trim()))
    } else {
        None
    }
}

fn docker_layer_key(hash: &str) -> String {
    format!("docker-layer:{hash}")
}

fn parse_buildkit_progress_key(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let (step, rest) = trimmed.split_once(' ')?;
    if !is_buildkit_step(step) || !rest.contains("sha256:") {
        return None;
    }

    let digest = rest
        .split_whitespace()
        .find_map(|part| part.strip_prefix("sha256:"))?
        .chars()
        .take(16)
        .collect::<String>();

    if digest.is_empty() {
        None
    } else {
        Some(format!("buildkit:{step}:sha256:{digest}"))
    }
}

fn is_buildkit_step(value: &str) -> bool {
    value
        .strip_prefix('#')
        .is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_docker_layer_line_after_progress_control_chars() {
        let parsed = parse_docker_layer_line("\r\n    8a1b2c3d4e5f Downloading [====>]");

        assert_eq!(parsed, Some(("8a1b2c3d4e5f", "Downloading [====>]")));
    }

    #[test]
    fn docker_layer_key_is_stable_for_repeated_progress_segments() {
        assert_eq!(
            docker_layer_key("8a1b2c3d4e5f"),
            docker_layer_key("8a1b2c3d4e5f")
        );
    }

    #[test]
    fn buildkit_sha_progress_uses_stable_key() {
        let line = "#6 sha256:0c9513d9d1269ed66e93cdd86e573688d09e1e04c4a2114eb2109b350a272694 9.44MB / 47.31MB 6.0s";

        assert_eq!(
            parse_buildkit_progress_key(line),
            Some("buildkit:#6:sha256:0c9513d9d1269ed6".to_string())
        );
    }

    #[test]
    fn buildkit_non_digest_output_is_not_compacted() {
        assert_eq!(parse_buildkit_progress_key("#6 RUN apt-get update"), None);
    }

    #[test]
    fn complete_drains_output_before_the_final_status_and_preserves_full_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("process.log");
        let (tx, rx) = std::sync::mpsc::channel();
        let bridge = create_bridge(&tx, path.clone(), true);
        for index in 0..500 {
            bridge.send(format!("output-{index}")).unwrap();
        }
        bridge
            .complete(
                Ok(ProcOutput {
                    text: String::new(),
                    success: true,
                }),
                "test",
            )
            .unwrap();
        tx.send(LogLine::success("finished")).unwrap();
        let lines: Vec<_> = rx.try_iter().collect();
        assert_eq!(lines.len(), 501);
        for (index, line) in lines[..500].iter().enumerate() {
            assert_eq!(line.text, format!("output-{index}"));
        }
        assert_eq!(
            lines.last().unwrap().level,
            crate::handlers::log_types::LogLevel::Success
        );
        let saved = std::fs::read_to_string(path).unwrap();
        assert_eq!(
            saved
                .lines()
                .filter(|line| line.starts_with("output-"))
                .count(),
            500
        );
    }

    #[test]
    fn failed_process_does_not_replay_the_captured_transcript() {
        let dir = tempfile::tempdir().unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let bridge = create_bridge(&tx, dir.path().join("process.log"), false);
        let transcript = "unique-output-marker\n".repeat(1000);
        bridge.send(transcript.clone()).unwrap();
        let error = bridge
            .complete(
                Ok(ProcOutput {
                    text: transcript,
                    success: false,
                }),
                "docker-compose up",
            )
            .unwrap_err();
        let summary = error.to_string();
        assert!(summary.contains("docker-compose up"));
        assert!(!summary.contains("unique-output-marker"));
        assert!(summary.len() < 500);
        assert_eq!(rx.try_iter().count(), 1000);
    }

    #[test]
    fn progress_keys_are_stable_within_a_command_but_never_shared_between_commands() {
        let dir = tempfile::tempdir().unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        for index in 0..2 {
            let bridge = create_bridge(&tx, dir.path().join(format!("{index}.log")), true);
            bridge.send("8a1b2c3d4e5f Downloading 10%".into()).unwrap();
            bridge.send("8a1b2c3d4e5f Downloading 20%".into()).unwrap();
            drop(bridge);
        }
        let lines: Vec<_> = rx.try_iter().collect();
        assert_eq!(lines[0].key, lines[1].key);
        assert_eq!(lines[2].key, lines[3].key);
        assert_ne!(lines[0].key, lines[2].key);
    }

    #[cfg(windows)]
    #[test]
    fn real_failed_process_emits_each_stdout_and_stderr_line_only_once() {
        use crate::handlers::ps_script::{PsRunner, ScriptRunner};
        let dir = tempfile::tempdir().unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let bridge = create_bridge(&tx, dir.path().join("process.log"), false);
        let result = PsRunner::new().run_direct_sync(
            "cmd.exe",
            &[
                "/D",
                "/C",
                "echo stdout-marker & echo stderr-marker 1>&2 & exit /b 56",
            ],
            None,
            Some(&bridge),
        );
        let error = bridge.complete(result, "intentional-failure").unwrap_err();
        tx.send(LogLine::error(error.to_string())).unwrap();
        let lines: Vec<_> = rx.try_iter().collect();
        assert_eq!(
            lines
                .iter()
                .filter(|line| line.text.trim() == "stdout-marker")
                .count(),
            1
        );
        assert_eq!(
            lines
                .iter()
                .filter(|line| line.text.trim() == "stderr-marker")
                .count(),
            1
        );
        assert_eq!(
            lines.last().unwrap().level,
            crate::handlers::log_types::LogLevel::Error
        );
        assert!(!lines.last().unwrap().text.contains("stdout-marker"));
    }
}
