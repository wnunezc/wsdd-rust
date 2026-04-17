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

use std::collections::HashMap;
use std::io::Write;

use crate::config::environment::path_config;
use crate::handlers::log_types::{LogLine, LogSender};
use crate::handlers::ps_script::OutputSender;

/// Creates an output bridge that forwards each process line into the UI log.
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
pub fn make_docker_progress_bridge(tx: &LogSender) -> OutputSender {
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
