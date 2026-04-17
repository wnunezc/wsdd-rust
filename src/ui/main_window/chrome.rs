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
//! Main menu and toolbar rendering.

use crate::app::WsddApp;
use crate::config::environment::{env_config, path_config, path_to_string};
use crate::handlers::external_app;
use crate::handlers::log_types::LogLine;
use crate::handlers::ps_script::{launch, launch_url};
use crate::handlers::setting::AppTheme;
use crate::i18n::tr;
use crate::ui::ActiveView;

use super::actions;
use super::SUPPORT_ISSUES_URL;

/// Renders the application menu bar.
pub(super) fn render_menu_bar(ctx: &egui::Context, app: &mut WsddApp) {
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
                                actions::force_poll(ctx, app);
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button(&reload_docker).clicked() {
                                actions::start_docker_restart(ctx, app);
                                ui.close_menu();
                            }
                            if ui.button(&clear_logs).clicked() {
                                app.main_log.clear();
                                ui.close_menu();
                            }
                        });

                        ui.menu_button(menu_tools, |ui| {
                            if ui.button(&backup_environment).clicked() {
                                actions::start_environment_backup(ctx, app);
                                ui.close_menu();
                            }
                            if ui.button(&restore_environment).clicked() {
                                actions::start_environment_restore(ctx, app);
                                ui.close_menu();
                            }
                            if ui.button(&restore_project_backup).clicked() {
                                actions::start_project_restore(ctx, app);
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button(&wsl_settings).clicked() {
                                app.ui.active = ActiveView::WslSettings;
                                ui.close_menu();
                            }
                            if ui.button(&wsl_restart).clicked() {
                                actions::start_wsl_restart(ctx, app);
                                ui.close_menu();
                            }
                            if ui.button(&wsl_shutdown).clicked() {
                                actions::start_wsl_shutdown(ctx, app);
                                ui.close_menu();
                            }
                            if ui.button(&wsl_start).clicked() {
                                actions::start_wsl_start(ctx, app);
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
                                    let _ = app.main_log_tx.send(LogLine::error(format!(
                                        "[Support] Could not open GitHub Issues: {e}"
                                    )));
                                }
                                ui.close_menu();
                            }
                        });
                    });
                });
        });
}

/// Renders the main toolbar.
pub(super) fn render_toolbar(ctx: &egui::Context, app: &mut WsddApp) {
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
                            launch_url("http://pma.wsdd.dock");
                        }

                        ui.add_space(2.0);

                        if ui
                            .add(egui::Button::new(format!("⚡ {terminal_ps}")).min_size(h))
                            .on_hover_text(&terminal_ps_hint)
                            .clicked()
                        {
                            let env_dir = path_to_string(path_config().environment_root());
                            let command = format!("Set-Location -LiteralPath '{}'", env_dir);
                            launch(
                                env_config().pwsh_exe(),
                                &["-NoLogo", "-NoProfile", "-NoExit", "-Command", &command],
                                None,
                            );
                        }

                        ui.add_space(2.0);

                        if ui
                            .add(egui::Button::new(format!("⬛ {terminal_cmd}")).min_size(h))
                            .on_hover_text(&terminal_cmd_hint)
                            .clicked()
                        {
                            let env_dir = path_to_string(path_config().environment_root());
                            let command = format!("cd /d \"{env_dir}\"");
                            launch("cmd.exe", &["/k", &command], None);
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
                            actions::force_poll(ctx, app);
                        }

                        ui.add_space(2.0);

                        if ui
                            .add(
                                egui::Button::new(format!("⟳ {reload_projects_label}")).min_size(h),
                            )
                            .on_hover_text(&reload_projects_hint)
                            .clicked()
                        {
                            actions::reload_projects(app);
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
