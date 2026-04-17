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

use crate::app::WsddApp;
use crate::handlers::log_types::LogLevel;
use crate::i18n::tr;

use super::surface::show_surface_panel;

const LOG_FONT_SIZE: f32 = 12.5;

/// Renders the bottom log panel.
pub(super) fn render_log_panel(ctx: &egui::Context, app: &mut WsddApp) {
    let log_title = tr("log_title");
    let clear_label = tr("btn_clear");
    let copy_label = tr("btn_copy");
    let desired_height = (ctx.available_rect().height() * 0.5).max(240.0);

    egui::TopBottomPanel::bottom("log_panel")
        .default_height(desired_height)
        .min_height(220.0)
        .max_height(desired_height.max(520.0))
        .resizable(true)
        .show(ctx, |ui| {
            show_surface_panel(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(log_title).size(14.0).strong());
                    ui.label(
                        egui::RichText::new(format!(
                            "{} {}",
                            app.main_log.len(),
                            tr("status_bar_logs")
                        ))
                        .size(12.5)
                        .color(ui.visuals().weak_text_color()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button(&clear_label).clicked() {
                            app.main_log.clear();
                        }
                        ui.add_space(4.0);
                        if ui.small_button(&copy_label).clicked() {
                            let text: String = app
                                .main_log
                                .iter()
                                .map(|l| l.text.as_str())
                                .collect::<Vec<_>>()
                                .join("\n");
                            ui.output_mut(|o| o.copied_text = text);
                        }
                    });
                });
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                egui::Frame::none()
                    .fill(ui.visuals().extreme_bg_color)
                    .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .stick_to_bottom(true)
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                let dark = ui.visuals().dark_mode;
                                for line in &app.main_log {
                                    let color = log_color(&line.level, dark);
                                    ui.label(
                                        egui::RichText::new(&line.text)
                                            .font(egui::FontId::monospace(LOG_FONT_SIZE))
                                            .color(color),
                                    );
                                }
                            });
                    });
            });
        });
}

fn log_color(level: &LogLevel, dark: bool) -> egui::Color32 {
    match level {
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
