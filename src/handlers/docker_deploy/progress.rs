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
            let _ = log_tx.send(LogLine::raw(line));
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
                        LogLine::raw(format!("  {hash}  {rest}")).with_key(docker_layer_key(hash)),
                    );
                    let cat = layer_status_category(rest).to_string();
                    let prev = layer_cats.entry(hash.to_string()).or_default();
                    if *prev != cat {
                        *prev = cat;
                        if let Some(ref mut f) = file {
                            let _ = writeln!(f, "{trimmed}");
                        }
                    }
                } else if let Some(key) = parse_buildkit_progress_key(trimmed) {
                    let _ = log_tx.send(LogLine::raw(trimmed.to_string()).with_key(key));
                    if let Some(ref mut f) = file {
                        let _ = writeln!(f, "{trimmed}");
                    }
                } else {
                    let _ = log_tx.send(LogLine::raw(trimmed.to_string()));
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
    fn layer_status_category_groups_download_progress() {
        assert_eq!(layer_status_category("Downloading [====>]"), "downloading");
        assert_eq!(layer_status_category("Download complete"), "downloaded");
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
}
