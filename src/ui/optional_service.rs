//! Configuration and lifecycle dialog for one optional developer service.

use crate::app::{BackgroundJobStatus, WsddApp};
use crate::handlers::docker_deploy::{self, OptionalServiceKind};
use crate::handlers::log_types::LogLine;
use crate::handlers::ngrok;
use crate::handlers::ps_script::launch_url;
use crate::handlers::setting::AppSettings;
use crate::i18n::tr;
use crate::ui::ActiveView;

fn job_key(kind: OptionalServiceKind) -> String {
    format!("tool:{}", kind.service_name())
}

/// Renders one service's saved configuration and explicit Docker actions.
pub fn render(ctx: &egui::Context, app: &mut WsddApp, kind: OptionalServiceKind) {
    let key = job_key(kind);
    if app.ui.optional_service_draft.is_none() {
        app.ui.optional_service_draft = Some(app.settings.clone());
        app.ui.optional_service_error = None;
        app.ui.optional_service_notice = None;
        if !app.is_job_running(&key) {
            app.jobs.remove(&key);
            if kind == OptionalServiceKind::Ngrok
                && app.settings.optional_services.ngrok.enabled
                && app.containers.iter().any(|container| {
                    container.name == kind.container_name() && container.is_running()
                })
            {
                spawn_ngrok_check(ctx, app, key.clone());
            }
        }
    }

    let busy = app.is_job_running(&key);
    let container = app
        .containers
        .iter()
        .find(|container| container.name == kind.container_name());
    let container_status = match container {
        Some(container) if container.is_running() => tr("tool_status_running"),
        Some(_) => tr("tool_status_stopped"),
        None => tr("tool_status_absent"),
    };
    let operation = app.jobs.get(&key).map(|job| {
        (
            job.status.clone(),
            job.last_error.clone(),
            job.label.clone(),
        )
    });
    let latest_trace = app
        .operation_traces
        .iter()
        .rev()
        .find(|trace| trace.key.starts_with(&format!("{key}:")))
        .cloned();
    let mut save = false;
    let mut deploy = false;
    let mut stop = false;
    let mut check_api = false;
    let mut cancel = false;
    let mut open_log = false;
    let mut copy_log = false;
    let mut open = true;

    crate::ui::render_modal_backdrop(ctx, "optional_service_backdrop");
    egui::Window::new(format!("{}: {}", tr("menu_tools"), kind.display_name()))
        .collapsible(false)
        .resizable(false)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(520.0)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(format!(
                "{}: {container_status}",
                tr("tool_container_status")
            ));
            if kind == OptionalServiceKind::Ngrok {
                ui.label(tr("tool_ngrok_api_status_note"));
            } else {
                ui.label(tr("tool_live_validation_pending"));
            }
            if let Some(notice) = &app.ui.optional_service_notice {
                ui.label(notice);
            }
            if let Some(error) = &app.ui.optional_service_error {
                ui.colored_label(ui.visuals().error_fg_color, error);
            }
            if let Some((status, error, label)) = &operation {
                if app.ui.optional_service_notice.is_none() {
                    match status {
                        BackgroundJobStatus::Running => {
                            ui.label(format!("{label}: {}", tr("tool_operation_running")));
                        }
                        BackgroundJobStatus::Succeeded => {
                            ui.label(format!("{label}: {}", tr("tool_operation_success")));
                        }
                        BackgroundJobStatus::Failed => {
                            ui.colored_label(
                                ui.visuals().error_fg_color,
                                format!(
                                    "{label}: {}",
                                    error.as_deref().unwrap_or(&tr("tool_operation_failed"))
                                ),
                            );
                        }
                    }
                }
            }
            if let Some(trace) = &latest_trace {
                ui.separator();
                ui.label(egui::RichText::new(tr("trace_title")).strong());
                for stage in &trace.stages {
                    let elapsed = stage
                        .finished_at
                        .unwrap_or_else(std::time::Instant::now)
                        .duration_since(stage.started_at)
                        .as_secs_f32();
                    let symbol = match stage.finished_at {
                        None => "◌",
                        Some(_) if stage.success => "✓",
                        Some(_) => "✗",
                    };
                    ui.label(format!("{symbol} {} · {elapsed:.1} s", stage.label));
                }
                ui.horizontal(|ui| {
                    let exists = trace.log_path.is_file();
                    open_log = ui
                        .add_enabled(exists, egui::Button::new(tr("trace_open_log")))
                        .clicked();
                    copy_log = ui
                        .add_enabled(exists, egui::Button::new(tr("trace_copy_log")))
                        .clicked();
                });
                if trace.finished_at.is_none() {
                    ui.ctx()
                        .request_repaint_after(std::time::Duration::from_millis(250));
                }
            }
            ui.separator();
            if let Some(draft) = app.ui.optional_service_draft.as_mut() {
                render_fields(ui, draft, kind);
            }
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                save = ui
                    .add_enabled(!busy, egui::Button::new(tr("btn_save")))
                    .clicked();
                deploy = ui
                    .add_enabled(!busy, egui::Button::new(tr("col_deploy")))
                    .clicked();
                stop = ui
                    .add_enabled(!busy, egui::Button::new(tr("tool_stop")))
                    .clicked();
                if kind == OptionalServiceKind::Ngrok {
                    check_api = ui
                        .add_enabled(!busy, egui::Button::new(tr("tool_check_api")))
                        .clicked();
                }
                cancel = ui.button(tr("btn_cancel")).clicked();
            });
        });

    if deploy {
        match save_draft(app, kind, true) {
            Ok(settings) => {
                app.ui.optional_service_notice = None;
                let runner = app.runner.clone();
                let tx = app.main_log_tx.clone();
                let events = app.trace_tx.clone();
                let label = format!("Deploy {}", kind.display_name());
                let path = crate::handlers::deploy_trace::operation_log_path();
                let trace_key = format!("{key}:{}", path.display());
                let worker_key = trace_key.clone();
                let worker_path = path.clone();
                if app.spawn_blocking_job(ctx, key, label, move || {
                    crate::handlers::deploy_trace::run_logged(
                        worker_key,
                        worker_path,
                        tx,
                        events,
                        |log, trace| {
                            docker_deploy::deploy_optional_service_traced_sync(
                                &runner, &settings, kind, log, trace,
                            )
                            .map_err(|error| {
                                let message =
                                    format!("[Deploy {}] Error: {error}", kind.display_name());
                                let _ = log.send(LogLine::error(&message));
                                message
                            })
                        },
                    )
                }) {
                    app.track_operation(
                        trace_key,
                        format!("Herramienta {}", kind.display_name()),
                        path,
                    );
                }
            }
            Err(error) => app.ui.optional_service_error = Some(error),
        }
    } else if save {
        match save_draft(app, kind, false) {
            Ok(_) => app.ui.optional_service_notice = Some(tr("tool_saved_pending_deploy")),
            Err(error) => app.ui.optional_service_error = Some(error),
        }
    } else if stop {
        let runner = app.runner.clone();
        let tx = app.main_log_tx.clone();
        let label = format!("{} {}", tr("tool_stop"), kind.display_name());
        app.spawn_blocking_job(ctx, key, label, move || {
            docker_deploy::stop_optional_service_sync(&runner, kind, &tx)
                .map_err(|error| error.to_string())
        });
        app.ui.optional_service_notice = None;
        app.ui.optional_service_error = None;
    } else if check_api {
        app.ui.optional_service_notice = None;
        spawn_ngrok_check(ctx, app, key);
    } else if cancel || !open {
        app.ui.optional_service_draft = None;
        app.ui.optional_service_error = None;
        app.ui.optional_service_notice = None;
        app.ui.active = ActiveView::Main;
    }

    if let Some(trace) = &latest_trace {
        if open_log {
            if let Err(error) = crate::handlers::external_app::open_text_file(&trace.log_path) {
                app.ui.optional_service_error = Some(error.to_string());
            }
        } else if copy_log {
            match std::fs::read_to_string(&trace.log_path) {
                Ok(contents) => {
                    ctx.output_mut(|output| {
                        output.copied_text = crate::handlers::diagnostics::redact(&contents)
                    });
                }
                Err(error) => app.ui.optional_service_error = Some(error.to_string()),
            }
        }
    }
}

fn render_fields(ui: &mut egui::Ui, draft: &mut AppSettings, kind: OptionalServiceKind) {
    egui::Grid::new(format!("tool_fields_{}", kind.service_name()))
        .num_columns(2)
        .spacing([12.0, 8.0])
        .show(ui, |ui| match kind {
            OptionalServiceKind::Ngrok => {
                ui.label(tr("settings_ngrok_authtoken"));
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut draft.ngrok_authtoken)
                            .password(true)
                            .desired_width(230.0),
                    );
                    if ui.button(tr("settings_ngrok_open_dashboard")).clicked() {
                        launch_url("https://dashboard.ngrok.com/get-started/your-authtoken");
                    }
                });
                ui.end_row();
                ui.label(tr("settings_ngrok_api_port"));
                ui.add(
                    egui::DragValue::new(&mut draft.optional_services.ngrok.api_host_port)
                        .range(1..=65535),
                );
                ui.end_row();
                ui.label(tr("settings_ngrok_inspect"));
                ui.checkbox(&mut draft.optional_services.ngrok.inspect, "");
                ui.end_row();
                ui.label(tr("tool_restart_with_docker"));
                ui.checkbox(&mut draft.optional_services.ngrok.auto_start, "");
                ui.end_row();
            }
            OptionalServiceKind::Redis => {
                ui.label(tr("settings_redis_port"));
                ui.add(
                    egui::DragValue::new(&mut draft.optional_services.redis.host_port)
                        .range(1..=65535),
                );
                ui.end_row();
                ui.label(tr("tool_restart_with_docker"));
                ui.checkbox(&mut draft.optional_services.redis.auto_start, "");
                ui.end_row();
            }
            OptionalServiceKind::Memcached => {
                ui.label(tr("settings_memcached_port"));
                ui.add(
                    egui::DragValue::new(&mut draft.optional_services.memcached.host_port)
                        .range(1..=65535),
                );
                ui.end_row();
                ui.label(tr("settings_memcached_memory"));
                ui.add(
                    egui::DragValue::new(&mut draft.optional_services.memcached.memory_limit_mb)
                        .range(1..=65535),
                );
                ui.end_row();
                ui.label(tr("tool_restart_with_docker"));
                ui.checkbox(&mut draft.optional_services.memcached.auto_start, "");
                ui.end_row();
            }
            OptionalServiceKind::Mailpit => {
                ui.label(tr("settings_mailpit_smtp_port"));
                ui.add(
                    egui::DragValue::new(&mut draft.optional_services.mailpit.smtp_host_port)
                        .range(1..=65535),
                );
                ui.end_row();
                ui.label(tr("settings_mailpit_ui_port"));
                ui.add(
                    egui::DragValue::new(&mut draft.optional_services.mailpit.ui_host_port)
                        .range(1..=65535),
                );
                ui.end_row();
                ui.label(tr("settings_mailpit_virtual_host"));
                ui.add(
                    egui::TextEdit::singleline(&mut draft.optional_services.mailpit.virtual_host)
                        .desired_width(220.0),
                );
                ui.end_row();
                ui.label(tr("tool_restart_with_docker"));
                ui.checkbox(&mut draft.optional_services.mailpit.auto_start, "");
                ui.end_row();
            }
        });
}

fn save_draft(
    app: &mut WsddApp,
    kind: OptionalServiceKind,
    deploy: bool,
) -> Result<AppSettings, String> {
    let draft = app
        .ui
        .optional_service_draft
        .as_ref()
        .ok_or_else(|| tr("tool_operation_failed"))?;
    let next = updated_settings(&app.settings, draft, kind, deploy)?;
    if kind == OptionalServiceKind::Ngrok {
        next.save().map_err(|error| error.to_string())?;
    } else {
        next.save_public_config()
            .map_err(|error| error.to_string())?;
    }
    app.settings = next.clone();
    app.ui.optional_service_draft = Some(next.clone());
    app.ui.optional_service_error = None;
    Ok(next)
}

fn updated_settings(
    current: &AppSettings,
    draft: &AppSettings,
    kind: OptionalServiceKind,
    deploy: bool,
) -> Result<AppSettings, String> {
    let mut next = current.clone();
    match kind {
        OptionalServiceKind::Ngrok => {
            next.optional_services.ngrok = draft.optional_services.ngrok.clone();
            next.ngrok_authtoken = draft.ngrok_authtoken.trim().to_string();
            if deploy {
                next.optional_services.ngrok.enabled = true;
            }
        }
        OptionalServiceKind::Redis => {
            next.optional_services.redis = draft.optional_services.redis.clone();
            if deploy {
                next.optional_services.redis.enabled = true;
            }
        }
        OptionalServiceKind::Memcached => {
            next.optional_services.memcached = draft.optional_services.memcached.clone();
            if deploy {
                next.optional_services.memcached.enabled = true;
            }
        }
        OptionalServiceKind::Mailpit => {
            next.optional_services.mailpit = draft.optional_services.mailpit.clone();
            next.optional_services.mailpit.virtual_host = next
                .optional_services
                .mailpit
                .virtual_host
                .trim()
                .to_string();
            if deploy {
                next.optional_services.mailpit.enabled = true;
            }
        }
    }
    next.validate_optional_services()
        .map_err(|error| error.to_string())?;
    Ok(next)
}

fn spawn_ngrok_check(ctx: &egui::Context, app: &mut WsddApp, key: String) {
    let settings = app.settings.clone();
    let runner = app.runner.clone();
    app.spawn_blocking_job(ctx, key, tr("tool_check_api"), move || {
        let status = ngrok::agent_status(&settings, &runner).map_err(|error| error.to_string())?;
        if status.api_reachable {
            Ok(())
        } else {
            Err(tr("tool_ngrok_api_unreachable"))
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saving_one_tool_preserves_other_services_and_secrets() {
        let mut current = AppSettings::default();
        current.ngrok_authtoken = "existing-token".into();
        current.optional_services.redis.host_port = 6380;
        let mut draft = current.clone();
        draft.optional_services.redis.host_port = 6381;
        draft.optional_services.mailpit.ui_host_port = 9999;
        draft.ngrok_authtoken = "changed-token".into();

        let next = updated_settings(&current, &draft, OptionalServiceKind::Redis, false)
            .expect("valid Redis settings");
        assert_eq!(next.optional_services.redis.host_port, 6381);
        assert!(!next.optional_services.redis.enabled);
        assert_eq!(
            next.optional_services.mailpit,
            current.optional_services.mailpit
        );
        assert_eq!(next.ngrok_authtoken, "existing-token");
    }

    #[test]
    fn deploying_ngrok_requires_token_and_marks_it_available() {
        let current = AppSettings::default();
        let mut draft = current.clone();
        assert!(updated_settings(&current, &draft, OptionalServiceKind::Ngrok, true).is_err());
        draft.ngrok_authtoken = " token ".into();
        let next = updated_settings(&current, &draft, OptionalServiceKind::Ngrok, true)
            .expect("valid ngrok settings");
        assert!(next.optional_services.ngrok.enabled);
        assert_eq!(next.ngrok_authtoken, "token");
    }
}
