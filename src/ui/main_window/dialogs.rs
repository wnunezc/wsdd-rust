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
//! Main-window modal dialogs.

use crate::app::WsddApp;
use crate::i18n::{tr, trf};
use crate::ui::projects_panel;

/// Renders the project removal confirmation dialog.
pub(super) fn render_confirm_dialog(ctx: &egui::Context, app: &mut WsddApp) {
    let name = match app.ui.confirm_remove_project.clone() {
        Some(n) => n,
        None => return,
    };
    let title = tr("confirm_delete_title");
    let body = trf("confirm_delete_body", &[("name", &name)]);
    let irreversible = tr("confirm_delete_irreversible");
    let delete_label = tr("main_delete");
    let cancel_label = tr("btn_cancel");

    let mut open = true;
    crate::ui::render_modal_backdrop(ctx, "confirm_remove_project_backdrop");
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(body);
            ui.label(irreversible);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(delete_label).clicked() {
                    projects_panel::do_remove_project(ctx, app, &name);
                    app.ui.confirm_remove_project = None;
                }
                if ui.button(cancel_label).clicked() {
                    app.ui.confirm_remove_project = None;
                }
            });
        });

    if !open {
        app.ui.confirm_remove_project = None;
    }
}
