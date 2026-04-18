use std::collections::HashSet;
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use super::{ContainerLogEntry, ContainerLogWatcher, WsddApp};
use crate::config::environment::env_config;
use crate::handlers::ps_script::strip_ansi;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

const MAX_CONTAINER_LOG_LINES: usize = 1_500;
const WATCHER_RETRY_DELAY: Duration = Duration::from_secs(5);

impl WsddApp {
    /// Drains live Docker log channels and keeps the buffer bounded.
    pub(super) fn drain_container_log_channels(&mut self) {
        while let Ok(name) = self.container_log_done_rx.try_recv() {
            self.container_log_watchers.remove(&name);
            self.container_log_retry_after
                .insert(name, Instant::now() + WATCHER_RETRY_DELAY);
        }

        while let Ok(line) = self.container_log_rx.try_recv() {
            let is_dup = self.container_logs.last().is_some_and(|last| {
                last.container_name == line.container_name && last.text == line.text
            });
            if !is_dup {
                self.container_logs.push(line);
            }
        }

        if self.container_logs.len() > MAX_CONTAINER_LOG_LINES {
            let excess = self.container_logs.len() - MAX_CONTAINER_LOG_LINES;
            self.container_logs.drain(..excess);
        }
    }

    /// Starts/stops `docker logs --follow` watchers for running WSDD containers.
    pub fn sync_container_log_watchers(&mut self, ctx: &egui::Context) {
        let running: HashSet<String> = self
            .containers
            .iter()
            .filter(|container| container.is_running())
            .map(|container| container.name.clone())
            .collect();

        let watched: Vec<String> = self.container_log_watchers.keys().cloned().collect();
        for name in watched {
            if !running.contains(&name) {
                if let Some(watcher) = self.container_log_watchers.remove(&name) {
                    let _ = watcher.stop_tx.send(());
                }
                self.container_log_retry_after.remove(&name);
            }
        }

        let now = Instant::now();
        self.container_log_retry_after
            .retain(|_, retry_at| *retry_at > now);

        for name in running {
            if self.container_log_watchers.contains_key(&name) {
                continue;
            }
            if self
                .container_log_retry_after
                .get(&name)
                .is_some_and(|retry_at| *retry_at > now)
            {
                continue;
            }

            let watcher = spawn_container_log_watcher(
                name.clone(),
                self.container_log_tx.clone(),
                self.container_log_done_tx.clone(),
                ctx.clone(),
            );
            self.container_log_watchers.insert(name, watcher);
        }
    }
}

fn spawn_container_log_watcher(
    container_name: String,
    log_tx: mpsc::Sender<ContainerLogEntry>,
    done_tx: mpsc::Sender<String>,
    ctx: egui::Context,
) -> ContainerLogWatcher {
    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let thread_name = container_name.clone();

    std::thread::spawn(move || {
        follow_container_logs(thread_name.clone(), log_tx, stop_rx, ctx);
        let _ = done_tx.send(thread_name);
    });

    ContainerLogWatcher { stop_tx }
}

fn follow_container_logs(
    container_name: String,
    log_tx: mpsc::Sender<ContainerLogEntry>,
    stop_rx: mpsc::Receiver<()>,
    ctx: egui::Context,
) {
    let mut cmd = Command::new(env_config().docker_exe());
    cmd.args(["logs", "--tail", "200", "--follow", &container_name])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let Ok(mut child) = cmd.spawn() else {
        return;
    };

    let stdout_handle = child.stdout.take().map(|mut stdout| {
        let tx = log_tx.clone();
        let name = container_name.clone();
        let repaint = ctx.clone();
        std::thread::spawn(move || read_container_log_stream(&name, &mut stdout, &tx, &repaint))
    });

    let stderr_handle = child.stderr.take().map(|mut stderr| {
        let tx = log_tx.clone();
        let name = container_name.clone();
        let repaint = ctx.clone();
        std::thread::spawn(move || read_container_log_stream(&name, &mut stderr, &tx, &repaint))
    });

    loop {
        if stop_rx.try_recv().is_ok() {
            let _ = child.kill();
            break;
        }

        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => std::thread::sleep(Duration::from_millis(250)),
            Err(_) => break,
        }
    }

    let _ = child.wait();
    if let Some(handle) = stdout_handle {
        let _ = handle.join();
    }
    if let Some(handle) = stderr_handle {
        let _ = handle.join();
    }
}

fn read_container_log_stream(
    container_name: &str,
    reader: &mut impl Read,
    tx: &mpsc::Sender<ContainerLogEntry>,
    ctx: &egui::Context,
) {
    let mut pending = Vec::new();
    let mut buf = [0_u8; 4096];

    loop {
        match reader.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(bytes_read) => {
                for byte in &buf[..bytes_read] {
                    if *byte == b'\n' || *byte == b'\r' {
                        flush_container_log_segment(container_name, &mut pending, tx, ctx);
                    } else {
                        pending.push(*byte);
                    }
                }
            }
        }
    }

    flush_container_log_segment(container_name, &mut pending, tx, ctx);
}

fn flush_container_log_segment(
    container_name: &str,
    pending: &mut Vec<u8>,
    tx: &mpsc::Sender<ContainerLogEntry>,
    ctx: &egui::Context,
) {
    if pending.is_empty() {
        return;
    }

    let raw = String::from_utf8_lossy(pending);
    let clean = strip_ansi(raw.trim_end_matches(['\n', '\r']));
    pending.clear();

    if clean.trim().is_empty() {
        return;
    }

    let _ = tx.send(ContainerLogEntry {
        container_name: container_name.to_string(),
        text: clean,
    });
    ctx.request_repaint();
}
