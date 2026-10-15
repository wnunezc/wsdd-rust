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
    let request = match app.ui.confirm_remove_projects.clone() {
        Some(request) => request,
        None => return,
    };
    let title = tr("confirm_delete_title");
    let count = request.project_names.len().to_string();
    let body = trf(
        if request.php_container.is_some() {
            "projects_remove_container_confirm"
        } else {
            "projects_remove_confirm"
        },
        &[("count", &count)],
    );
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
            if let Some(name) = &request.php_container {
                ui.strong(name);
            }
            ui.label(body);
            egui::ScrollArea::vertical()
                .max_height(220.0)
                .show(ui, |ui| {
                    for name in &request.project_names {
                        ui.label(name);
                    }
                });
            ui.label(tr("projects_remove_storage"));
            ui.label(irreversible);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        !projects_panel::project_operations_busy(app),
                        egui::Button::new(delete_label),
                    )
                    .clicked()
                {
                    projects_panel::do_remove_projects(ctx, app, request.clone());
                    app.ui.confirm_remove_projects = None;
                }
                if ui.button(cancel_label).clicked() {
                    app.ui.confirm_remove_projects = None;
                }
            });
        });

    if !open {
        app.ui.confirm_remove_projects = None;
    }
}
