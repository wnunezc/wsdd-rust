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
//! Formulario de alta de nuevo proyecto WSDD.
//!
//! Equivalente a `Forms/AddNewProject.cs`.
//! El estado del formulario vive en `UiState` (campos `form_*`).
//! Al confirmar se lanza el deploy completo en un background thread.

use std::sync::mpsc;

use crate::app::WsddApp;
use crate::handlers::log_types::LogLine;
use crate::models::project::{EntryPoint, PhpVersion, Project, ProjectStatus, normalize_domain};
use crate::ui::ActiveView;

/// Renderiza el formulario de nuevo proyecto.
pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    // Drena el canal del file dialog (si hay resultado pendiente)
    let mut picked_path: Option<String> = None;
    if let Some(ref rx) = app.ui.folder_pick_rx {
        if let Ok(result) = rx.try_recv() {
            picked_path = result;
            app.ui.folder_pick_rx = None;
        }
    }
    if let Some(path) = picked_path {
        app.ui.form_work_path = path;
    }

    let mut open = true;
    egui::Window::new("Agregar nuevo proyecto")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .min_width(440.0)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.add_space(4.0);

            egui::Grid::new("add_project_form")
                .num_columns(2)
                .spacing([12.0, 10.0])
                .show(ui, |ui| {
                    // ── Nombre ────────────────────────────────────────────────
                    ui.label("Nombre:");
                    ui.text_edit_singleline(&mut app.ui.form_name);
                    ui.end_row();

                    // ── Dominio — sufijo .dock visible ────────────────────────
                    ui.label("Dominio:");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut app.ui.form_domain)
                            .on_hover_text("Ejemplo: miapp  →  se guardará como miapp.dock");
                        ui.label(
                            egui::RichText::new(".dock")
                                .monospace()
                                .color(egui::Color32::from_rgb(100, 180, 255)),
                        );
                    });
                    ui.end_row();

                    // ── PHP ───────────────────────────────────────────────────
                    ui.label("PHP:");
                    egui::ComboBox::from_id_salt("php_version_combo")
                        .selected_text(app.ui.form_php.display_name())
                        .show_ui(ui, |ui| {
                            for v in PhpVersion::all() {
                                let label = v.display_name();
                                ui.selectable_value(&mut app.ui.form_php, v, label);
                            }
                        });
                    ui.end_row();

                    // ── Work Path — con botón Explorar ────────────────────────
                    ui.label("Work Path:");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut app.ui.form_work_path)
                            .on_hover_text("Ruta absoluta al directorio del código fuente");

                        if ui.button("Explorar...").clicked() && app.ui.folder_pick_rx.is_none() {
                            let (tx, rx) = mpsc::channel::<Option<String>>();
                            app.ui.folder_pick_rx = Some(rx);
                            let ctx_clone = ctx.clone();
                            std::thread::spawn(move || {
                                let result = rfd::FileDialog::new()
                                    .set_title("Seleccionar directorio del proyecto")
                                    .pick_folder()
                                    .map(|p| p.to_string_lossy().to_string());
                                let _ = tx.send(result);
                                ctx_clone.request_repaint();
                            });
                        }
                    });
                    ui.end_row();

                    // ── Entry Point ───────────────────────────────────────────
                    ui.label("Entry Point:");
                    ui.vertical(|ui| {
                        ui.radio_value(
                            &mut app.ui.form_entry,
                            EntryPoint::Root,
                            "Raíz del proyecto",
                        );
                        ui.radio_value(
                            &mut app.ui.form_entry,
                            EntryPoint::Public,
                            "/public  (Laravel, Symfony)",
                        );
                        ui.horizontal(|ui| {
                            let custom_selected = matches!(app.ui.form_entry, EntryPoint::Custom(_));
                            if ui.radio(custom_selected, "Personalizado:").clicked() {
                                app.ui.form_entry =
                                    EntryPoint::Custom(app.ui.form_entry_custom.clone());
                            }
                            let resp = ui.add_enabled(
                                custom_selected,
                                egui::TextEdit::singleline(&mut app.ui.form_entry_custom)
                                    .hint_text("/subdir"),
                            );
                            if resp.changed() {
                                app.ui.form_entry =
                                    EntryPoint::Custom(app.ui.form_entry_custom.clone());
                            }
                        });
                    });
                    ui.end_row();

                    // ── SSL ───────────────────────────────────────────────────
                    ui.label("SSL:");
                    ui.checkbox(
                        &mut app.ui.form_ssl,
                        "Generar certificado SSL con mkcert",
                    );
                    ui.end_row();
                });

            ui.add_space(12.0);

            // ── Error ─────────────────────────────────────────────────────────
            if let Some(ref err) = app.ui.form_error.clone() {
                ui.colored_label(egui::Color32::from_rgb(220, 80, 80), err);
                ui.add_space(8.0);
            }

            ui.horizontal(|ui| {
                if ui.button("Desplegar").clicked() {
                    try_submit(app);
                }
                if ui.button("Cancelar").clicked() {
                    app.ui.active = ActiveView::Main;
                }
            });
        });

    if !open {
        app.ui.active = ActiveView::Main;
    }
}

/// Valida el formulario y lanza el deploy si todo es correcto.
fn try_submit(app: &mut WsddApp) {
    let name = app.ui.form_name.trim().to_string();
    let domain_raw = app.ui.form_domain.trim().to_string();
    let work_path = app.ui.form_work_path.trim().to_string();

    if name.is_empty() {
        app.ui.form_error = Some("El nombre no puede estar vacío.".to_string());
        return;
    }
    if domain_raw.is_empty() {
        app.ui.form_error = Some("El dominio no puede estar vacío.".to_string());
        return;
    }
    // Whitelist: solo caracteres válidos en un nombre de dominio.
    // Previene inyección en comandos PS1 (mkcert, docker-compose).
    let domain_base = domain_raw
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.")
        .trim_end_matches('/')
        .trim_end_matches(".com")
        .trim_end_matches(".net")
        .trim_end_matches(".local")
        .trim_end_matches(".dock");
    if !domain_base.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.') {
        app.ui.form_error = Some(
            "El dominio solo puede contener letras, números, guiones y puntos.".to_string(),
        );
        return;
    }
    if work_path.is_empty() {
        app.ui.form_error = Some("El Work Path no puede estar vacío.".to_string());
        return;
    }
    if !std::path::Path::new(&work_path).exists() {
        app.ui.form_error = Some(format!("El directorio no existe: {work_path}"));
        return;
    }
    if crate::handlers::project::exists(&name) {
        app.ui.form_error =
            Some(format!("Ya existe un proyecto con el nombre '{name}'."));
        return;
    }

    let domain = normalize_domain(&domain_raw);

    if app.projects.iter().any(|p| p.domain == domain) {
        app.ui.form_error =
            Some(format!("Ya existe un proyecto con el dominio '{domain}'."));
        return;
    }

    let entry_point = match &app.ui.form_entry {
        EntryPoint::Custom(_) => {
            EntryPoint::Custom(app.ui.form_entry_custom.trim().to_string())
        }
        other => other.clone(),
    };

    let project = Project {
        name: name.clone(),
        domain,
        php_version: app.ui.form_php.clone(),
        work_path,
        entry_point,
        ssl: app.ui.form_ssl,
        status: ProjectStatus::default(),
    };

    app.ui.form_error = None;

    let tx = app.main_log_tx.clone();
    let runner = app.runner.clone();
    let proj_clone = project.clone();
    std::thread::spawn(move || {
        if let Err(e) = crate::handlers::deploy::deploy_project(&proj_clone, &runner, &tx) {
            let _ = tx.send(LogLine::error(format!("[Deploy] Error: {e}")));
        }
    });

    app.projects.push(project);
    app.ui.active = ActiveView::Main;
}
