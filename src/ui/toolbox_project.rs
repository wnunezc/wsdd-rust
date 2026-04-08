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
use crate::i18n::tr;
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

    let title = format!("{} — {}", tr("col_toolbox"), project.name);
    let url = if project.ssl {
        format!("https://{}", project.domain)
    } else {
        format!("http://{}", project.domain)
    };
    let entry_point = if project.entry_point.as_path().is_empty() {
        tr("info_root")
    } else {
        project.entry_point.as_path().to_string()
    };

    let mut open = true;
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .min_width(360.0)
        .open(&mut open)
        .show(ctx, |ui| {
            let actions = tr("toolbox_actions");
            let project_info = tr("toolbox_project_info");
            let open_folder = tr("main_open_folder");
            let open_browser = tr("main_open_browser");
            let close_label = tr("btn_close");
            let info_name = format!("{}:", tr("col_name"));
            let info_domain = format!("{}:", tr("col_domain"));
            let info_php = format!("{}:", tr("col_php"));
            let info_work_path = format!("{}:", tr("info_work_path"));
            let info_entry_point = format!("{}:", tr("info_entry_point"));
            let info_ssl = format!("{}:", tr("col_ssl"));

            ui.add_space(4.0);

            // ── Acciones rápidas ──────────────────────────────────────────
            ui.strong(actions);
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                if ui
                    .button(format!("📁 {open_folder}"))
                    .on_hover_text(&project.work_path)
                    .clicked()
                {
                    launch("explorer.exe", &[&project.work_path], None);
                }

                if ui
                    .button(format!("🌐 {open_browser}"))
                    .on_hover_text(&url)
                    .clicked()
                {
                    launch("cmd", &["/c", "start", &url], None);
                }
            });

            ui.add_space(12.0);
            ui.separator();

            // ── Info del proyecto ─────────────────────────────────────────
            ui.strong(project_info);
            ui.add_space(4.0);

            egui::Grid::new("toolbox_project_info")
                .num_columns(2)
                .spacing([12.0, 6.0])
                .show(ui, |ui| {
                    ui.label(&info_name);
                    ui.label(&project.name);
                    ui.end_row();

                    ui.label(&info_domain);
                    ui.label(&project.domain);
                    ui.end_row();

                    ui.label(&info_php);
                    ui.label(project.php_version.display_name());
                    ui.end_row();

                    ui.label(&info_work_path);
                    ui.label(&project.work_path);
                    ui.end_row();

                    ui.label(&info_entry_point);
                    ui.label(&entry_point);
                    ui.end_row();

                    ui.label(&info_ssl);
                    ui.label(if project.ssl {
                        tr("btn_yes")
                    } else {
                        tr("btn_no")
                    });
                    ui.end_row();
                });

            ui.add_space(12.0);

            if ui.button(close_label).clicked() {
                app.ui.toolbox_project_name = None;
                app.ui.active = ActiveView::Main;
            }
        });

    if !open {
        app.ui.toolbox_project_name = None;
        app.ui.active = ActiveView::Main;
    }
}
