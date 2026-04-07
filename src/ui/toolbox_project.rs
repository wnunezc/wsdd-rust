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
//! Herramientas del proyecto WSDD.
//!
//! Equivalente a `Forms/ToolBoxProject.cs`.
//! Muestra acciones rápidas sobre un proyecto: abrir carpeta, abrir en navegador,
//! info del proyecto.

use crate::app::WsddApp;
use crate::handlers::ps_script::launch;
use crate::ui::ActiveView;

/// Renderiza el panel de herramientas del proyecto.
pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    let name = match app.ui.toolbox_project_name.clone() {
        Some(n) => n,
        None => {
            app.ui.active = ActiveView::Main;
            return;
        }
    };

    let project = match app.projects.iter().find(|p| p.name == name) {
        Some(p) => p.clone(),
        None => {
            app.ui.active = ActiveView::Main;
            return;
        }
    };

    let title = format!("Herramientas — {}", project.name);
    let url = if project.ssl {
        format!("https://{}", project.domain)
    } else {
        format!("http://{}", project.domain)
    };

    let mut open = true;
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .min_width(360.0)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.add_space(4.0);

            // ── Acciones rápidas ──────────────────────────────────────────
            ui.strong("Acciones");
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                if ui
                    .button("📁 Abrir carpeta")
                    .on_hover_text(&project.work_path)
                    .clicked()
                {
                    launch("explorer.exe", &[&project.work_path], None);
                }

                if ui
                    .button("🌐 Abrir en navegador")
                    .on_hover_text(&url)
                    .clicked()
                {
                    launch("cmd", &["/c", "start", &url], None);
                }
            });

            ui.add_space(12.0);
            ui.separator();

            // ── Info del proyecto ─────────────────────────────────────────
            ui.strong("Información del proyecto");
            ui.add_space(4.0);

            egui::Grid::new("toolbox_project_info")
                .num_columns(2)
                .spacing([12.0, 6.0])
                .show(ui, |ui| {
                    ui.label("Nombre:");
                    ui.label(&project.name);
                    ui.end_row();

                    ui.label("Dominio:");
                    ui.label(&project.domain);
                    ui.end_row();

                    ui.label("PHP:");
                    ui.label(project.php_version.display_name());
                    ui.end_row();

                    ui.label("Work Path:");
                    ui.label(&project.work_path);
                    ui.end_row();

                    ui.label("Entry Point:");
                    ui.label(if project.entry_point.as_path().is_empty() {
                        "Raíz"
                    } else {
                        project.entry_point.as_path()
                    });
                    ui.end_row();

                    ui.label("SSL:");
                    ui.label(if project.ssl { "Sí" } else { "No" });
                    ui.end_row();
                });

            ui.add_space(12.0);

            if ui.button("Cerrar").clicked() {
                app.ui.toolbox_project_name = None;
                app.ui.active = ActiveView::Main;
            }
        });

    if !open {
        app.ui.toolbox_project_name = None;
        app.ui.active = ActiveView::Main;
    }
}
