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

use crate::app::WsddApp;
use crate::handlers::docker::{self, ContainerInfo};
use crate::handlers::log_types::LogLine;
use crate::i18n::tr;
use crate::ui::ActiveView;

pub fn render(ui: &mut egui::Ui, app: &mut WsddApp) {
    ui.spacing_mut().item_spacing = egui::vec2(14.0, 9.0);

    if app.containers.is_empty() {
        ui.add_space(4.0);
        ui.label(tr("containers_empty"));
        return;
    }

    let containers: Vec<ContainerInfo> = app.containers.clone();

    enum PendingAction {
        ContainerOp(&'static str, String),
        Toolbox(String),
    }

    let mut pending: Option<PendingAction> = None;
    let col_name = tr("col_name");
    let col_status = tr("col_status");
    let col_start = tr("main_start");
    let col_stop = tr("main_stop");
    let col_restart = tr("main_restart");
    let col_toolbox = tr("col_toolbox");
    let start_label = format!("{} {}", '\u{25B6}', tr("main_start"));
    let stop_label = format!("{} {}", '\u{25A0}', tr("main_stop"));
    let restart_label = format!("{} {}", '\u{21BA}', tr("main_restart"));

    egui::Grid::new("containers_grid")
        .num_columns(6)
        .striped(true)
        .min_col_width(110.0)
        .spacing([18.0, 10.0])
        .show(ui, |ui| {
            ui.strong(col_name);
            ui.strong(col_status);
            ui.strong(col_start);
            ui.strong(col_stop);
            ui.strong(col_restart);
            ui.strong(col_toolbox.clone());
            ui.end_row();

            for container in &containers {
                let running = container.is_running();
                let container_busy = app.is_job_running(&container_job_key(&container.name));

                ui.label(egui::RichText::new(&container.name).strong());

                if running {
                    ui.colored_label(egui::Color32::from_rgb(80, 200, 80), &container.status);
                } else {
                    ui.colored_label(egui::Color32::from_rgb(200, 80, 80), &container.status);
                }

                let start_btn =
                    ui.add_enabled(!running && !container_busy, egui::Button::new(&start_label));
                if start_btn.clicked() && pending.is_none() {
                    pending = Some(PendingAction::ContainerOp("start", container.name.clone()));
                }

                let stop_btn =
                    ui.add_enabled(running && !container_busy, egui::Button::new(&stop_label));
                if stop_btn.clicked() && pending.is_none() {
                    pending = Some(PendingAction::ContainerOp("stop", container.name.clone()));
                }

                let restart_btn =
                    ui.add_enabled(!container_busy, egui::Button::new(&restart_label));
                if restart_btn.clicked() && pending.is_none() {
                    pending = Some(PendingAction::ContainerOp(
                        "restart",
                        container.name.clone(),
                    ));
                }

                if ui.button(&col_toolbox).clicked() && pending.is_none() {
                    pending = Some(PendingAction::Toolbox(container.name.clone()));
                }

                ui.end_row();
            }
        });

    match pending {
        Some(PendingAction::ContainerOp(action, name)) => {
            spawn_container_op(ui.ctx(), app, action, name);
        }
        Some(PendingAction::Toolbox(name)) => {
            app.ui.toolbox_container_name = Some(name);
            app.ui.active = ActiveView::ToolboxContainer;
        }
        None => {}
    }
}

fn spawn_container_op(ctx: &egui::Context, app: &mut WsddApp, action: &'static str, name: String) {
    let tx = app.main_log_tx.clone();
    let job_key = container_job_key(&name);
    let label = format!("Container {} {}", action, name);

    let _ = app.spawn_async_job(ctx, job_key, label, async move {
        let _ = tx.send(LogLine::info(format!("[Docker] {action} -> {name}...")));

        let result = match action {
            "start" => docker::start_container(&name, None).await,
            "stop" => docker::stop_container(&name, None).await,
            "restart" => docker::restart_container(&name, None).await,
            _ => Ok(()),
        };

        match result {
            Ok(()) => {
                let _ = tx.send(LogLine::success(format!("[Docker] {name} - {action} OK")));
                Ok(())
            }
            Err(e) => {
                let message = format!("[Docker] {name} - error: {e}");
                let _ = tx.send(LogLine::error(message.clone()));
                Err(message)
            }
        }
    });
}

fn container_job_key(name: &str) -> String {
    format!("container:{name}")
}
