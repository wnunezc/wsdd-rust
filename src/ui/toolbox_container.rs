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
//! Herramientas del contenedor Docker.
//!
//! Equivalente a `Forms/ToolBoxContainer.cs`.
//! Permite abrir TTY, reiniciar, ver logs y abrir URLs del contenedor.

use crate::app::WsddApp;
use crate::handlers::docker::restart_container_sync;
use crate::handlers::log_types::LogLine;
use crate::handlers::ps_script::{launch, launch_shell_window};
use crate::i18n::tr;
use crate::models::project::PhpVersion;
use crate::ui::ActiveView;

/// Renderiza el panel de herramientas del contenedor.
pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    let name = match app.ui.toolbox_container_name.clone() {
        Some(n) => n,
        None => {
            app.ui.active = ActiveView::Main;
            return;
        }
    };

    // Buscar el ContainerInfo para obtener URLs y estado
    let container = app.containers.iter().find(|c| c.name == name).cloned();

    let mut spawn_restart = false;
    let mut open = true;

    egui::Window::new(format!("{} — {}", tr("col_toolbox"), name))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .min_width(360.0)
        .open(&mut open)
        .show(ctx, |ui| {
            let operations = tr("toolbox_operations");
            let urls_title = tr("toolbox_urls");
            let status_title = tr("col_status");
            let close_label = tr("btn_close");
            let restart_label = tr("main_restart");
            let logs_label = tr("main_view_logs");
            let name_label = format!("{}:", tr("col_name"));
            let image_label = format!("{}:", tr("col_image"));
            let status_label = format!("{}:", tr("col_status"));
            let ports_label = format!("{}:", tr("col_ports"));
            let webmin_user_label = format!("{}:", tr("settings_webmin_user"));

            ui.add_space(4.0);

            // ── Acciones del contenedor ───────────────────────────────────
            ui.strong(operations);
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                if ui
                    .button(format!("⬛ {}", tr("toolbox_tty")))
                    .on_hover_text(tr("toolbox_tty_hint"))
                    .clicked()
                {
                    launch_shell_window(&format!("docker exec -it {name} sh"));
                }

                if ui
                    .button(format!("↺ {restart_label}"))
                    .on_hover_text(&restart_label)
                    .clicked()
                {
                    spawn_restart = true;
                }

                if ui
                    .button(format!("📋 {logs_label}"))
                    .on_hover_text(tr("toolbox_logs_hint"))
                    .clicked()
                {
                    launch_shell_window(&format!("docker logs {name} -f"));
                }
            });

            // ── URLs del contenedor ───────────────────────────────────────
            if let Some(ref c) = container {
                if !c.urls.is_empty() {
                    ui.add_space(12.0);
                    ui.separator();
                    ui.strong(urls_title);
                    ui.add_space(4.0);

                    for url in &c.urls {
                        let display = if url.starts_with("http") {
                            url.clone()
                        } else {
                            format!("http://{url}")
                        };
                        if ui.link(&display).clicked() {
                            launch("cmd", &["/c", "start", &display], None);
                        }
                    }
                }

                // Estado
                ui.add_space(12.0);
                ui.separator();
                ui.strong(status_title);
                ui.add_space(4.0);
                egui::Grid::new("toolbox_container_info")
                    .num_columns(2)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(&name_label);
                        ui.label(&c.name);
                        ui.end_row();

                        ui.label(&image_label);
                        ui.label(&c.image);
                        ui.end_row();

                        ui.label(&status_label);
                        if c.is_running() {
                            ui.colored_label(egui::Color32::from_rgb(80, 200, 80), &c.status);
                        } else {
                            ui.colored_label(egui::Color32::from_rgb(200, 80, 80), &c.status);
                        }
                        ui.end_row();

                        if !c.ports.is_empty() {
                            ui.label(&ports_label);
                            ui.label(&c.ports);
                            ui.end_row();
                        }

                        if let Some(php_version) = PhpVersion::from_container_name(&c.name) {
                            if let Some(credentials) =
                                app.settings.webmin_credentials_for(&php_version)
                            {
                                ui.label(&webmin_user_label);
                                ui.label(&credentials.username);
                                ui.end_row();
                            }
                        }
                    });
            }

            ui.add_space(12.0);

            if ui.button(close_label).clicked() {
                app.ui.toolbox_container_name = None;
                app.ui.active = ActiveView::Main;
            }
        });

    // Ejecutar restart fuera del closure para evitar borrow conflict
    if spawn_restart {
        let runner = app.runner.clone();
        let tx = app.main_log_tx.clone();
        let cname = name.clone();
        std::thread::spawn(move || {
            let _ = tx.send(LogLine::info(format!("[Docker] Reiniciando {cname}...")));
            match restart_container_sync(&runner, &cname) {
                Ok(()) => {
                    let _ = tx.send(LogLine::success(format!("[Docker] {cname} reiniciado ✓")));
                }
                Err(e) => {
                    let _ = tx.send(LogLine::error(format!(
                        "[Docker] Error al reiniciar {cname}: {e}"
                    )));
                }
            }
        });
    }

    if !open {
        app.ui.toolbox_container_name = None;
        app.ui.active = ActiveView::Main;
    }
}
