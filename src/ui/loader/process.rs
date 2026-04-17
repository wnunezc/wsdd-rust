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
//! Process and channel handling for the requirements loader.

use std::sync::mpsc;

use crate::{
    app::WsddApp,
    handlers::{
        log_types::LogLine,
        requirements::{run_requirements, LoaderOutcome},
    },
};

/// Starts the background requirements job and wires its log/result channels.
pub(super) fn start_requirements(ctx: &egui::Context, app: &mut WsddApp) {
    let (log_tx, log_rx) = mpsc::channel::<LogLine>();
    let (outcome_tx, outcome_rx) = mpsc::channel::<LoaderOutcome>();

    app.requirement_rx = Some(log_rx);
    app.loader_outcome_rx = Some(outcome_rx);

    let first_run = !app.loader_silent;
    let _ = app.spawn_blocking_job(
        ctx,
        "requirements:loader",
        "Loader requirements",
        move || {
            run_requirements(log_tx, outcome_tx, first_run);
            Ok(())
        },
    );
}

/// Drains pending log lines, replacing keyed progress lines in place.
pub(super) fn drain_log(app: &mut WsddApp) {
    if let Some(rx) = &app.requirement_rx {
        while let Ok(line) = rx.try_recv() {
            if let Some(key) = line.key.clone() {
                if let Some(existing) = app
                    .ui
                    .requirement_log
                    .iter_mut()
                    .find(|l| l.key.as_deref() == Some(key.as_str()))
                {
                    *existing = line;
                } else {
                    app.ui.requirement_log.push(line);
                }
            } else {
                app.ui.requirement_log.push(line);
            }
        }
    }
}

/// Applies a completed loader outcome once.
pub(super) fn handle_outcome(app: &mut WsddApp) {
    if app.loader_done {
        return;
    }
    if let Some(rx) = &app.loader_outcome_rx {
        if let Ok(outcome) = rx.try_recv() {
            app.loader_done = true;
            match outcome {
                LoaderOutcome::BlockingError => app.loader_error = true,
                LoaderOutcome::AllDone => {
                    app.settings.setup_completed = true;
                    app.first_run = false;
                    app.loader_silent = true;
                }
                LoaderOutcome::DoneWithContinue => {
                    app.settings.setup_completed = true;
                    app.first_run = false;
                    app.loader_silent = false;
                }
            }
        }
    }
}
