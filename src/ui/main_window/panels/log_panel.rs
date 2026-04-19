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
//! Main-window log panel rendering.

use crate::app::{ContainerLogEntry, WsddApp};
use crate::handlers::log_types::LogLevel;
use crate::i18n::tr;

use super::surface::show_surface_panel;

const LOG_FONT_SIZE: f32 = 12.5;

/// Renders the bottom log panel.
pub(super) fn render_log_panel(ctx: &egui::Context, app: &mut WsddApp) {
    let desired_height = (ctx.available_rect().height() * 0.5).max(240.0);

    egui::TopBottomPanel::bottom("log_panel")
        .default_height(desired_height)
        .min_height(220.0)
        .max_height(desired_height.max(520.0))
        .resizable(true)
        .show(ctx, |ui| {
            show_surface_panel(ui, |ui| {
                if ui.available_width() < 900.0 {
                    render_main_log(ui, app);
                    ui.add_space(10.0);
                    render_container_log(ui, app);
                } else {
                    ui.columns(2, |columns| {
                        render_main_log(&mut columns[0], app);
                        render_container_log(&mut columns[1], app);
                    });
                }
            });
        });
}

fn render_main_log(ui: &mut egui::Ui, app: &mut WsddApp) {
    let title = tr("log_title");
    let count = app.main_log.len();
    render_log_header(ui, &title, count, |ui| {
        if ui.small_button(tr("btn_clear")).clicked() {
            app.main_log.clear();
        }
        ui.add_space(4.0);
        if ui.small_button(tr("btn_copy")).clicked() {
            let text: String = app
                .main_log
                .iter()
                .map(|l| l.text.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            ui.output_mut(|o| o.copied_text = text);
        }
    });
    render_log_frame(ui, "main_log_scroll", |ui, dark| {
        for line in &app.main_log {
            render_log_line(ui, &line.text, log_color(&line.level, dark));
        }
    });
}

fn render_container_log(ui: &mut egui::Ui, app: &mut WsddApp) {
    let title = tr("container_log_title");
    let count = app.container_logs.len();
    render_log_header(ui, &title, count, |ui| {
        if ui.small_button(tr("btn_clear")).clicked() {
            app.container_logs.clear();
        }
        ui.add_space(4.0);
        if ui.small_button(tr("btn_copy")).clicked() {
            let text = container_log_text(&app.container_logs);
            ui.output_mut(|o| o.copied_text = text);
        }
    });
    render_log_frame(ui, "container_log_scroll", |ui, dark| {
        for line in &app.container_logs {
            render_container_log_line(ui, line, dark);
        }
    });
}

fn render_log_header(
    ui: &mut egui::Ui,
    title: &str,
    count: usize,
    add_actions: impl FnOnce(&mut egui::Ui),
) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(title).size(14.0).strong());
        ui.label(
            egui::RichText::new(format!("{} {}", count, tr("status_bar_logs")))
                .size(12.5)
                .color(ui.visuals().weak_text_color()),
        );
        ui.with_layout(
            egui::Layout::right_to_left(egui::Align::Center),
            add_actions,
        );
    });
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);
}

fn render_log_frame(
    ui: &mut egui::Ui,
    id: &'static str,
    add_lines: impl FnOnce(&mut egui::Ui, bool),
) {
    egui::Frame::none()
        .fill(ui.visuals().extreme_bg_color)
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt(id)
                .stick_to_bottom(true)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        let dark = ui.visuals().dark_mode;
                        add_lines(ui, dark);
                    });
                });
        });
}

fn render_log_line(ui: &mut egui::Ui, text: &str, color: egui::Color32) {
    let mut job = base_log_layout_job(ui.available_width());
    job.append(text, 0.0, log_text_format(color));
    ui.add(egui::Label::new(job).wrap());
}

fn render_container_log_line(ui: &mut egui::Ui, line: &ContainerLogEntry, dark: bool) {
    let mut job = base_log_layout_job(ui.available_width());
    job.append(
        &format!("[{}] ", line.container_name),
        0.0,
        log_text_format(container_color(&line.container_name, dark)),
    );
    job.append(
        &line.text,
        0.0,
        log_text_format(log_color(&LogLevel::Raw, dark)),
    );
    ui.add(egui::Label::new(job).wrap());
}

fn base_log_layout_job(wrap_width: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = wrap_width.max(1.0);
    job.wrap.break_anywhere = true;
    job.halign = egui::Align::LEFT;
    job.justify = false;
    job
}

fn log_text_format(color: egui::Color32) -> egui::text::TextFormat {
    egui::text::TextFormat {
        font_id: egui::FontId::monospace(LOG_FONT_SIZE),
        color,
        ..Default::default()
    }
}

fn container_log_text(lines: &[ContainerLogEntry]) -> String {
    lines
        .iter()
        .map(|line| format!("[{}] {}", line.container_name, line.text))
        .collect::<Vec<_>>()
        .join("\n")
}

fn log_color(level: &LogLevel, dark: bool) -> egui::Color32 {
    match level {
        LogLevel::Raw => {
            if dark {
                egui::Color32::from_rgb(205, 205, 205)
            } else {
                egui::Color32::from_rgb(50, 50, 50)
            }
        }
        LogLevel::Success => {
            if dark {
                egui::Color32::from_rgb(80, 200, 80)
            } else {
                egui::Color32::from_rgb(0, 130, 0)
            }
        }
        LogLevel::Warn => {
            if dark {
                egui::Color32::from_rgb(240, 180, 40)
            } else {
                egui::Color32::from_rgb(160, 80, 0)
            }
        }
        LogLevel::Error => {
            if dark {
                egui::Color32::from_rgb(220, 60, 60)
            } else {
                egui::Color32::from_rgb(180, 0, 0)
            }
        }
        LogLevel::Info => {
            if dark {
                egui::Color32::LIGHT_GRAY
            } else {
                egui::Color32::from_rgb(50, 50, 50)
            }
        }
    }
}

fn container_color(name: &str, dark: bool) -> egui::Color32 {
    const DARK_PALETTE: [egui::Color32; 10] = [
        egui::Color32::from_rgb(88, 196, 255),
        egui::Color32::from_rgb(128, 220, 112),
        egui::Color32::from_rgb(255, 196, 82),
        egui::Color32::from_rgb(255, 128, 168),
        egui::Color32::from_rgb(180, 156, 255),
        egui::Color32::from_rgb(96, 224, 196),
        egui::Color32::from_rgb(255, 150, 96),
        egui::Color32::from_rgb(164, 218, 255),
        egui::Color32::from_rgb(214, 236, 96),
        egui::Color32::from_rgb(255, 164, 220),
    ];
    const LIGHT_PALETTE: [egui::Color32; 10] = [
        egui::Color32::from_rgb(0, 88, 160),
        egui::Color32::from_rgb(20, 112, 48),
        egui::Color32::from_rgb(150, 86, 0),
        egui::Color32::from_rgb(164, 32, 84),
        egui::Color32::from_rgb(88, 64, 176),
        egui::Color32::from_rgb(0, 118, 112),
        egui::Color32::from_rgb(174, 70, 0),
        egui::Color32::from_rgb(20, 88, 142),
        egui::Color32::from_rgb(112, 114, 0),
        egui::Color32::from_rgb(142, 48, 126),
    ];

    let palette = if dark { DARK_PALETTE } else { LIGHT_PALETTE };
    let idx = stable_name_hash(name) as usize % palette.len();
    palette[idx]
}

fn stable_name_hash(name: &str) -> u32 {
    name.bytes().fold(2_166_136_261_u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(16_777_619)
    })
}
