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

use crate::app::WsddApp;
use crate::handlers::docker;
use crate::handlers::log_types::LogLine;
use crate::handlers::ps_script::{launch_shell_window, launch_url};
use crate::i18n::tr;
use crate::models::project::PhpVersion;
use crate::ui::ActiveView;

pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    let name = match app.ui.toolbox_container_name.clone() {
        Some(name) => name,
        None => {
            app.ui.active = ActiveView::Main;
            return;
        }
    };

    let container = app
        .containers
        .iter()
        .find(|entry| entry.name == name)
        .cloned();
    let mut spawn_restart = false;
    let mut open = true;

    crate::ui::render_modal_backdrop(ctx, "toolbox_container_backdrop");
    egui::Window::new(format!("{} - {}", tr("col_toolbox"), name))
        .collapsible(false)
        .resizable(false)
        .order(egui::Order::Foreground)
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
            ui.strong(operations);
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                if ui
                    .button(format!("{} {}", '\u{2B1B}', tr("toolbox_tty")))
                    .on_hover_text(tr("toolbox_tty_hint"))
                    .clicked()
                {
                    launch_shell_window(&format!("docker exec -it {name} sh"));
                }

                if ui
                    .button(format!("{} {restart_label}", '\u{21BA}'))
                    .on_hover_text(&restart_label)
                    .clicked()
                {
                    spawn_restart = true;
                }

                if ui
                    .button(format!("{} {logs_label}", '\u{1F4CB}'))
                    .on_hover_text(tr("toolbox_logs_hint"))
                    .clicked()
                {
                    launch_shell_window(&format!("docker logs {name} -f"));
                }
            });

            if let Some(ref container_info) = container {
                if !container_info.urls.is_empty() {
                    ui.add_space(12.0);
                    ui.separator();
                    ui.strong(urls_title);
                    ui.add_space(4.0);

                    for url in &container_info.urls {
                        let display = if url.starts_with("http") {
                            url.clone()
                        } else {
                            format!("http://{url}")
                        };
                        if ui.link(&display).clicked() {
                            launch_url(&display);
                        }
                    }
                }

                ui.add_space(12.0);
                ui.separator();
                ui.strong(status_title);
                ui.add_space(4.0);

                egui::Grid::new("toolbox_container_info")
                    .num_columns(2)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(&name_label);
                        ui.label(&container_info.name);
                        ui.end_row();

                        ui.label(&image_label);
                        ui.label(&container_info.image);
                        ui.end_row();

                        ui.label(&status_label);
                        if container_info.is_running() {
                            ui.colored_label(
                                egui::Color32::from_rgb(80, 200, 80),
                                &container_info.status,
                            );
                        } else {
                            ui.colored_label(
                                egui::Color32::from_rgb(200, 80, 80),
                                &container_info.status,
                            );
                        }
                        ui.end_row();

                        if !container_info.ports.is_empty() {
                            ui.label(&ports_label);
                            ui.label(&container_info.ports);
                            ui.end_row();
                        }

                        if let Some(php_version) =
                            PhpVersion::from_container_name(&container_info.name)
                        {
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

    if spawn_restart {
        let tx = app.main_log_tx.clone();
        let container_name = name.clone();
        let job_key = format!("container:{container_name}");
        let label = format!("Restart container {}", container_name);

        let _ = app.spawn_async_job(ctx, job_key, label, async move {
            let _ = tx.send(LogLine::info(format!(
                "[Docker] Reiniciando {container_name}..."
            )));
            match docker::restart_container(&container_name, None).await {
                Ok(()) => {
                    let _ = tx.send(LogLine::success(format!(
                        "[Docker] {container_name} reiniciado OK"
                    )));
                    Ok(())
                }
                Err(e) => {
                    let message = format!("[Docker] Error al reiniciar {container_name}: {e}");
                    let _ = tx.send(LogLine::error(message.clone()));
                    Err(message)
                }
            }
        });
    }

    if !open {
        app.ui.toolbox_container_name = None;
        app.ui.active = ActiveView::Main;
    }
}
