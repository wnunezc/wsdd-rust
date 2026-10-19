//! Per-deployment diagnostic capture and operation progress events.

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::config::environment::path_config;
use crate::errors::InfraError;
use crate::handlers::diagnostics;
use crate::handlers::log_types::{LogLine, LogSender};

static NEXT_LOG: AtomicU64 = AtomicU64::new(1);

/// A structured update for the deployment summary shown in the UI.
#[derive(Debug)]
pub enum TraceEvent {
    StageStarted {
        key: String,
        label: &'static str,
        at: Instant,
    },
    StageFinished {
        key: String,
        label: &'static str,
        at: Instant,
        success: bool,
    },
    Buildkit {
        key: String,
        step: String,
    },
    OperationFinished {
        key: String,
        success: bool,
    },
}

/// Sends stage boundaries without parsing human-facing log messages.
#[derive(Clone)]
pub struct TraceHandle {
    key: String,
    events: Sender<TraceEvent>,
}

impl TraceHandle {
    pub fn new(key: String, events: Sender<TraceEvent>) -> Self {
        Self { key, events }
    }

    /// Runs one real operation stage and reports its measured outcome.
    pub fn stage<T>(
        &self,
        label: &'static str,
        log: &LogSender,
        task: impl FnOnce() -> Result<T, InfraError>,
    ) -> Result<T, InfraError> {
        let started = Instant::now();
        let _ = self.events.send(TraceEvent::StageStarted {
            key: self.key.clone(),
            label,
            at: started,
        });
        let _ = log.send(LogLine::info(format!("[{label}] Iniciando...")));
        let result = task();
        let success = result.is_ok();
        let elapsed = started.elapsed().as_secs_f32();
        let _ = self.events.send(TraceEvent::StageFinished {
            key: self.key.clone(),
            label,
            at: Instant::now(),
            success,
        });
        let line = if success {
            LogLine::success(format!("[{label}] Completado en {elapsed:.1} s"))
        } else {
            LogLine::error(format!("[{label}] Fallo tras {elapsed:.1} s"))
        };
        let _ = log.send(line);
        result
    }
}

/// Creates a unique path without using a project name in the filename.
pub fn operation_log_path() -> PathBuf {
    let tick = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    let sequence = NEXT_LOG.fetch_add(1, Ordering::Relaxed);
    path_config()
        .logs_dir()
        .join(format!("wsdd-operation-{tick}-{sequence}.log"))
}

/// Records all UI lines for one operation while preserving the existing daily bridge log.
pub fn run_logged<T>(
    key: String,
    path: PathBuf,
    main_log: LogSender,
    events: Sender<TraceEvent>,
    task: impl FnOnce(&LogSender, &TraceHandle) -> Result<T, String>,
) -> Result<T, String> {
    let (tx, rx) = mpsc::channel::<LogLine>();
    let writer_main = main_log.clone();
    let writer_events = events.clone();
    let writer_key = key.clone();
    let writer = std::thread::spawn(move || {
        let mut file = match path.parent().map(fs::create_dir_all) {
            Some(Ok(())) => match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => Some(file),
                Err(error) => {
                    let _ = writer_main.send(LogLine::warn(format!(
                        "No se pudo abrir el log de esta operación: {error}"
                    )));
                    None
                }
            },
            Some(Err(error)) => {
                let _ = writer_main.send(LogLine::warn(format!(
                    "No se pudo crear el directorio del log: {error}"
                )));
                None
            }
            None => None,
        };
        let mut build_steps = HashMap::new();
        for mut line in rx {
            line.text = diagnostics::redact(&line.text);
            if let Some(step) = buildkit_step(&line.text, &mut build_steps) {
                let _ = writer_events.send(TraceEvent::Buildkit {
                    key: writer_key.clone(),
                    step,
                });
            }
            if let Some(output) = file.as_mut() {
                if writeln!(output, "{}", line.text).is_err() {
                    file = None;
                    let _ = writer_main.send(LogLine::warn(
                        "El log de esta operación quedó incompleto por un error de escritura.",
                    ));
                }
            }
            let _ = writer_main.send(line);
        }
    });
    let trace = TraceHandle::new(key.clone(), events.clone());
    let result = task(&tx, &trace);
    drop(trace);
    drop(tx);
    if writer.join().is_err() {
        let _ = main_log.send(LogLine::warn(
            "El registro de esta operación terminó inesperadamente.",
        ));
    }
    let _ = events.send(TraceEvent::OperationFinished {
        key,
        success: result.is_ok(),
    });
    result
}

fn buildkit_step(line: &str, known: &mut HashMap<String, String>) -> Option<String> {
    let line = line.trim_start();
    let (id, rest) = line.split_once(' ')?;
    if !id.starts_with('#') || !id[1..].bytes().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    let rest = rest.trim_start();
    if let Some(bracket) = rest
        .strip_prefix('[')
        .and_then(|value| value.split_once(']'))
    {
        let numbers = bracket.0.trim();
        if let Some((current, total)) = numbers.split_once('/') {
            let (Ok(current), Ok(total)) =
                (current.trim().parse::<u32>(), total.trim().parse::<u32>())
            else {
                return None;
            };
            let label = format!("BuildKit paso {current}/{total} ({id})");
            known.insert(id.to_string(), label.clone());
            return Some(format!("{label}: en curso"));
        }
    }
    let status = if rest.starts_with("DONE") {
        "completado"
    } else if rest.starts_with("CACHED") {
        "caché"
    } else if rest.starts_with("ERROR") {
        "error"
    } else {
        return None;
    };
    known.get(id).map(|label| format!("{label}: {status}"))
}

#[cfg(test)]
mod tests {
    use super::{buildkit_step, run_logged, TraceEvent};
    use crate::errors::InfraError;
    use crate::handlers::log_types::LogLine;
    use std::collections::HashMap;
    use std::sync::mpsc;

    #[test]
    fn buildkit_reports_observed_step_without_exposing_run_arguments() {
        let mut known = HashMap::new();
        assert_eq!(
            buildkit_step("#5 [ 1/25] RUN PASSWORD=synthetic-secret", &mut known),
            Some("BuildKit paso 1/25 (#5): en curso".into())
        );
        assert_eq!(
            buildkit_step("#5 DONE 0.5s", &mut known),
            Some("BuildKit paso 1/25 (#5): completado".into())
        );
        assert_eq!(buildkit_step("#6 ERROR: build failed", &mut known), None);
    }

    #[test]
    fn failed_operation_keeps_full_redacted_progress_file_and_stage_events() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("operation.log");
        let (main_tx, main_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();
        let result: Result<(), String> = run_logged(
            "fixture".into(),
            path.clone(),
            main_tx,
            event_tx,
            |log, trace| {
                trace
                    .stage("Construcción", log, || {
                        let _ = log.send(LogLine::raw("#5 [ 1/25] RUN PASSWORD=fixture-secret"));
                        let _ = log.send(LogLine::raw("#5 ERROR: build failed"));
                        Err(InfraError::PrerequisiteNotMet("fixture failure".into()))
                    })
                    .map_err(|error| error.to_string())
            },
        );
        assert!(result.is_err());
        let contents = std::fs::read_to_string(path).unwrap();
        assert!(contents.contains("#5 [ 1/25]"));
        assert!(contents.contains("#5 ERROR: build failed"));
        assert!(!contents.contains("fixture-secret"));
        assert!(main_rx.try_iter().count() >= 4);
        let events: Vec<_> = event_rx.try_iter().collect();
        assert!(events.iter().any(|event| matches!(event, TraceEvent::Buildkit { step, .. } if step == "BuildKit paso 1/25 (#5): error")));
        assert!(matches!(
            events.last(),
            Some(TraceEvent::OperationFinished { success: false, .. })
        ));
    }
}
