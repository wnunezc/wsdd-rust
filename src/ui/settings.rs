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
//! Pantalla de configuracion de WSDD.
//! Equivalente a `Forms/Setting.cs`.
//!
//! Edita una copia de `AppSettings` (draft) y aplica al guardar.
//! Los cambios se descartan si el usuario cancela.

use crate::app::WsddApp;
use crate::handlers::docker_deploy;
use crate::i18n::{self, tr, Language};
use crate::models::project::PhpVersion;
use crate::ui::ActiveView;

pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    // Inicializar draft al entrar por primera vez en esta visita
    if app.ui.settings_draft.is_none() {
        app.ui.settings_draft = Some(app.settings.clone());
        app.ui.settings_error = None;
    }

    let mut save = false;
    let mut cancel = false;
    let title = format!("⚙ {}", tr("settings_title"));
    let cancel_label = format!("  {}  ", tr("btn_cancel"));
    let save_label = format!("  {}  ", tr("btn_save"));
    let general_title = tr("settings_general");
    let php_title = tr("settings_php");
    let tools_title = tr("settings_tools_section");
    let projects_path_label = format!("{}:", tr("settings_projects_path"));
    let language_label = format!("{}:", tr("settings_language"));
    let docker_path_label = format!("{}:", tr("settings_docker_path"));
    let wsl_distro_label = format!("{}:", tr("settings_wsl_distro"));
    let log_lines_label = format!("{}:", tr("settings_log_lines"));
    let auto_start_label = format!("{}:", tr("settings_auto_start"));
    let php_note = tr("settings_apply_new_php_note");
    let php_memory_label = format!("{}:", tr("settings_php_memory"));
    let php_upload_label = format!("{}:", tr("settings_php_upload"));
    let php_timezone_label = format!("{}:", tr("settings_php_timezone"));
    let prereq_title = tr("settings_prereq_section");
    let prereq_note = tr("settings_prereq_note");
    let prereq_runtime_note = tr("settings_prereq_runtime_note");
    let mysql_database_label = format!("{}:", tr("settings_mysql_database"));
    let mysql_user_label = format!("{}:", tr("settings_mysql_user"));
    let mysql_password_label = format!("{}:", tr("settings_mysql_password"));
    let mysql_root_password_label = format!("{}:", tr("settings_mysql_root_password"));
    let webmin_credentials_title = tr("settings_webmin_credentials_section");
    let webmin_credentials_note = tr("settings_webmin_credentials_note");
    let webmin_runtime_note = tr("settings_webmin_credentials_runtime_note");
    let webmin_user_label = tr("settings_webmin_user");
    let webmin_password_label = tr("settings_webmin_password");
    let webmin_version_label = format!("{}:", tr("settings_webmin_version"));
    let mut open = true;

    crate::ui::render_modal_backdrop(ctx, "settings_backdrop");

    egui::Window::new(&title)
        .collapsible(false)
        .resizable(true)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(860.0)
        .default_height(680.0)
        .min_width(760.0)
        .min_height(540.0)
        .open(&mut open)
        .show(ctx, |ui| {
            let draft = app.ui.settings_draft.as_mut().unwrap();

            // ── Cabecera ──────────────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(&cancel_label).clicked() {
                        cancel = true;
                    }
                    ui.add_space(4.0);
                    if ui
                        .add(
                            egui::Button::new(&save_label)
                                .fill(egui::Color32::from_rgb(34, 139, 34)),
                        )
                        .clicked()
                    {
                        save = true;
                    }
                });
            });
        ui.separator();
        ui.add_space(6.0);

        if let Some(error) = &app.ui.settings_error {
            ui.label(
                egui::RichText::new(error).color(egui::Color32::from_rgb(220, 80, 80)),
            );
            ui.add_space(6.0);
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            // ── General ───────────────────────────────────────────────────
            egui::CollapsingHeader::new(egui::RichText::new(&general_title).strong())
                .default_open(true)
                .show(ui, |ui| {
                    egui::Grid::new("settings_general")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .min_col_width(180.0)
                        .show(ui, |ui| {
                            ui.label(&projects_path_label);
                            ui.add(
                                egui::TextEdit::singleline(&mut draft.projects_path)
                                    .desired_width(300.0)
                                    .hint_text(r"C:\WSDD-Projects"),
                            );
                            ui.end_row();

                            ui.label(&language_label);
                            egui::ComboBox::from_id_salt("settings_language")
                                .selected_text(draft.language.native_name())
                                .width(180.0)
                                .show_ui(ui, |ui| {
                                    for &language in Language::all() {
                                        ui.selectable_value(
                                            &mut draft.language,
                                            language,
                                            language.native_name(),
                                        );
                                    }
                                });
                            ui.end_row();

                            ui.label(&docker_path_label);
                            let docker_val =
                                draft.docker_path.get_or_insert_with(String::new);
                            ui.add(
                                egui::TextEdit::singleline(docker_val)
                                    .desired_width(300.0)
                                    .hint_text(
                                        r"C:\Program Files\Docker\Docker\Docker Desktop.exe (opcional)",
                                    ),
                            );
                            ui.end_row();

                            ui.label(&wsl_distro_label);
                            let distro_val =
                                draft.wsl_distro.get_or_insert_with(String::new);
                            ui.add(
                                egui::TextEdit::singleline(distro_val)
                                    .desired_width(200.0)
                                    .hint_text("Ubuntu-22.04 (opcional)"),
                            );
                            ui.end_row();

                            ui.label(&log_lines_label);
                            ui.add(
                                egui::DragValue::new(&mut draft.log_max_lines)
                                    .range(100..=10000)
                                    .speed(10.0),
                            );
                            ui.end_row();

                            ui.label(&auto_start_label);
                            ui.checkbox(&mut draft.auto_start_containers, "");
                            ui.end_row();
                        });
                });

            ui.add_space(8.0);

            // ── PHP ───────────────────────────────────────────────────────
            egui::CollapsingHeader::new(egui::RichText::new(&php_title).strong())
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(&php_note)
                        .size(11.0)
                        .color(ui.visuals().weak_text_color()),
                    );
                    ui.add_space(6.0);

                    egui::Grid::new("settings_php")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .min_col_width(180.0)
                        .show(ui, |ui| {
                            ui.label(&php_memory_label);
                            ui.add(
                                egui::TextEdit::singleline(&mut draft.php_memory_limit)
                                    .desired_width(100.0)
                                    .hint_text("512M"),
                            );
                            ui.end_row();

                            ui.label(&php_upload_label);
                            ui.add(
                                egui::TextEdit::singleline(&mut draft.php_upload_max_filesize)
                                    .desired_width(100.0)
                                    .hint_text("256M"),
                            );
                            ui.end_row();

                            ui.label(&php_timezone_label);
                            ui.add(
                                egui::TextEdit::singleline(&mut draft.php_timezone)
                                    .desired_width(200.0)
                                    .hint_text("UTC"),
                            );
                            ui.end_row();
                        });
                });

            ui.add_space(8.0);

            // ── Prerequisitos ─────────────────────────────────────────────
            egui::CollapsingHeader::new(egui::RichText::new(&prereq_title).strong())
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(&prereq_note)
                            .size(11.0)
                            .color(ui.visuals().weak_text_color()),
                    );
                    ui.label(
                        egui::RichText::new(&prereq_runtime_note)
                            .size(11.0)
                            .color(ui.visuals().weak_text_color()),
                    );
                    ui.add_space(6.0);

                    egui::Grid::new("settings_prereq")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .min_col_width(180.0)
                        .show(ui, |ui| {
                            ui.label(&mysql_database_label);
                            ui.add(
                                egui::TextEdit::singleline(
                                    &mut draft.prereq_credentials.mysql_database,
                                )
                                .desired_width(180.0)
                                .hint_text("wsdd-database"),
                            );
                            ui.end_row();

                            ui.label(&mysql_user_label);
                            ui.add(
                                egui::TextEdit::singleline(
                                    &mut draft.prereq_credentials.mysql_user,
                                )
                                .desired_width(180.0)
                                .hint_text("tester"),
                            );
                            ui.end_row();

                            ui.label(&mysql_password_label);
                            ui.add(
                                egui::TextEdit::singleline(
                                    &mut draft.prereq_credentials.mysql_password,
                                )
                                .desired_width(180.0)
                                .password(true)
                                .hint_text("required"),
                            );
                            ui.end_row();

                            ui.label(&mysql_root_password_label);
                            ui.add(
                                egui::TextEdit::singleline(
                                    &mut draft.prereq_credentials.mysql_root_password,
                                )
                                .desired_width(180.0)
                                .password(true)
                                .hint_text("required"),
                            );
                            ui.end_row();
                        });
                });

            ui.add_space(8.0);

            // ── Webmin por versión PHP ───────────────────────────────────
            egui::CollapsingHeader::new(
                egui::RichText::new(&webmin_credentials_title).strong(),
            )
            .default_open(true)
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(&webmin_credentials_note)
                        .size(11.0)
                        .color(ui.visuals().weak_text_color()),
                );
                ui.label(
                    egui::RichText::new(&webmin_runtime_note)
                        .size(11.0)
                        .color(ui.visuals().weak_text_color()),
                );
                ui.add_space(6.0);

                egui::Grid::new("settings_webmin_credentials")
                    .num_columns(3)
                    .spacing([12.0, 8.0])
                    .min_col_width(150.0)
                    .show(ui, |ui| {
                        ui.strong(tr("col_php"));
                        ui.strong(&webmin_user_label);
                        ui.strong(&webmin_password_label);
                        ui.end_row();

                        for php_version in PhpVersion::all() {
                            let current = draft
                                .webmin_credentials_entry(&php_version)
                                .cloned()
                                .unwrap_or_else(|| crate::handlers::setting::WebminCredentials {
                                    php_version: php_version.clone(),
                                    username: String::new(),
                                    password: String::new(),
                                });

                            let mut username = current.username;
                            let mut password = current.password;

                            ui.label(php_version.display_name());
                            let user_changed = ui
                                .add(
                                    egui::TextEdit::singleline(&mut username)
                                        .desired_width(150.0)
                                        .hint_text("admin"),
                                )
                                .changed();
                            let password_changed = ui
                                .add(
                                    egui::TextEdit::singleline(&mut password)
                                        .desired_width(150.0)
                                        .password(true)
                                        .hint_text("required"),
                                )
                                .changed();
                            ui.end_row();

                            if user_changed || password_changed {
                                draft.set_webmin_credentials_draft(
                                    php_version,
                                    username,
                                    password,
                                );
                            }
                        }
                    });
            });

            ui.add_space(8.0);

            // ── Herramientas ──────────────────────────────────────────────
            egui::CollapsingHeader::new(
                egui::RichText::new(&tools_title).strong(),
            )
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("settings_tools")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .min_col_width(180.0)
                    .show(ui, |ui| {
                        ui.label(&webmin_version_label);
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut draft.webmin_version)
                                    .desired_width(100.0)
                                    .hint_text("2.630"),
                            );
                            ui.label(
                                egui::RichText::new("(PHP Dockerfiles)")
                                    .size(11.0)
                                    .color(ui.visuals().weak_text_color()),
                            );
                        });
                        ui.end_row();
                    });
            });

            ui.add_space(16.0);
        });
    });

    if !open && !save {
        cancel = true;
    }

    // ── Aplicar accion fuera del closure ─────────────────────────────────────
    if save {
        if let Some(mut draft) = app.ui.settings_draft.take() {
            let selected_language = draft.language;
            // Normalizar strings vacíos a None en campos opcionales
            if draft.docker_path.as_deref() == Some("") {
                draft.docker_path = None;
            }
            if draft.wsl_distro.as_deref() == Some("") {
                draft.wsl_distro = None;
            }
            if let Err(e) = draft.validate_prerequisite_credentials() {
                app.ui.settings_error = Some(e.to_string());
                app.ui.settings_draft = Some(draft);
                return;
            }
            if let Err(e) = draft.validate_webmin_credentials() {
                app.ui.settings_error = Some(e.to_string());
                app.ui.settings_draft = Some(draft);
                return;
            }

            app.settings = draft;
            i18n::set_language(selected_language);
            if let Err(e) = app.settings.save() {
                app.ui.settings_error = Some(format!("Error guardando configuracion: {e}"));
                app.ui.settings_draft = Some(app.settings.clone());
                return;
            }

            if let Err(e) = docker_deploy::sync_prerequisite_compose_sync(&app.settings) {
                app.ui.settings_error =
                    Some(format!("Error actualizando init.yml administrado: {e}"));
                app.ui.settings_draft = Some(app.settings.clone());
                return;
            }

            if let Err(e) = docker_deploy::sync_saved_php_version_resources_sync(&app.settings) {
                app.ui.settings_error =
                    Some(format!("Error actualizando recursos PHP/Webmin: {e}"));
                app.ui.settings_draft = Some(app.settings.clone());
                return;
            }
        }
        app.ui.settings_error = None;
        app.ui.active = ActiveView::Main;
    } else if cancel {
        app.ui.settings_draft = None;
        app.ui.settings_error = None;
        app.ui.active = ActiveView::Main;
    }
}
