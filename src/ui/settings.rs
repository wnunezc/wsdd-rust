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
use crate::ui::ActiveView;

pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    // Inicializar draft al entrar por primera vez en esta visita
    if app.ui.settings_draft.is_none() {
        app.ui.settings_draft = Some(app.settings.clone());
    }

    let mut save = false;
    let mut cancel = false;

    egui::CentralPanel::default().show(ctx, |ui| {
        let draft = app.ui.settings_draft.as_mut().unwrap();

        // ── Cabecera ──────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.heading("⚙ Configuracion");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("  Cancelar  ").clicked() {
                    cancel = true;
                }
                ui.add_space(4.0);
                if ui
                    .add(egui::Button::new("  Guardar  ").fill(egui::Color32::from_rgb(34, 139, 34)))
                    .clicked()
                {
                    save = true;
                }
            });
        });
        ui.separator();
        ui.add_space(6.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // ── General ───────────────────────────────────────────────────
            egui::CollapsingHeader::new(egui::RichText::new("General").strong())
                .default_open(true)
                .show(ui, |ui| {
                    egui::Grid::new("settings_general")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .min_col_width(180.0)
                        .show(ui, |ui| {
                            ui.label("Ruta de proyectos:");
                            ui.add(
                                egui::TextEdit::singleline(&mut draft.projects_path)
                                    .desired_width(300.0)
                                    .hint_text(r"C:\WSDD-Projects"),
                            );
                            ui.end_row();

                            ui.label("Docker Desktop (path):");
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

                            ui.label("WSL Distro:");
                            let distro_val =
                                draft.wsl_distro.get_or_insert_with(String::new);
                            ui.add(
                                egui::TextEdit::singleline(distro_val)
                                    .desired_width(200.0)
                                    .hint_text("Ubuntu-22.04 (opcional)"),
                            );
                            ui.end_row();

                            ui.label("Max lineas en log:");
                            ui.add(
                                egui::DragValue::new(&mut draft.log_max_lines)
                                    .range(100..=10000)
                                    .speed(10.0),
                            );
                            ui.end_row();

                            ui.label("Auto-iniciar contenedores:");
                            ui.checkbox(&mut draft.auto_start_containers, "Al arrancar WSDD");
                            ui.end_row();
                        });
                });

            ui.add_space(8.0);

            // ── PHP ───────────────────────────────────────────────────────
            egui::CollapsingHeader::new(egui::RichText::new("PHP — contenedores Docker").strong())
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(
                            "Estos valores se aplican al generar nuevos contenedores PHP. \
                             No afectan contenedores existentes.",
                        )
                        .size(11.0)
                        .color(ui.visuals().weak_text_color()),
                    );
                    ui.add_space(6.0);

                    egui::Grid::new("settings_php")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .min_col_width(180.0)
                        .show(ui, |ui| {
                            ui.label("memory_limit:");
                            ui.add(
                                egui::TextEdit::singleline(&mut draft.php_memory_limit)
                                    .desired_width(100.0)
                                    .hint_text("512M"),
                            );
                            ui.end_row();

                            ui.label("upload_max_filesize / post_max_size:");
                            ui.add(
                                egui::TextEdit::singleline(&mut draft.php_upload_max_filesize)
                                    .desired_width(100.0)
                                    .hint_text("256M"),
                            );
                            ui.end_row();

                            ui.label("Timezone:");
                            ui.add(
                                egui::TextEdit::singleline(&mut draft.php_timezone)
                                    .desired_width(200.0)
                                    .hint_text("UTC"),
                            );
                            ui.end_row();
                        });
                });

            ui.add_space(8.0);

            // ── Herramientas ──────────────────────────────────────────────
            egui::CollapsingHeader::new(
                egui::RichText::new("Herramientas integradas").strong(),
            )
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("settings_tools")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .min_col_width(180.0)
                    .show(ui, |ui| {
                        ui.label("Version de Webmin:");
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut draft.webmin_version)
                                    .desired_width(100.0)
                                    .hint_text("2.021"),
                            );
                            ui.label(
                                egui::RichText::new("(Dockerfiles PHP)")
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

    // ── Aplicar accion fuera del closure ─────────────────────────────────────
    if save {
        if let Some(mut draft) = app.ui.settings_draft.take() {
            // Normalizar strings vacíos a None en campos opcionales
            if draft.docker_path.as_deref() == Some("") {
                draft.docker_path = None;
            }
            if draft.wsl_distro.as_deref() == Some("") {
                draft.wsl_distro = None;
            }
            app.settings = draft;
            if let Err(e) = app.settings.save() {
                tracing::error!("Error guardando configuracion: {e}");
            }
        }
        app.ui.active = ActiveView::Main;
    } else if cancel {
        app.ui.settings_draft = None;
        app.ui.active = ActiveView::Main;
    }
}
