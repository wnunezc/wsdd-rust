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
//! Main-window actions and background jobs.

use std::sync::mpsc;

use crate::app::WsddApp;
use crate::handlers::docker;
use crate::handlers::log_types::LogLine;
use crate::i18n::tr;

use super::POLL_INTERVAL;

/// Starts a container poll when the interval elapsed and no poll is active.
pub(super) fn poll_containers(ctx: &egui::Context, app: &mut WsddApp) {
    if app.container_poll_active {
        return;
    }
    if app.last_container_poll.elapsed() < POLL_INTERVAL {
        return;
    }
    start_poll(ctx, app);
}

/// Forces an immediate container poll unless another poll is active.
pub(super) fn force_poll(ctx: &egui::Context, app: &mut WsddApp) {
    if app.container_poll_active {
        return;
    }
    start_poll(ctx, app);
}

/// Restarts Docker Desktop through the central job coordinator.
pub(super) fn start_docker_restart(ctx: &egui::Context, app: &mut WsddApp) {
    let tx = app.main_log_tx.clone();
    let runner = app.runner.clone();
    let _ = app.spawn_async_job(
        ctx,
        "lifecycle:docker",
        "Restart Docker Desktop",
        async move {
            let _ = tx.send(LogLine::info("[Docker] Reiniciando Docker Desktop..."));
            docker::restart(&runner, None).await.map_err(|e| {
                let message = format!("[Lifecycle] Error reiniciando Docker Desktop: {e}");
                let _ = tx.send(LogLine::error(message.clone()));
                message
            })?;
            let _ = tx.send(LogLine::success("[Docker] Docker Desktop reiniciado."));
            Ok(())
        },
    );
}

/// Restarts WSL through the central job coordinator.
pub(super) fn start_wsl_restart(ctx: &egui::Context, app: &mut WsddApp) {
    let tx = app.main_log_tx.clone();
    let runner = app.runner.clone();
    let _ = app.spawn_async_job(ctx, "lifecycle:wsl", "Restart WSL", async move {
        let _ = tx.send(LogLine::info("[WSL] Reiniciando servicios WSL..."));
        docker::restart_wsl(&runner, None).await.map_err(|e| {
            let message = format!("[Lifecycle] Error reiniciando WSL: {e}");
            let _ = tx.send(LogLine::error(message.clone()));
            message
        })?;
        let _ = tx.send(LogLine::success("[WSL] Servicios WSL reiniciados."));
        Ok(())
    });
}

/// Shuts WSL down through the central job coordinator.
pub(super) fn start_wsl_shutdown(ctx: &egui::Context, app: &mut WsddApp) {
    let tx = app.main_log_tx.clone();
    let runner = app.runner.clone();
    let _ = app.spawn_async_job(ctx, "lifecycle:wsl", "Shutdown WSL", async move {
        let _ = tx.send(LogLine::info("[WSL] Apagando WSL por completo..."));
        docker::stop_wsl(&runner, None).await.map_err(|e| {
            let message = format!("[Lifecycle] Error apagando WSL: {e}");
            let _ = tx.send(LogLine::error(message.clone()));
            message
        })?;
        let _ = tx.send(LogLine::success("[WSL] WSL apagado."));
        Ok(())
    });
}

/// Starts WSL through the central job coordinator.
pub(super) fn start_wsl_start(ctx: &egui::Context, app: &mut WsddApp) {
    let tx = app.main_log_tx.clone();
    let runner = app.runner.clone();
    let _ = app.spawn_async_job(ctx, "lifecycle:wsl", "Start WSL", async move {
        let _ = tx.send(LogLine::info("[WSL] Iniciando servicios WSL..."));
        docker::start_wsl(&runner, None).await.map_err(|e| {
            let message = format!("[Lifecycle] Error iniciando WSL: {e}");
            let _ = tx.send(LogLine::error(message.clone()));
            message
        })?;
        let _ = tx.send(LogLine::success("[WSL] Servicios WSL iniciados."));
        Ok(())
    });
}

/// Reloads project metadata from disk into the app state.
pub(super) fn reload_projects(app: &mut WsddApp) {
    match crate::handlers::project::list_all() {
        Ok(list) => app.projects = list,
        Err(e) => {
            let _ = app
                .main_log_tx
                .send(crate::handlers::log_types::LogLine::error(format!(
                    "[Proyectos] Error al cargar: {e}"
                )));
        }
    }
}

/// Starts a full environment backup job after selecting a destination ZIP.
pub(super) fn start_environment_backup(ctx: &egui::Context, app: &mut WsddApp) {
    let Some(path) = rfd::FileDialog::new()
        .set_title(tr("backup_dialog_save_title"))
        .add_filter("WSDD Backup", &["zip"])
        .set_file_name(crate::handlers::backup::default_full_backup_name())
        .save_file()
    else {
        return;
    };

    let tx = app.main_log_tx.clone();
    let runner = app.runner.clone();
    let _ = app.spawn_blocking_job(ctx, "backup:environment", "Backup environment", move || {
        crate::handlers::backup::backup_environment(&path, &runner, &tx).map_err(|e| {
            let message = format!("[Backup] Error: {e}");
            let _ = tx.send(LogLine::error(message.clone()));
            message
        })
    });
}

/// Starts a full environment restore job after selecting a source ZIP.
pub(super) fn start_environment_restore(ctx: &egui::Context, app: &mut WsddApp) {
    let Some(path) = rfd::FileDialog::new()
        .set_title(tr("backup_dialog_restore_title"))
        .add_filter("WSDD Backup", &["zip"])
        .pick_file()
    else {
        return;
    };

    let tx = app.main_log_tx.clone();
    let runner = app.runner.clone();
    let _ = app.spawn_blocking_job(
        ctx,
        "restore:environment",
        "Restore environment",
        move || {
            crate::handlers::backup::restore_environment(&path, &runner, &tx).map_err(|e| {
                let message = format!("[Restore] Error: {e}");
                let _ = tx.send(LogLine::error(message.clone()));
                message
            })
        },
    );
}

/// Starts a project restore job after selecting a project backup ZIP.
pub(super) fn start_project_restore(ctx: &egui::Context, app: &mut WsddApp) {
    let Some(path) = rfd::FileDialog::new()
        .set_title(tr("project_restore_dialog_title"))
        .add_filter("WSDD Backup", &["zip"])
        .pick_file()
    else {
        return;
    };

    let tx = app.main_log_tx.clone();
    let runner = app.runner.clone();
    let _ = app.spawn_blocking_job(
        ctx,
        "restore:project",
        "Restore project backup",
        move || {
            crate::handlers::backup::restore_project(&path, &runner, &tx).map_err(|e| {
                let message = format!("[Restore] Error: {e}");
                let _ = tx.send(LogLine::error(message.clone()));
                message
            })?;
            Ok(())
        },
    );
}

fn start_poll(ctx: &egui::Context, app: &mut WsddApp) {
    let (tx, rx) = mpsc::channel();
    app.container_poll_rx = Some(rx);
    app.container_poll_active = true;

    let runner = app.runner.clone();
    let started = app.spawn_async_job(ctx, "poll:containers", "Poll containers", async move {
        let snapshot = docker::gather_poll_snapshot(&runner).await;
        let _ = tx.send(snapshot);
        Ok(())
    });

    if !started {
        app.container_poll_rx = None;
        app.container_poll_active = false;
    }
}
