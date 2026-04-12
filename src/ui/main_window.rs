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
//! - Layout dual: Contenedores / Proyectos visibles al mismo tiempo.
//! - Panel de log en la parte inferior.
//! - Polling automático de contenedores cada 3 segundos.
//! - Diálogo de confirmación para eliminación de proyectos.

use std::sync::mpsc;
use std::time::Duration;

use crate::app::WsddApp;
use crate::handlers::docker::gather_poll_snapshot_sync;
use crate::handlers::external_app;
use crate::handlers::log_types::{LogLevel, LogLine};
use crate::handlers::ps_script::{launch, ScriptRunner};
use crate::handlers::setting::AppTheme;
use crate::i18n::{tr, trf};
use crate::ui::ActiveView;
use crate::ui::{containers_panel, projects_panel};

const POLL_INTERVAL: Duration = Duration::from_secs(3);
const SUPPORT_ISSUES_URL: &str = "https://github.com/wnunezc/wsdd-rust/issues/new";

/// Punto de entrada del panel principal.
pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    // Iniciar polling si corresponde
    poll_containers(ctx, app);

    render_menu_bar(ctx, app);
    render_toolbar(ctx, app);
    render_status_bar(ctx, app);
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
    let backup_environment = tr("menu_backup_environment");
    let restore_environment = tr("menu_restore_environment");
    let restore_project_backup = tr("menu_restore_project_backup");
    let wsl_settings = tr("menu_wsl_settings");
    let wsl_restart = tr("menu_restart_wsl");
    let wsl_shutdown = tr("menu_shutdown_wsl");
    let wsl_start = tr("menu_start_wsl");
    let settings = tr("menu_settings");
    let help = tr("menu_help");
    let about = tr("menu_about");
    let support = tr("menu_support_report_bug");

    egui::TopBottomPanel::top("menu_bar")
        .exact_height(30.0)
        .show(ctx, |ui| {
            egui::Frame::none()
                .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                .show(ui, |ui| {
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
                                start_script_sequence_action(
                                    ctx,
                                    app,
                                    "[Docker] Reiniciando Docker Desktop...",
                                    "[Docker] Docker Desktop reiniciado.",
                                    &["dd-stop.ps1", "dd-start.ps1"],
                                );
                                ui.close_menu();
                            }
                            if ui.button(&clear_logs).clicked() {
                                app.main_log.clear();
                                ui.close_menu();
                            }
                        });

                        ui.menu_button(menu_tools, |ui| {
                            if ui.button(&backup_environment).clicked() {
                                start_environment_backup(app);
                                ui.close_menu();
                            }
                            if ui.button(&restore_environment).clicked() {
                                start_environment_restore(app);
                                ui.close_menu();
                            }
                            if ui.button(&restore_project_backup).clicked() {
                                start_project_restore(app);
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button(&wsl_settings).clicked() {
                                app.ui.active = ActiveView::WslSettings;
                                ui.close_menu();
                            }
                            if ui.button(&wsl_restart).clicked() {
                                start_script_sequence_action(
                                    ctx,
                                    app,
                                    "[WSL] Reiniciando servicios WSL...",
                                    "[WSL] Servicios WSL reiniciados.",
                                    &["wsl-restart.ps1"],
                                );
                                ui.close_menu();
                            }
                            if ui.button(&wsl_shutdown).clicked() {
                                start_script_sequence_action(
                                    ctx,
                                    app,
                                    "[WSL] Apagando WSL por completo...",
                                    "[WSL] WSL apagado.",
                                    &["wsl-shutdown.ps1"],
                                );
                                ui.close_menu();
                            }
                            if ui.button(&wsl_start).clicked() {
                                start_script_sequence_action(
                                    ctx,
                                    app,
                                    "[WSL] Iniciando servicios WSL...",
                                    "[WSL] Servicios WSL iniciados.",
                                    &["wsl-start.ps1"],
                                );
                                ui.close_menu();
                            }
                            ui.separator();
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
                            ui.separator();
                            if ui.button(&support).clicked() {
                                if let Err(e) = external_app::open_url(SUPPORT_ISSUES_URL) {
                                    let _ = app.main_log_tx.send(
                                        crate::handlers::log_types::LogLine::error(format!(
                                            "[Support] Could not open GitHub Issues: {e}"
                                        )),
                                    );
                                }
                                ui.close_menu();
                            }
                        });
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

    egui::TopBottomPanel::top("toolbar")
        .exact_height(42.0)
        .show(ctx, |ui| {
            egui::Frame::none()
                .inner_margin(egui::Margin::symmetric(8.0, 5.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let h = egui::vec2(0.0, 28.0);

                        if ui
                            .add(egui::Button::new(format!("◎ {phpmyadmin}")).min_size(h))
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
                            .add(
                                egui::Button::new(format!("⟳ {reload_projects_label}")).min_size(h),
                            )
                            .on_hover_text(&reload_projects_hint)
                            .clicked()
                        {
                            reload_projects(app);
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(4.0);
                            let before = app.settings.theme;
                            egui::ComboBox::from_id_salt("theme_switcher")
                                .selected_text(app.settings.theme.display_name())
                                .width(118.0)
                                .show_ui(ui, |ui| {
                                    for &t in AppTheme::all() {
                                        ui.selectable_value(
                                            &mut app.settings.theme,
                                            t,
                                            t.display_name(),
                                        );
                                    }
                                });
                            if app.settings.theme != before {
                                let _ = app.settings.save();
                            }
                            ui.label(egui::RichText::new(&theme_label).size(13.5).strong());
                        });
                    });
                });
        });
}

// ─── Panel de log ─────────────────────────────────────────────────────────────

fn render_log_panel(ctx: &egui::Context, app: &mut WsddApp) {
    let log_title = tr("log_title");
    let clear_label = tr("btn_clear");
    let copy_label = tr("btn_copy");
    let desired_height = (ctx.available_rect().height() * 0.5).max(240.0);

    egui::TopBottomPanel::bottom("log_panel")
        .default_height(desired_height)
        .min_height(220.0)
        .max_height(desired_height.max(520.0))
        .resizable(true)
        .show(ctx, |ui| {
            show_surface_panel(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(log_title).size(14.0).strong());
                    ui.label(
                        egui::RichText::new(format!(
                            "{} {}",
                            app.main_log.len(),
                            tr("status_bar_logs")
                        ))
                        .size(12.5)
                        .color(ui.visuals().weak_text_color()),
                    );
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
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                egui::Frame::none()
                    .fill(ui.visuals().extreme_bg_color)
                    .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                    .show(ui, |ui| {
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
                                    ui.label(
                                        egui::RichText::new(&line.text).monospace().color(color),
                                    );
                                }
                            });
                    });
            });
        });
}

fn render_status_bar(ctx: &egui::Context, app: &mut WsddApp) {
    let total_containers = app.containers.len();
    let running_containers = app.containers.iter().filter(|c| c.is_running()).count();
    let projects = app.projects.len();
    let logs = app.main_log.len();

    let docker_value = if app.docker_status.daemon_ready {
        tr("status_ready")
    } else if app.container_poll_active {
        tr("status_polling")
    } else {
        tr("status_unavailable")
    };
    let docker_color = if app.docker_status.daemon_ready {
        egui::Color32::from_rgb(80, 200, 80)
    } else if app.container_poll_active {
        ctx.style().visuals.hyperlink_color
    } else {
        ctx.style().visuals.error_fg_color
    };

    let cpu_value = app
        .docker_status
        .cpu_percent
        .map(|value| format!("{value:.1}%"))
        .unwrap_or_else(|| "—".to_string());
    let cpu_color = match app.docker_status.cpu_percent {
        Some(value) if value >= 70.0 => ctx.style().visuals.error_fg_color,
        Some(value) if value >= 35.0 => ctx.style().visuals.warn_fg_color,
        Some(_) => egui::Color32::from_rgb(80, 200, 80),
        None => ui_text_color(ctx),
    };

    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(36.0)
        .show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(10.0, 4.0);

                status_item(
                    ui,
                    &tr("status_bar_containers"),
                    &total_containers.to_string(),
                    None,
                );
                status_separator(ui);
                status_item(
                    ui,
                    &tr("status_bar_running"),
                    &running_containers.to_string(),
                    Some(egui::Color32::from_rgb(80, 200, 80)),
                );
                status_separator(ui);
                status_item(ui, &tr("status_bar_projects"), &projects.to_string(), None);
                status_separator(ui);
                status_item(ui, &tr("status_bar_logs"), &logs.to_string(), None);
                status_separator(ui);

                let docker_response = status_item(
                    ui,
                    &tr("status_bar_docker"),
                    &docker_value,
                    Some(docker_color),
                );
                if let Some(process_name) = &app.docker_status.process_name {
                    docker_response.on_hover_text(format!(
                        "{} process(es): {}",
                        app.docker_status.process_count, process_name
                    ));
                }

                status_separator(ui);
                status_item(ui, &tr("status_bar_cpu"), &cpu_value, Some(cpu_color));
                status_separator(ui);
                status_item(
                    ui,
                    &tr("status_bar_memory"),
                    &format_memory_mb(app.docker_status.memory_mb),
                    None,
                );
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
        ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);

        if ui.available_width() < 900.0 {
            render_main_section(ui, &containers, |ui| {
                containers_panel::render(ui, app);
            });
            render_main_section(ui, &projects, |ui| {
                projects_panel::render(ui, app);
            });
        } else {
            ui.columns(2, |columns| {
                render_main_section(&mut columns[0], &containers, |ui| {
                    containers_panel::render(ui, app);
                });
                render_main_section(&mut columns[1], &projects, |ui| {
                    projects_panel::render(ui, app);
                });
            });
        }
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
    let (tx, rx) = mpsc::channel();
    app.container_poll_rx = Some(rx);
    app.container_poll_active = true;

    let runner = app.runner.clone();
    let ctx = ctx.clone();

    std::thread::spawn(move || {
        let snapshot = gather_poll_snapshot_sync(&runner);
        let _ = tx.send(snapshot);
        ctx.request_repaint();
    });
}

fn start_script_sequence_action(
    ctx: &egui::Context,
    app: &mut WsddApp,
    started: &'static str,
    success: &'static str,
    scripts: &'static [&'static str],
) {
    let tx = app.main_log_tx.clone();
    let runner = app.runner.clone();
    let ctx = ctx.clone();

    std::thread::spawn(move || {
        let _ = tx.send(LogLine::info(started));

        for script in scripts {
            if let Err(e) = runner.run_script_sync(script, None, None) {
                let _ = tx.send(LogLine::error(format!(
                    "[Lifecycle] Error ejecutando {script}: {e}"
                )));
                ctx.request_repaint();
                return;
            }
        }

        let _ = tx.send(LogLine::success(success));
        ctx.request_repaint();
    });
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Botón de tab con color de selección adaptado al tema.
///
/// En temas oscuros evita el celeste de `selection.bg_fill` y usa el highlight
/// neutro del widget activo. En tema claro mantiene el color de selección nativo.
fn render_main_section<Contents>(ui: &mut egui::Ui, title: &str, add_contents: Contents)
where
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
                    .auto_shrink([false; 2])
                    .show(ui, add_contents);
            });
    });
}

fn show_surface_panel<Contents>(ui: &mut egui::Ui, add_contents: Contents)
where
    Contents: FnOnce(&mut egui::Ui),
{
    egui::Frame::none()
        .inner_margin(egui::Margin::symmetric(10.0, 6.0))
        .show(ui, |ui| {
            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::symmetric(12.0, 10.0))
                .show(ui, |ui| add_contents(ui));
        });
}

fn status_item(
    ui: &mut egui::Ui,
    label: &str,
    value: &str,
    accent: Option<egui::Color32>,
) -> egui::Response {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("{label}:"))
                .size(13.5)
                .strong()
                .color(status_label_color(ui.ctx())),
        );
        let mut rich = egui::RichText::new(value).size(13.5).strong();
        if let Some(color) = accent {
            rich = rich.color(color);
        }
        ui.label(rich);
    })
    .response
}

fn status_separator(ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("|")
            .size(13.5)
            .strong()
            .color(ui.visuals().widgets.noninteractive.bg_stroke.color),
    );
}

fn format_poll_age(elapsed: Duration) -> String {
    if elapsed.as_secs() == 0 {
        tr("status_just_now")
    } else {
        trf(
            "status_seconds_ago",
            &[("seconds", &elapsed.as_secs().to_string())],
        )
    }
}

fn ui_text_color(ctx: &egui::Context) -> egui::Color32 {
    ctx.style().visuals.widgets.noninteractive.fg_stroke.color
}

fn status_label_color(ctx: &egui::Context) -> egui::Color32 {
    if ctx.style().visuals.dark_mode {
        egui::Color32::from_rgb(220, 220, 220)
    } else {
        egui::Color32::from_rgb(70, 70, 70)
    }
}

fn format_memory_mb(memory_mb: Option<u64>) -> String {
    match memory_mb {
        Some(value) if value >= 1024 => format!("{:.1} GB", value as f32 / 1024.0),
        Some(value) => format!("{value} MB"),
        None => "—".to_string(),
    }
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

fn start_environment_backup(app: &mut WsddApp) {
    let Some(path) = rfd::FileDialog::new()
        .set_title(tr("backup_dialog_save_title"))
        .add_filter("WSDD Backup", &["zip"])
        .set_file_name(crate::handlers::backup::default_full_backup_name())
        .save_file()
    else {
        return;
    };

    let tx = app.main_log_tx.clone();
    let runner = app.runner.clone();
    std::thread::spawn(move || {
        if let Err(e) = crate::handlers::backup::backup_environment(&path, &runner, &tx) {
            let _ = tx.send(LogLine::error(format!("[Backup] Error: {e}")));
        }
    });
}

fn start_environment_restore(app: &mut WsddApp) {
    let Some(path) = rfd::FileDialog::new()
        .set_title(tr("backup_dialog_restore_title"))
        .add_filter("WSDD Backup", &["zip"])
        .pick_file()
    else {
        return;
    };

    let tx = app.main_log_tx.clone();
    let runner = app.runner.clone();
    std::thread::spawn(move || {
        if let Err(e) = crate::handlers::backup::restore_environment(&path, &runner, &tx) {
            let _ = tx.send(LogLine::error(format!("[Restore] Error: {e}")));
        }
    });
}

fn start_project_restore(app: &mut WsddApp) {
    let Some(path) = rfd::FileDialog::new()
        .set_title(tr("project_restore_dialog_title"))
        .add_filter("WSDD Backup", &["zip"])
        .pick_file()
    else {
        return;
    };

    let tx = app.main_log_tx.clone();
    let runner = app.runner.clone();
    std::thread::spawn(move || {
        if let Err(e) = crate::handlers::backup::restore_project(&path, &runner, &tx) {
            let _ = tx.send(LogLine::error(format!("[Restore] Error: {e}")));
        }
    });
}
