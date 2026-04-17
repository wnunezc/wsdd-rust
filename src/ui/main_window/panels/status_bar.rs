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
//! Main-window status bar rendering.

use crate::app::WsddApp;
use crate::i18n::tr;

/// Renders the bottom status bar.
pub(super) fn render_status_bar(ctx: &egui::Context, app: &mut WsddApp) {
    let total_containers = app.containers.len();
    let running_containers = app.containers.iter().filter(|c| c.is_running()).count();
    let projects = app.projects.len();
    let logs = app.main_log.len();

    let docker_value = if app.docker_status.daemon_ready {
        tr("status_ready")
    } else if app.container_poll_active {
        tr("status_polling")
    } else {
        tr("status_unavailable")
    };
    let docker_color = if app.docker_status.daemon_ready {
        egui::Color32::from_rgb(80, 200, 80)
    } else if app.container_poll_active {
        ctx.style().visuals.hyperlink_color
    } else {
        ctx.style().visuals.error_fg_color
    };

    let cpu_value = app
        .docker_status
        .cpu_percent
        .map(|value| format!("{value:.1}%"))
        .unwrap_or_else(|| "—".to_string());
    let cpu_color = match app.docker_status.cpu_percent {
        Some(value) if value >= 70.0 => ctx.style().visuals.error_fg_color,
        Some(value) if value >= 35.0 => ctx.style().visuals.warn_fg_color,
        Some(_) => egui::Color32::from_rgb(80, 200, 80),
        None => ui_text_color(ctx),
    };

    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(36.0)
        .show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(10.0, 4.0);

                status_item(
                    ui,
                    &tr("status_bar_containers"),
                    &total_containers.to_string(),
                    None,
                );
                status_separator(ui);
                status_item(
                    ui,
                    &tr("status_bar_running"),
                    &running_containers.to_string(),
                    Some(egui::Color32::from_rgb(80, 200, 80)),
                );
                status_separator(ui);
                status_item(ui, &tr("status_bar_projects"), &projects.to_string(), None);
                status_separator(ui);
                status_item(ui, &tr("status_bar_logs"), &logs.to_string(), None);
                status_separator(ui);

                let docker_response = status_item(
                    ui,
                    &tr("status_bar_docker"),
                    &docker_value,
                    Some(docker_color),
                );
                if let Some(process_name) = &app.docker_status.process_name {
                    docker_response.on_hover_text(format!(
                        "{} process(es): {}",
                        app.docker_status.process_count, process_name
                    ));
                }

                status_separator(ui);
                status_item(ui, &tr("status_bar_cpu"), &cpu_value, Some(cpu_color));
                status_separator(ui);
                status_item(
                    ui,
                    &tr("status_bar_memory"),
                    &format_memory_mb(app.docker_status.memory_mb),
                    None,
                );
            });
        });
}

fn status_item(
    ui: &mut egui::Ui,
    label: &str,
    value: &str,
    accent: Option<egui::Color32>,
) -> egui::Response {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("{label}:"))
                .size(13.5)
                .strong()
                .color(status_label_color(ui.ctx())),
        );
        let mut rich = egui::RichText::new(value).size(13.5).strong();
        if let Some(color) = accent {
            rich = rich.color(color);
        }
        ui.label(rich);
    })
    .response
}

fn status_separator(ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("|")
            .size(13.5)
            .strong()
            .color(ui.visuals().widgets.noninteractive.bg_stroke.color),
    );
}

fn ui_text_color(ctx: &egui::Context) -> egui::Color32 {
    ctx.style().visuals.widgets.noninteractive.fg_stroke.color
}

fn status_label_color(ctx: &egui::Context) -> egui::Color32 {
    if ctx.style().visuals.dark_mode {
        egui::Color32::from_rgb(220, 220, 220)
    } else {
        egui::Color32::from_rgb(70, 70, 70)
    }
}

fn format_memory_mb(memory_mb: Option<u64>) -> String {
    match memory_mb {
        Some(value) if value >= 1024 => format!("{:.1} GB", value as f32 / 1024.0),
        Some(value) => format!("{value} MB"),
        None => "—".to_string(),
    }
}
