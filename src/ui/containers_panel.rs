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
//! Panel de contenedores Docker.
//!
//! Muestra la tabla de contenedores WSDD con botones Start/Stop/Restart/Toolbox por fila.
//! Equivalente a la sección de contenedores en `Forms/Main.cs`.

use crate::app::WsddApp;
use crate::handlers::docker::{ContainerInfo, restart_container_sync, start_container_sync, stop_container_sync};
use crate::handlers::log_types::LogLine;
use crate::ui::ActiveView;

/// Renderiza la tabla de contenedores.
pub fn render(ui: &mut egui::Ui, app: &mut WsddApp) {
    if app.containers.is_empty() {
        ui.label("No hay contenedores WSDD detectados.");
        return;
    }

    let containers: Vec<ContainerInfo> = app.containers.clone();

    enum PendingAction {
        ContainerOp(&'static str, String),
        Toolbox(String),
    }
    let mut pending: Option<PendingAction> = None;

    egui::Grid::new("containers_grid")
        .num_columns(6)
        .striped(true)
        .min_col_width(110.0)
        .show(ui, |ui| {
            ui.strong("Nombre");
            ui.strong("Estado");
            ui.strong("Start");
            ui.strong("Stop");
            ui.strong("Restart");
            ui.strong("Toolbox");
            ui.end_row();

            for c in &containers {
                let running = c.is_running();

                ui.label(&c.name);

                if running {
                    ui.colored_label(egui::Color32::from_rgb(80, 200, 80), &c.status);
                } else {
                    ui.colored_label(egui::Color32::from_rgb(200, 80, 80), &c.status);
                }

                let start_btn = ui.add_enabled(!running, egui::Button::new("▶ Start"));
                if start_btn.clicked() && pending.is_none() {
                    pending = Some(PendingAction::ContainerOp("start", c.name.clone()));
                }

                let stop_btn = ui.add_enabled(running, egui::Button::new("■ Stop"));
                if stop_btn.clicked() && pending.is_none() {
                    pending = Some(PendingAction::ContainerOp("stop", c.name.clone()));
                }

                if ui.button("↺ Restart").clicked() && pending.is_none() {
                    pending = Some(PendingAction::ContainerOp("restart", c.name.clone()));
                }

                if ui.button("Toolbox").clicked() && pending.is_none() {
                    pending = Some(PendingAction::Toolbox(c.name.clone()));
                }

                ui.end_row();
            }
        });

    match pending {
        Some(PendingAction::ContainerOp(action, name)) => {
            spawn_container_op(app, action, name);
        }
        Some(PendingAction::Toolbox(name)) => {
            app.ui.toolbox_container_name = Some(name);
            app.ui.active = ActiveView::ToolboxContainer;
        }
        None => {}
    }
}

/// Lanza una operación de contenedor en un background thread.
fn spawn_container_op(app: &mut WsddApp, action: &'static str, name: String) {
    let runner = app.runner.clone();
    let tx = app.main_log_tx.clone();

    let _ = tx.send(LogLine::info(format!("[Docker] {action} → {name}...")));

    std::thread::spawn(move || {
        let result = match action {
            "start" => start_container_sync(&runner, &name),
            "stop" => stop_container_sync(&runner, &name),
            "restart" => restart_container_sync(&runner, &name),
            _ => Ok(()),
        };

        match result {
            Ok(()) => {
                let _ = tx.send(LogLine::success(format!("[Docker] {name} — {action} OK ✓")));
            }
            Err(e) => {
                let _ = tx.send(LogLine::error(format!("[Docker] {name} — error: {e}")));
            }
        }
    });
}
