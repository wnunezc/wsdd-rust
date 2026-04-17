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
//! Central containers/projects layout rendering.

use crate::app::WsddApp;
use crate::i18n::tr;
use crate::ui::{containers_panel, projects_panel};

use super::surface::show_surface_panel;

/// Renders the central containers/projects layout.
pub(super) fn render_center(ctx: &egui::Context, app: &mut WsddApp) {
    let containers = tr("main_containers");
    let projects = tr("main_projects");

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);

        if ui.available_width() < 900.0 {
            render_main_section(ui, "containers", &containers, |ui| {
                containers_panel::render(ui, app);
            });
            render_main_section(ui, "projects", &projects, |ui| {
                projects_panel::render(ui, app);
            });
        } else {
            ui.columns(2, |columns| {
                render_main_section(&mut columns[0], "containers", &containers, |ui| {
                    containers_panel::render(ui, app);
                });
                render_main_section(&mut columns[1], "projects", &projects, |ui| {
                    projects_panel::render(ui, app);
                });
            });
        }
    });
}

fn render_main_section<Contents>(
    ui: &mut egui::Ui,
    scroll_id: &'static str,
    title: &str,
    add_contents: Contents,
) where
    Contents: FnOnce(&mut egui::Ui),
{
    show_surface_panel(ui, |ui| {
        ui.label(egui::RichText::new(title).size(14.0).strong());
        ui.add_space(6.0);
        ui.separator();
        ui.add_space(6.0);

        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
            .show(ui, |ui| {
                let available = ui.available_size();
                ui.set_min_size(available);
                egui::ScrollArea::both()
                    .id_salt(("main_section_scroll", scroll_id))
                    .auto_shrink([false; 2])
                    .show(ui, add_contents);
            });
    });
}
