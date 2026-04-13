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

use crate::app::WsddApp;
use crate::handlers::log_types::LogLine;
use crate::i18n::tr;
use crate::models::project::Project;
use crate::ui::ActiveView;

pub enum DeployFlowOutcome {
    Started,
    WaitingForCredentials,
    Failed,
}

pub fn render(ui: &mut egui::Ui, app: &mut WsddApp) {
    ui.spacing_mut().item_spacing = egui::vec2(14.0, 9.0);

    if app.projects.is_empty() {
        ui.add_space(4.0);
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
        .spacing([18.0, 10.0])
        .show(ui, |ui| {
            ui.strong(col_name);
            ui.strong(col_domain);
            ui.strong(col_php);
            ui.strong(col_ssl);
            ui.strong(col_deploy.clone());
            ui.strong(col_remove.clone());
            ui.strong(col_toolbox.clone());
            ui.end_row();

            for project in &projects {
                let project_busy = app.is_job_running(&project_job_key(&project.name));

                ui.label(egui::RichText::new(&project.name).strong());
                ui.label(&project.domain);
                ui.label(project.php_version.display_name());
                ui.label(if project.ssl { "\u{2713}" } else { "\u{2014}" });

                if ui
                    .add_enabled(!project_busy, egui::Button::new(&col_deploy))
                    .clicked()
                    && pending.is_none()
                {
                    pending = Some(PendingAction::Deploy(project.clone()));
                }

                if ui
                    .add_enabled(!project_busy, egui::Button::new(&col_remove))
                    .clicked()
                    && pending.is_none()
                {
                    pending = Some(PendingAction::Remove(project.name.clone()));
                }

                if ui.button(&col_toolbox).clicked() && pending.is_none() {
                    pending = Some(PendingAction::Toolbox(project.name.clone()));
                }

                ui.end_row();
            }
        });

    match pending {
        Some(PendingAction::Deploy(project)) => {
            let _ = prepare_deploy(ui.ctx(), app, project, false);
        }
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

pub fn prepare_deploy(
    ctx: &egui::Context,
    app: &mut WsddApp,
    project: Project,
    add_project_to_list: bool,
) -> DeployFlowOutcome {
    if app
        .settings
        .webmin_credentials_for(&project.php_version)
        .is_some()
    {
        if add_project_to_list {
            add_project_to_memory_if_missing(app, &project);
        }
        spawn_deploy(ctx, app, project);
        return DeployFlowOutcome::Started;
    }

    match crate::handlers::docker::php_container_exists_sync(
        &app.runner,
        project.php_version.container_tag(),
    ) {
        Ok(true) => {
            let adopted = app
                .settings
                .ensure_legacy_webmin_credentials(project.php_version.clone());

            if adopted {
                if let Err(e) = app.settings.save() {
                    let message =
                        format!("No se pudo guardar las credenciales legacy de Webmin: {e}");
                    app.ui.form_error = Some(message.clone());
                    let _ = app
                        .main_log_tx
                        .send(LogLine::error(format!("[Deploy] {message}")));
                    return DeployFlowOutcome::Failed;
                }

                let _ = app.main_log_tx.send(LogLine::warn(format!(
                    "[Deploy] {} ya existia; se conservaran credenciales Webmin legacy admin/admin para compatibilidad.",
                    project.php_version.display_name()
                )));
            }

            if add_project_to_list {
                add_project_to_memory_if_missing(app, &project);
            }
            spawn_deploy(ctx, app, project);
            DeployFlowOutcome::Started
        }
        Ok(false) => {
            app.ui.open_webmin_prompt(project, add_project_to_list);
            DeployFlowOutcome::WaitingForCredentials
        }
        Err(e) => {
            let message = format!(
                "No se pudo verificar el contenedor {}: {e}",
                project.php_version.display_name()
            );
            app.ui.form_error = Some(message.clone());
            let _ = app
                .main_log_tx
                .send(LogLine::error(format!("[Deploy] {message}")));
            DeployFlowOutcome::Failed
        }
    }
}

pub fn spawn_deploy(ctx: &egui::Context, app: &mut WsddApp, project: Project) {
    let tx = app.main_log_tx.clone();
    let runner = app.runner.clone();
    let settings = app.settings.clone();
    let project_name = project.name.clone();
    let job_key = project_job_key(&project_name);
    let label = format!("Deploy project {}", project_name);

    let _ = app.spawn_blocking_job(ctx, job_key, label, move || {
        crate::handlers::deploy::deploy_project(&project, &settings, &runner, &tx).map_err(|e| {
            let message = format!("[Deploy] Error: {e}");
            let _ = tx.send(LogLine::error(message.clone()));
            message
        })
    });
}

pub fn do_remove_project(ctx: &egui::Context, app: &mut WsddApp, name: &str) {
    let project = match app.projects.iter().find(|p| p.name == name) {
        Some(project) => project.clone(),
        None => return,
    };

    app.projects.retain(|project_ref| project_ref.name != name);

    let tx = app.main_log_tx.clone();
    let runner = app.runner.clone();
    let job_key = project_job_key(&project.name);
    let label = format!("Remove project {}", project.name);

    let _ = app.spawn_blocking_job(ctx, job_key, label, move || {
        crate::handlers::deploy::remove_project(&project, &runner, &tx).map_err(|e| {
            let message = format!("[Remove] Error: {e}");
            let _ = tx.send(LogLine::error(message.clone()));
            message
        })
    });
}

fn add_project_to_memory_if_missing(app: &mut WsddApp, project: &Project) {
    if !app
        .projects
        .iter()
        .any(|existing| existing.name == project.name)
    {
        app.projects.push(project.clone());
    }
}

fn project_job_key(name: &str) -> String {
    format!("project:{name}")
}
