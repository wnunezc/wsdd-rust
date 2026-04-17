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
//! About dialog renderer.

use crate::app::WsddApp;
use crate::i18n::tr;
use crate::ui::ActiveView;

mod copy;

/// Renders the About modal with product, license, dependency, and tooling data.
pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    let copy = copy::about_copy(app.settings.language);
    let mut open = true;

    crate::ui::render_modal_backdrop(ctx, "about_backdrop");

    egui::Window::new("About WSDD")
        .collapsible(false)
        .resizable(true)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(720.0)
        .default_height(620.0)
        .min_width(640.0)
        .min_height(520.0)
        .open(&mut open)
        .show(ctx, |ui| {
            render_header(ui, &copy);
            ui.add_space(16.0);
            ui.separator();
            ui.add_space(12.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                render_description(ui, &copy);
                ui.add_space(14.0);
                render_info_grid(ui, &copy);
                ui.add_space(14.0);
                ui.separator();
                ui.add_space(10.0);
                render_dependencies(ui, &copy);
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);
                render_tools(ui, &copy);
                ui.add_space(24.0);
            });

            ui.separator();
            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                if ui.button(format!("  {}  ", tr("btn_close"))).clicked() {
                    app.ui.active = ActiveView::Main;
                }
            });
            ui.add_space(8.0);
        });

    if !open {
        app.ui.active = ActiveView::Main;
    }
}

fn render_header(ui: &mut egui::Ui, copy: &copy::AboutCopy) {
    ui.add_space(24.0);
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new("◈ WebStack Deployer for Docker")
                .size(22.0)
                .strong(),
        );
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION")))
                .size(15.0)
                .color(ui.visuals().hyperlink_color),
        );
        ui.label(
            egui::RichText::new(copy.edition_line)
                .size(11.0)
                .color(ui.visuals().weak_text_color()),
        );
    });
}

fn render_description(ui: &mut egui::Ui, copy: &copy::AboutCopy) {
    egui::Frame::none()
        .fill(ui.visuals().faint_bg_color)
        .inner_margin(egui::Margin::symmetric(16.0, 10.0))
        .rounding(egui::Rounding::same(6.0))
        .show(ui, |ui| {
            ui.set_max_width(560.0);
            ui.label(copy.description);
        });
}

fn render_info_grid(ui: &mut egui::Ui, copy: &copy::AboutCopy) {
    egui::Grid::new("about_info")
        .num_columns(2)
        .spacing([20.0, 6.0])
        .show(ui, |ui| {
            ui.label(egui::RichText::new(tr("about_author")).strong());
            ui.label("Walter Nunez / Icaros Net S.A");
            ui.end_row();

            ui.label(egui::RichText::new(copy.copyright_label).strong());
            ui.label("(c) 2026 Walter Nunez / Icaros Net S.A");
            ui.end_row();

            ui.label(egui::RichText::new(tr("about_license")).strong());
            ui.label(copy.license_value);
            ui.end_row();

            ui.label(egui::RichText::new(copy.platform_label).strong());
            ui.label(copy.platform_value);
            ui.end_row();

            ui.label(egui::RichText::new(copy.migrated_from_label).strong());
            ui.label(copy.migrated_from_value);
            ui.end_row();

            ui.label(egui::RichText::new(copy.gui_framework_label).strong());
            ui.label(copy.gui_framework_value);
            ui.end_row();

            ui.label(egui::RichText::new(copy.fonts_label).strong());
            ui.label(copy.fonts_value);
            ui.end_row();
        });
}

fn render_dependencies(ui: &mut egui::Ui, copy: &copy::AboutCopy) {
    ui.label(egui::RichText::new(copy.dependencies_title).strong());
    ui.add_space(6.0);

    for (name, purpose, license) in copy.dependencies {
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(format!("• {name}"))
                    .monospace()
                    .size(12.0),
            );
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(format!("— {purpose}"))
                    .color(ui.visuals().weak_text_color())
                    .size(12.0),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(*license)
                        .color(ui.visuals().hyperlink_color)
                        .size(11.0),
                );
            });
        });
    }
}

fn render_tools(ui: &mut egui::Ui, copy: &copy::AboutCopy) {
    ui.label(egui::RichText::new(copy.external_tools_title).strong());
    ui.add_space(6.0);

    for (tool, desc) in copy.tools {
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(egui::RichText::new(format!("• {tool}")).strong().size(12.0));
            ui.label(
                egui::RichText::new(format!("— {desc}"))
                    .color(ui.visuals().weak_text_color())
                    .size(12.0),
            );
        });
    }
}
