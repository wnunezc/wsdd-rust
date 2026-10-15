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
use crate::i18n::{tr, trf};
use crate::models::project::Project;
use crate::ui::ActiveView;

pub(crate) mod view;
use view::ProjectColumn;

pub enum DeployFlowOutcome {
    Started,
    WaitingForCredentials,
    Failed,
}

pub fn render(ui: &mut egui::Ui, app: &mut WsddApp) {
    ui.spacing_mut().item_spacing = egui::vec2(14.0, 6.0);
    app.ui
        .selected_projects
        .retain(|name| app.projects.iter().any(|p| &p.name == name));

    if app.projects.is_empty() {
        ui.add_space(4.0);
        ui.label(tr("projects_empty"));
        return;
    }

    let busy = project_operations_busy(app);
    ui.horizontal(|ui| {
        let button_label = trf(
            "projects_remove_selected",
            &[("count", &app.ui.selected_projects.len().to_string())],
        );
        let button_width = ui
            .painter()
            .layout_no_wrap(
                button_label,
                egui::TextStyle::Button.resolve(ui.style()),
                ui.visuals().text_color(),
            )
            .size()
            .x
            + 2.0 * ui.spacing().button_padding.x;
        let filter_label = tr("projects_filter");
        let label_width = ui
            .painter()
            .layout_no_wrap(
                filter_label.clone(),
                egui::TextStyle::Body.resolve(ui.style()),
                ui.visuals().text_color(),
            )
            .size()
            .x;
        let field_width =
            (ui.available_width() - button_width - label_width - 3.0 * ui.spacing().item_spacing.x)
                .clamp(80.0, 320.0);
        ui.label(filter_label);
        let response = ui.add_sized(
            [field_width, 28.0],
            egui::TextEdit::singleline(&mut app.ui.project_table.filter).margin(egui::Margin {
                left: 6.0,
                right: 28.0,
                top: 5.0,
                bottom: 5.0,
            }),
        );
        let clear_rect = egui::Rect::from_center_size(
            egui::pos2(response.rect.right() - 14.0, response.rect.center().y),
            egui::vec2(24.0, 24.0),
        );
        if ui
            .put(clear_rect, egui::Button::new("\u{00D7}").frame(false))
            .on_hover_text(tr("projects_clear_filter"))
            .clicked()
        {
            app.ui.project_table.filter.clear();
        }
        let visible = app.ui.project_table.visible_projects(&app.projects);
        app.ui
            .selected_projects
            .retain(|name| visible.iter().any(|p| &p.name == name));
        let selected_count = app.ui.selected_projects.len();
        let count = selected_count.to_string();
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_enabled(
                    !busy && selected_count > 0,
                    egui::Button::new(trf("projects_remove_selected", &[("count", &count)])),
                )
                .clicked()
            {
                app.ui.confirm_remove_projects = Some(crate::ui::state::RemovalRequest {
                    project_names: app.ui.selected_projects.iter().cloned().collect(),
                    php_container: None,
                });
            }
        });
    });
    ui.add_space(8.0);
    let projects = app.ui.project_table.visible_projects(&app.projects);
    let mut pending = None;
    egui_extras::TableBuilder::new(ui)
        .id_salt("projects_table")
        .striped(true)
        .vscroll(false)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(egui_extras::Column::exact(28.0))
        .column(egui_extras::Column::remainder().clip(true))
        .column(egui_extras::Column::remainder().clip(true))
        .column(egui_extras::Column::exact(82.0))
        .column(egui_extras::Column::exact(88.0))
        .header(30.0, |mut header| {
            header.col(|ui| {
                let mut all_selected = !projects.is_empty()
                    && projects
                        .iter()
                        .all(|p| app.ui.selected_projects.contains(&p.name));
                if ui
                    .add_enabled(!busy, egui::Checkbox::without_text(&mut all_selected))
                    .on_hover_text(tr("projects_select_all"))
                    .changed()
                {
                    if all_selected {
                        app.ui
                            .selected_projects
                            .extend(projects.iter().map(|p| p.name.clone()));
                    } else {
                        app.ui
                            .selected_projects
                            .retain(|name| !projects.iter().any(|p| &p.name == name));
                    }
                }
            });
            for (column, label) in [
                (ProjectColumn::Name, tr("col_name")),
                (ProjectColumn::Domain, tr("col_domain")),
                (ProjectColumn::Php, tr("col_php")),
            ] {
                header.col(|ui| {
                    let arrow = if app.ui.project_table.column != Some(column) {
                        " \u{2195}"
                    } else if app.ui.project_table.descending {
                        " \u{25BC}"
                    } else {
                        " \u{25B2}"
                    };
                    let response = ui
                        .scope(|ui| {
                            ui.spacing_mut().button_padding.x = 0.0;
                            ui.add(egui::SelectableLabel::new(
                                app.ui.project_table.column == Some(column),
                                egui::RichText::new(format!("{label}{arrow}")).strong(),
                            ))
                        })
                        .inner;
                    if response.clicked() {
                        app.ui.project_table.sort_by(column);
                        ui.ctx().request_repaint();
                    }
                });
            }
            header.col(|ui| {
                ui.strong(tr("col_toolbox"));
            });
        })
        .body(|body| {
            body.rows(24.0, projects.len(), |mut row| {
                let project = &projects[row.index()];
                row.col(|ui| {
                    let mut selected = app.ui.selected_projects.contains(&project.name);
                    if ui
                        .add_enabled(!busy, egui::Checkbox::without_text(&mut selected))
                        .changed()
                    {
                        if selected {
                            app.ui.selected_projects.insert(project.name.clone());
                        } else {
                            app.ui.selected_projects.remove(&project.name);
                        }
                    }
                });
                row.col(|ui| {
                    ui.add(
                        egui::Label::new(egui::RichText::new(&project.name).strong()).truncate(),
                    )
                    .on_hover_text(&project.name);
                });
                row.col(|ui| {
                    ui.add(egui::Label::new(&project.domain).truncate())
                        .on_hover_text(&project.domain);
                });
                row.col(|ui| {
                    ui.label(project.php_version.display_name());
                });
                row.col(|ui| {
                    if ui.button(tr("col_toolbox")).clicked() {
                        pending = Some(project.name.clone());
                    }
                });
            });
        });
    if projects.is_empty() {
        ui.label(tr("projects_no_matches"));
    }

    if let Some(name) = pending {
        app.ui.toolbox_project_name = Some(name);
        app.ui.active = ActiveView::ToolboxProject;
    }
}

pub fn prepare_deploy(
    ctx: &egui::Context,
    app: &mut WsddApp,
    project: Project,
    add_project_to_list: bool,
) -> DeployFlowOutcome {
    if app.is_job_running(REMOVE_PROJECTS_JOB) {
        return DeployFlowOutcome::Failed;
    }
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
        project.php_version.runtime_container_tag(),
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
    if app.is_job_running(REMOVE_PROJECTS_JOB) {
        return;
    }
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

pub fn do_remove_projects(
    ctx: &egui::Context,
    app: &mut WsddApp,
    request: crate::ui::state::RemovalRequest,
) {
    if project_operations_busy(app) {
        return;
    }
    let tx = app.main_log_tx.clone();
    let runner = app.runner.clone();
    let _ = app.spawn_blocking_job(ctx, REMOVE_PROJECTS_JOB, "Remove projects", move || {
        crate::handlers::deploy::remove_projects(
            &request.project_names,
            request.php_container.as_deref(),
            &runner,
            &tx,
        )
        .map_err(|e| {
            let message = format!("[Remove] Error: {e}");
            let _ = tx.send(LogLine::error(message.clone()));
            message
        })
    });
}

pub(crate) const REMOVE_PROJECTS_JOB: &str = "projects:remove";

pub(crate) fn project_operations_busy(app: &WsddApp) -> bool {
    app.jobs.keys().any(|key| {
        (key.starts_with("project:") || key.starts_with("container:") || key == REMOVE_PROJECTS_JOB)
            && app.is_job_running(key)
    })
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

pub(crate) fn project_job_key(name: &str) -> String {
    format!("project:{name}")
}
