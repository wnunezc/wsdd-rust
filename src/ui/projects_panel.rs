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
//! Panel de proyectos WSDD.
//!
//! Muestra la tabla de proyectos con acciones Deploy, Remove y Toolbox.
//! Equivalente a `HandlerProject.Track()` + DataGridView en `Forms/Main.cs`.

use crate::app::WsddApp;
use crate::handlers::log_types::LogLine;
use crate::i18n::tr;
use crate::models::project::Project;
use crate::ui::ActiveView;

/// Renderiza la tabla de proyectos.
pub fn render(ui: &mut egui::Ui, app: &mut WsddApp) {
    if app.projects.is_empty() {
        ui.label(tr("projects_empty"));
        return;
    }

    let projects: Vec<Project> = app.projects.clone();

    enum PendingAction {
        Deploy(Project),
        Remove(String),
        Toolbox(String),
    }
    let mut pending: Option<PendingAction> = None;
    let col_name = tr("col_name");
    let col_domain = tr("col_domain");
    let col_php = tr("col_php");
    let col_ssl = tr("col_ssl");
    let col_deploy = tr("col_deploy");
    let col_remove = tr("col_remove");
    let col_toolbox = tr("col_toolbox");

    egui::Grid::new("projects_grid")
        .num_columns(7)
        .striped(true)
        .min_col_width(90.0)
        .show(ui, |ui| {
            ui.strong(col_name);
            ui.strong(col_domain);
            ui.strong(col_php);
            ui.strong(col_ssl);
            ui.strong(col_deploy.clone());
            ui.strong(col_remove.clone());
            ui.strong(col_toolbox.clone());
            ui.end_row();

            for p in &projects {
                ui.label(&p.name);
                ui.label(&p.domain);
                ui.label(p.php_version.display_name());
                ui.label(if p.ssl { "✓" } else { "—" });

                if ui.button(&col_deploy).clicked() && pending.is_none() {
                    pending = Some(PendingAction::Deploy(p.clone()));
                }
                if ui.button(&col_remove).clicked() && pending.is_none() {
                    pending = Some(PendingAction::Remove(p.name.clone()));
                }
                if ui.button(&col_toolbox).clicked() && pending.is_none() {
                    pending = Some(PendingAction::Toolbox(p.name.clone()));
                }

                ui.end_row();
            }
        });

    match pending {
        Some(PendingAction::Deploy(project)) => spawn_deploy(app, project),
        Some(PendingAction::Remove(name)) => {
            app.ui.confirm_remove_project = Some(name);
        }
        Some(PendingAction::Toolbox(name)) => {
            app.ui.toolbox_project_name = Some(name);
            app.ui.active = ActiveView::ToolboxProject;
        }
        None => {}
    }
}

/// Lanza el flujo de deploy completo en un background thread.
///
/// Llama a `handlers::deploy::deploy_project` que orquesta:
/// volumen → options.yml → rebuild PHP → vhost.conf → SSL (opc.) → hosts.
pub fn spawn_deploy(app: &mut WsddApp, project: Project) {
    let tx = app.main_log_tx.clone();
    let runner = app.runner.clone();

    std::thread::spawn(move || {
        if let Err(e) = crate::handlers::deploy::deploy_project(&project, &runner, &tx) {
            let _ = tx.send(LogLine::error(format!("[Deploy] Error: {e}")));
        }
    });
}

/// Elimina un proyecto: quita de la lista en memoria y lanza limpieza de infra en background.
///
/// Llamado desde `main_window` tras confirmación del usuario.
pub fn do_remove_project(app: &mut WsddApp, name: &str) {
    let project = match app.projects.iter().find(|p| p.name == name) {
        Some(p) => p.clone(),
        None => return,
    };

    // Quitar de la lista en memoria inmediatamente (feedback visual instantáneo)
    app.projects.retain(|p| p.name != name);

    let tx = app.main_log_tx.clone();
    let runner = app.runner.clone();
    std::thread::spawn(move || {
        if let Err(e) = crate::handlers::deploy::remove_project(&project, &runner, &tx) {
            let _ = tx.send(LogLine::error(format!("[Remove] Error: {e}")));
        }
    });
}
