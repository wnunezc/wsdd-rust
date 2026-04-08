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
//! Ventana principal de WSDD.
//!
//! Equivalente a `Forms/Main.cs`. Contiene:
//! - Barra de menú (Archivo, Herramientas, Ayuda, Docker).
//! - Toolbar: phpMyAdmin, terminal PowerShell, CMD, agregar proyecto, refrescar.
//! - Tabs: Contenedores / Proyectos.
//! - Panel de log en la parte inferior.
//! - Polling automático de contenedores cada 3 segundos.
//! - Diálogo de confirmación para eliminación de proyectos.

use std::sync::mpsc;
use std::time::Duration;

use crate::app::WsddApp;
use crate::handlers::docker::{list_containers_sync, ContainerInfo};
use crate::handlers::log_types::LogLevel;
use crate::handlers::ps_script::{launch, ScriptRunner};
use crate::handlers::setting::AppTheme;
use crate::i18n::{tr, trf};
use crate::ui::{containers_panel, projects_panel};
use crate::ui::{ActiveView, MainTab};

const POLL_INTERVAL: Duration = Duration::from_secs(3);

/// Punto de entrada del panel principal.
pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    // Iniciar polling si corresponde
    poll_containers(ctx, app);

    render_menu_bar(ctx, app);
    render_toolbar(ctx, app);
    render_log_panel(ctx, app);
    render_confirm_dialog(ctx, app);
    render_center(ctx, app);

    // Solicitar repaint para que el polling no espere interacción del usuario
    ctx.request_repaint_after(POLL_INTERVAL);
}

// ─── Menú ─────────────────────────────────────────────────────────────────────

fn render_menu_bar(ctx: &egui::Context, app: &mut WsddApp) {
    let menu_file = tr("menu_file");
    let menu_docker = tr("menu_docker");
    let menu_tools = tr("menu_tools");
    let menu_help = tr("menu_help");
    let add_project = tr("main_add_project");
    let exit_label = tr("menu_exit");
    let refresh_containers = tr("menu_refresh_containers");
    let reload_docker = tr("menu_reload_docker");
    let clear_logs = tr("menu_clear_logs");
    let wsl_settings = tr("menu_wsl_settings");
    let settings = tr("menu_settings");
    let help = tr("menu_help");
    let about = tr("menu_about");

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button(menu_file, |ui| {
                if ui.button(&add_project).clicked() {
                    app.ui.reset_add_project_form();
                    app.ui.active = ActiveView::AddProject;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(&exit_label).clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.menu_button(menu_docker, |ui| {
                if ui.button(&refresh_containers).clicked() {
                    force_poll(ctx, app);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(&reload_docker).clicked() {
                    let tx = app.main_log_tx.clone();
                    let runner = app.runner.clone();
                    std::thread::spawn(move || {
                        let _ = tx.send(crate::handlers::log_types::LogLine::info(
                            "[Docker] Reiniciando Docker Desktop...",
                        ));
                        // Equivalente a ReloadDockerDesktop en C#
                        let _ = runner.run_direct_sync(
                            "powershell",
                            &[
                                "-NoProfile",
                                "-NonInteractive",
                                "-Command",
                                "Stop-Process -Name 'Docker Desktop' -Force -ErrorAction SilentlyContinue; Start-Sleep 3; Start-Process 'Docker Desktop'",
                            ],
                            None,
                            None,
                        );
                        let _ = tx.send(crate::handlers::log_types::LogLine::success(
                            "[Docker] Docker Desktop reiniciado.",
                        ));
                    });
                    ui.close_menu();
                }
                if ui.button(&clear_logs).clicked() {
                    app.main_log.clear();
                    ui.close_menu();
                }
            });

            ui.menu_button(menu_tools, |ui| {
                if ui.button(&wsl_settings).clicked() {
                    app.ui.active = ActiveView::WslSettings;
                    ui.close_menu();
                }
                if ui.button(&settings).clicked() {
                    app.ui.active = ActiveView::Settings;
                    ui.close_menu();
                }
            });

            ui.menu_button(menu_help, |ui| {
                if ui.button(&help).clicked() {
                    app.ui.active = ActiveView::Helps;
                    ui.close_menu();
                }
                if ui.button(format!("{about}...")).clicked() {
                    app.ui.active = ActiveView::About;
                    ui.close_menu();
                }
            });
        });
    });
}

// ─── Toolbar ─────────────────────────────────────────────────────────────────

fn render_toolbar(ctx: &egui::Context, app: &mut WsddApp) {
    let phpmyadmin = tr("toolbar_phpmyadmin");
    let open_phpmyadmin = tr("toolbar_open_phpmyadmin");
    let terminal_ps = tr("toolbar_terminal_ps");
    let terminal_ps_hint = tr("toolbar_terminal_ps_hint");
    let terminal_cmd = tr("toolbar_terminal_cmd");
    let terminal_cmd_hint = tr("toolbar_terminal_cmd_hint");
    let add_project = tr("main_add_project");
    let add_project_hint = tr("toolbar_add_project_hint");
    let refresh = tr("btn_refresh");
    let refresh_hint = tr("menu_refresh_containers");
    let reload_projects_label = tr("toolbar_reload_projects");
    let reload_projects_hint = tr("toolbar_reload_projects_hint");
    let theme_label = format!("{}:", tr("settings_theme"));

    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.add_space(3.0);
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            let h = egui::vec2(0.0, 26.0);

            if ui
                .add(egui::Button::new(format!("⬡ {phpmyadmin}")).min_size(h))
                .on_hover_text(&open_phpmyadmin)
                .clicked()
            {
                launch("cmd", &["/c", "start", "http://pma.wsdd.dock"], None);
            }

            ui.add_space(2.0);

            if ui
                .add(egui::Button::new(format!("⚡ {terminal_ps}")).min_size(h))
                .on_hover_text(&terminal_ps_hint)
                .clicked()
            {
                launch(
                    "pwsh.exe",
                    &[
                        "-NoExit",
                        "-NoProfile",
                        "-Command",
                        "cd C:\\WSDD-Environment",
                    ],
                    None,
                );
            }

            ui.add_space(2.0);

            if ui
                .add(egui::Button::new(format!("⬛ {terminal_cmd}")).min_size(h))
                .on_hover_text(&terminal_cmd_hint)
                .clicked()
            {
                launch("cmd.exe", &["/k", "cd /d C:\\WSDD-Environment"], None);
            }

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            if ui
                .add(egui::Button::new(format!("＋ {add_project}")).min_size(h))
                .on_hover_text(&add_project_hint)
                .clicked()
            {
                app.ui.reset_add_project_form();
                app.ui.active = ActiveView::AddProject;
            }

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            if ui
                .add(egui::Button::new(format!("↺ {refresh}")).min_size(h))
                .on_hover_text(&refresh_hint)
                .clicked()
            {
                force_poll(ctx, app);
            }

            ui.add_space(2.0);

            if ui
                .add(egui::Button::new(format!("⟳ {reload_projects_label}")).min_size(h))
                .on_hover_text(&reload_projects_hint)
                .clicked()
            {
                reload_projects(app);
            }

            // ── Selector de tema — lado derecho ───────────────────────────
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(4.0);
                let before = app.settings.theme;
                egui::ComboBox::from_id_salt("theme_switcher")
                    .selected_text(app.settings.theme.display_name())
                    .width(108.0)
                    .show_ui(ui, |ui| {
                        for &t in AppTheme::all() {
                            ui.selectable_value(&mut app.settings.theme, t, t.display_name());
                        }
                    });
                if app.settings.theme != before {
                    let _ = app.settings.save();
                }
                ui.label(&theme_label);
            });
        });
        ui.add_space(3.0);
    });
}

// ─── Panel de log ─────────────────────────────────────────────────────────────

fn render_log_panel(ctx: &egui::Context, app: &mut WsddApp) {
    let log_title = tr("log_title");
    let clear_label = tr("btn_clear");
    let copy_label = tr("btn_copy");

    egui::TopBottomPanel::bottom("log_panel")
        .min_height(130.0)
        .max_height(300.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.strong(log_title);
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
            ui.separator();

            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    let dark = ui.visuals().dark_mode;
                    for line in &app.main_log {
                        let color = match line.level {
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
                        };
                        ui.colored_label(color, &line.text);
                    }
                });
        });
}

// ─── Diálogo de confirmación para eliminar proyecto ───────────────────────────

fn render_confirm_dialog(ctx: &egui::Context, app: &mut WsddApp) {
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
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(body);
            ui.label(irreversible);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(delete_label).clicked() {
                    projects_panel::do_remove_project(app, &name);
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

// ─── Panel central — tabs ─────────────────────────────────────────────────────

fn render_center(ctx: &egui::Context, app: &mut WsddApp) {
    let containers = tr("main_containers");
    let projects = tr("main_projects");

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            if tab_button(
                ui,
                &containers,
                app.ui.active_main_tab == MainTab::Containers,
            ) {
                app.ui.active_main_tab = MainTab::Containers;
            }
            if tab_button(ui, &projects, app.ui.active_main_tab == MainTab::Projects) {
                app.ui.active_main_tab = MainTab::Projects;
                reload_projects(app);
            }
        });
        ui.separator();

        egui::ScrollArea::both()
            .auto_shrink([false; 2])
            .show(ui, |ui| match app.ui.active_main_tab {
                MainTab::Containers => containers_panel::render(ui, app),
                MainTab::Projects => projects_panel::render(ui, app),
            });
    });
}

// ─── Polling de contenedores ─────────────────────────────────────────────────

/// Inicia un poll si no hay uno activo y ha pasado el intervalo mínimo.
fn poll_containers(ctx: &egui::Context, app: &mut WsddApp) {
    if app.container_poll_active {
        return;
    }
    if app.last_container_poll.elapsed() < POLL_INTERVAL {
        return;
    }
    start_poll(ctx, app);
}

/// Fuerza un poll inmediato ignorando el intervalo.
fn force_poll(ctx: &egui::Context, app: &mut WsddApp) {
    if app.container_poll_active {
        return;
    }
    start_poll(ctx, app);
}

fn start_poll(ctx: &egui::Context, app: &mut WsddApp) {
    let (tx, rx) = mpsc::channel::<Vec<ContainerInfo>>();
    app.container_poll_rx = Some(rx);
    app.container_poll_active = true;

    let runner = app.runner.clone();
    let ctx = ctx.clone();

    std::thread::spawn(move || {
        let containers = list_containers_sync(&runner).unwrap_or_default();
        let _ = tx.send(containers);
        ctx.request_repaint();
    });
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Botón de tab con color de selección adaptado al tema.
///
/// En temas oscuros evita el celeste de `selection.bg_fill` y usa el highlight
/// neutro del widget activo. En tema claro mantiene el color de selección nativo.
fn tab_button(ui: &mut egui::Ui, label: &str, selected: bool) -> bool {
    let (fill, text) = if selected {
        let fill = if ui.visuals().dark_mode {
            ui.visuals().widgets.active.weak_bg_fill
        } else {
            ui.visuals().selection.bg_fill
        };
        (fill, egui::RichText::new(label).strong())
    } else {
        (egui::Color32::TRANSPARENT, egui::RichText::new(label))
    };
    ui.add(
        egui::Button::new(text)
            .fill(fill)
            .stroke(egui::Stroke::NONE),
    )
    .clicked()
}

fn reload_projects(app: &mut WsddApp) {
    match crate::handlers::project::list_all() {
        Ok(list) => app.projects = list,
        Err(e) => {
            let _ = app
                .main_log_tx
                .send(crate::handlers::log_types::LogLine::error(format!(
                    "[Proyectos] Error al cargar: {e}"
                )));
        }
    }
}
