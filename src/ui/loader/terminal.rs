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
//! Visible terminal presentation for first-run requirements setup.

use egui::{Color32, Frame, Layout, Margin, RichText, ScrollArea};

use crate::{app::WsddApp, handlers::log_types::LogLevel, i18n::tr, ui::ActiveView};

const TERMINAL_FONT_SIZE: f32 = 12.5;

/// Renders the visible requirements terminal and completion actions.
pub(super) fn render(ctx: &egui::Context, app: &mut WsddApp) {
    let title = tr("loader_system_requirements");
    let processing = tr("loader_processing");
    let copy_label = tr("loader_copy_log");

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical(|ui| {
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.heading(RichText::new(&title).size(20.0).strong());
            });
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            let bottom_bar_height = if app.loader_done { 92.0 } else { 8.0 };
            let scroll_height = (ui.available_height() - bottom_bar_height).max(140.0);

            let dark = ui.visuals().dark_mode;
            let frame_fill = if dark {
                Color32::from_rgb(18, 18, 18)
            } else {
                Color32::from_rgb(240, 240, 240)
            };
            Frame::none()
                .fill(frame_fill)
                .inner_margin(Margin::same(8.0))
                .show(ui, |ui| {
                    ScrollArea::vertical()
                        .max_height(scroll_height)
                        .auto_shrink([false, false])
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for line in &app.ui.requirement_log {
                                ui.label(
                                    RichText::new(&line.text)
                                        .color(level_color(&line.level, dark))
                                        .font(egui::FontId::monospace(TERMINAL_FONT_SIZE)),
                                );
                            }

                            if !app.loader_done {
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.label(
                                        RichText::new(format!(" {processing}"))
                                            .color(Color32::GRAY)
                                            .font(egui::FontId::monospace(TERMINAL_FONT_SIZE)),
                                    );
                                });
                            }
                        });
                });

            if app.loader_done {
                ui.separator();
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    render_copy_button(ui, app, &copy_label);
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        render_action_buttons(ui, app);
                    });
                });
                ui.add_space(8.0);
            }
        });
    });

    if !app.loader_done {
        ctx.request_repaint();
    }
}

fn render_copy_button(ui: &mut egui::Ui, app: &mut WsddApp, copy_label: &str) {
    let copy_button =
        egui::Button::new(RichText::new(format!("⎘ {copy_label}")).size(16.0).strong())
            .min_size(egui::vec2(150.0, 40.0));
    if ui.add(copy_button).clicked() {
        let text: String = app
            .ui
            .requirement_log
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        ui.output_mut(|o| o.copied_text = text);
    }
}

fn render_action_buttons(ui: &mut egui::Ui, app: &mut WsddApp) {
    let button_size = egui::vec2(190.0, 44.0);

    if app.loader_error {
        let exit_button = egui::Button::new(
            RichText::new(format!("✗ {}", tr("menu_exit")))
                .size(17.0)
                .strong(),
        )
        .min_size(button_size);
        if ui.add(exit_button).clicked() {
            std::process::exit(0);
        }
    } else {
        let open_button = egui::Button::new(
            RichText::new(format!("➜ {}", tr("loader_open_wsdd")))
                .size(17.0)
                .strong(),
        )
        .min_size(button_size);
        if ui.add(open_button).clicked() {
            app.ui.active = ActiveView::Main;
        }
    }
}

fn level_color(level: &LogLevel, dark: bool) -> Color32 {
    if dark {
        match level {
            LogLevel::Raw | LogLevel::Info => Color32::from_rgb(200, 200, 200),
            LogLevel::Success => Color32::from_rgb(100, 220, 100),
            LogLevel::Warn => Color32::from_rgb(255, 200, 0),
            LogLevel::Error => Color32::from_rgb(255, 80, 80),
        }
    } else {
        match level {
            LogLevel::Raw | LogLevel::Info => Color32::from_rgb(50, 50, 50),
            LogLevel::Success => Color32::from_rgb(0, 130, 0),
            LogLevel::Warn => Color32::from_rgb(160, 80, 0),
            LogLevel::Error => Color32::from_rgb(180, 0, 0),
        }
    }
}
