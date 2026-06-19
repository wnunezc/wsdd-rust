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

use crate::app::{NgrokPublicationEvent, WsddApp};
use crate::handlers::log_types::LogLine;
use crate::handlers::ps_script::{launch, launch_url};
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
    crate::ui::render_modal_backdrop(ctx, "toolbox_project_backdrop");
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .min_width(360.0)
        .open(&mut open)
        .show(ctx, |ui| {
            let actions = tr("toolbox_actions");
            let project_info = tr("toolbox_project_info");
            let open_folder = tr("main_open_folder");
            let open_browser = tr("main_open_browser");
            let backup_project = tr("toolbox_backup_project");
            let ngrok_copy = NgrokToolboxCopy::new();
            let close_label = tr("btn_close");
            let info_name = format!("{}:", tr("col_name"));
            let info_domain = format!("{}:", tr("col_domain"));
            let info_php = format!("{}:", tr("col_php"));
            let info_work_path = format!("{}:", tr("info_work_path"));
            let info_storage_mode = format!("{}:", tr("info_storage_mode"));
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
                    launch_url(&url);
                }
            });

            ui.add_space(6.0);

            if ui.button(format!("💾 {backup_project}")).clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title(tr("project_backup_dialog_title"))
                    .add_filter("WSDD Backup", &["zip"])
                    .set_file_name(crate::handlers::backup::default_project_backup_name(
                        &project.name,
                    ))
                    .save_file()
                {
                    let tx = app.main_log_tx.clone();
                    let project_clone = project.clone();
                    let job_key = format!("backup:project:{}", project.name);
                    let label = format!("Backup project {}", project.name);
                    let _ = app.spawn_blocking_job(ctx, job_key, label, move || {
                        crate::handlers::backup::backup_project(&project_clone, &path, &tx).map_err(
                            |e| {
                                let message = format!("[Backup] Error: {e}");
                                let _ = tx.send(LogLine::error(message.clone()));
                                message
                            },
                        )
                    });
                }
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            ui.strong(&ngrok_copy.title);
            ui.add_space(4.0);
            render_ngrok_controls(ui, ctx, app, &project, &ngrok_copy);

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

                    ui.label(&info_storage_mode);
                    ui.label(storage_mode_display(project.storage_mode));
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

fn storage_mode_display(mode: crate::models::project::StorageMode) -> String {
    match mode {
        crate::models::project::StorageMode::ClassicBind => tr("storage_classic_bind"),
        crate::models::project::StorageMode::FastSyncWorkspace => tr("storage_fast_sync_workspace"),
        crate::models::project::StorageMode::NativeLinuxProject => {
            tr("storage_native_linux_project")
        }
    }
}

struct NgrokToolboxCopy {
    title: String,
    publish: String,
    stop: String,
    open: String,
    copy: String,
    unavailable: String,
}

impl NgrokToolboxCopy {
    fn new() -> Self {
        Self {
            title: tr("toolbox_ngrok_title"),
            publish: tr("toolbox_ngrok_publish"),
            stop: tr("toolbox_ngrok_stop"),
            open: tr("toolbox_ngrok_open"),
            copy: tr("toolbox_ngrok_copy"),
            unavailable: tr("toolbox_ngrok_unavailable"),
        }
    }
}

fn render_ngrok_controls(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    app: &mut WsddApp,
    project: &crate::models::project::Project,
    copy: &NgrokToolboxCopy,
) {
    let ngrok_running = app
        .containers
        .iter()
        .any(|container| container.name == "WSDD-Ngrok-Server" && container.is_running());
    let ngrok_ready = app.settings.optional_services.ngrok.enabled
        && !app.settings.ngrok_authtoken.trim().is_empty()
        && ngrok_running;
    let publish_key = format!("ngrok:publish:{}", project.name);
    let stop_key = format!("ngrok:stop:{}", project.name);
    let busy = app.is_job_running(&publish_key) || app.is_job_running(&stop_key);

    if let Some(publication) = app.ngrok_publications.get(&project.name).cloned() {
        ui.horizontal(|ui| {
            ui.label(&publication.public_url);
            if ui.button(&copy.open).clicked() {
                launch_url(&publication.public_url);
            }
            if ui.button(&copy.copy).clicked() {
                ui.output_mut(|o| o.copied_text = publication.public_url.clone());
            }
            if ui
                .add_enabled(!busy, egui::Button::new(&copy.stop))
                .clicked()
            {
                spawn_stop_publication(ctx, app, project.clone());
            }
        });
        return;
    }

    if !ngrok_ready {
        ui.add_enabled(false, egui::Button::new(&copy.publish))
            .on_hover_text(&copy.unavailable);
        return;
    }

    spawn_sync_publication_if_needed(ctx, app, project);

    if ui
        .add_enabled(!busy, egui::Button::new(&copy.publish))
        .clicked()
    {
        spawn_publish_project(ctx, app, project.clone());
    }
}

fn spawn_sync_publication_if_needed(
    ctx: &egui::Context,
    app: &mut WsddApp,
    project: &crate::models::project::Project,
) {
    let project_name = project.name.clone();
    if app.ngrok_publication_sync_attempted.contains(&project_name) {
        return;
    }

    let job_key = format!("ngrok:sync:{project_name}");
    if app.is_job_running(&job_key) {
        return;
    }

    app.ngrok_publication_sync_attempted
        .insert(project_name.clone());

    let settings = app.settings.clone();
    let tx = app.main_log_tx.clone();
    let event_tx = app.ngrok_publication_tx.clone();
    let project = project.clone();
    let label = format!("ngrok sync {project_name}");

    let _ = app.spawn_blocking_job(ctx, job_key, label, move || {
        match crate::handlers::ngrok::publication_for_project(&project, &settings) {
            Ok(Some(publication)) => {
                let _ = event_tx.send(NgrokPublicationEvent::Published(publication));
                Ok(())
            }
            Ok(None) => Ok(()),
            Err(e) => {
                let message = format!("[ngrok] Error leyendo publicacion activa: {e}");
                let _ = tx.send(LogLine::error(message.clone()));
                Err(message)
            }
        }
    });
}

fn spawn_publish_project(
    ctx: &egui::Context,
    app: &mut WsddApp,
    project: crate::models::project::Project,
) {
    let settings = app.settings.clone();
    let runner = app.runner.clone();
    let tx = app.main_log_tx.clone();
    let event_tx = app.ngrok_publication_tx.clone();
    let project_name = project.name.clone();
    let job_key = format!("ngrok:publish:{project_name}");
    let label = format!("ngrok publish {project_name}");

    let _ = app.spawn_blocking_job(ctx, job_key, label, move || {
        match crate::handlers::ngrok::publish_project(&project, &settings, &runner) {
            Ok(publication) => {
                let _ = tx.send(LogLine::success(format!(
                    "[ngrok] {} publicado en {}",
                    publication.project_name, publication.public_url
                )));
                let _ = event_tx.send(NgrokPublicationEvent::Published(publication));
                Ok(())
            }
            Err(e) => {
                let message = format!("[ngrok] Error publicando {project_name}: {e}");
                let _ = tx.send(LogLine::error(message.clone()));
                Err(message)
            }
        }
    });
}

fn spawn_stop_publication(
    ctx: &egui::Context,
    app: &mut WsddApp,
    project: crate::models::project::Project,
) {
    let settings = app.settings.clone();
    let tx = app.main_log_tx.clone();
    let event_tx = app.ngrok_publication_tx.clone();
    let project_name = project.name.clone();
    let job_key = format!("ngrok:stop:{project_name}");
    let label = format!("ngrok stop {project_name}");

    let _ = app.spawn_blocking_job(ctx, job_key, label, move || {
        match crate::handlers::ngrok::stop_project_publication(&project, &settings) {
            Ok(()) => {
                let _ = tx.send(LogLine::success(format!(
                    "[ngrok] Publicacion detenida para {project_name}"
                )));
                let _ = event_tx.send(NgrokPublicationEvent::Stopped { project_name });
                Ok(())
            }
            Err(e) => {
                let message = format!("[ngrok] Error deteniendo publicacion: {e}");
                let _ = tx.send(LogLine::error(message.clone()));
                Err(message)
            }
        }
    });
}
