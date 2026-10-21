//! Manual controls for the isolated two-instance MySQL experiment.

use crate::app::{BackgroundJobStatus, WsddApp};
use crate::handlers::mysql_test;
use crate::handlers::ps_script::PsRunner;
use crate::i18n::tr;
use crate::ui::ActiveView;

pub(crate) const JOB_KEY: &str = "tool:mysql-test";

#[derive(Clone, Copy)]
enum Action {
    Deploy,
    Recreate,
    Validate,
}

impl Action {
    fn label(self) -> &'static str {
        match self {
            Self::Deploy => "Deploy",
            Self::Recreate => "Recreate",
            Self::Validate => "Validate",
        }
    }

    fn run(
        self,
        runner: &PsRunner,
        log: &crate::handlers::log_types::LogSender,
        trace: &crate::handlers::deploy_trace::TraceHandle,
    ) -> Result<(), crate::errors::InfraError> {
        match self {
            Self::Deploy => mysql_test::deploy(runner, log, trace),
            Self::Recreate => mysql_test::recreate(runner, log, trace),
            Self::Validate => mysql_test::validate(runner, log, trace),
        }
    }
}

pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    let busy = app.is_job_running(JOB_KEY)
        || app.is_job_running(&crate::ui::containers_panel::container_job_key(
            mysql_test::CONTAINER,
        ))
        || app.is_job_running("backup:environment")
        || app.is_job_running("restore:environment");
    let container_status = match app
        .containers
        .iter()
        .find(|container| container.name == mysql_test::CONTAINER)
    {
        Some(container) if container.is_running() => tr("tool_status_running"),
        Some(_) => tr("tool_status_stopped"),
        None => tr("tool_status_absent"),
    };
    let latest_trace = app
        .operation_traces
        .iter()
        .rev()
        .find(|trace| trace.key.starts_with(&format!("{JOB_KEY}:")))
        .cloned();
    let mut action = None;
    let mut open_log = false;
    let mut copy_log = false;
    let mut open = true;

    crate::ui::render_modal_backdrop(ctx, "mysql_test_backdrop");
    egui::Window::new(tr("mysql_test_title"))
        .collapsible(false)
        .resizable(false)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(570.0)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(format!(
                "{}: {container_status}",
                tr("tool_container_status")
            ));
            ui.label(tr("mysql_test_primary"));
            ui.label(format!(
                "{}: {}",
                tr("mysql_test_replica"),
                mysql_test::data_dir().display()
            ));
            ui.label(tr("mysql_test_isolation"));

            if let Some(job) = app.jobs.get(JOB_KEY) {
                match job.status {
                    BackgroundJobStatus::Running => {
                        ui.label(format!("{}: {}", job.label, tr("tool_operation_running")));
                    }
                    BackgroundJobStatus::Succeeded => {
                        ui.label(format!("{}: {}", job.label, tr("tool_operation_success")));
                    }
                    BackgroundJobStatus::Failed => {
                        ui.colored_label(
                            ui.visuals().error_fg_color,
                            job.last_error
                                .as_deref()
                                .unwrap_or(&tr("mysql_test_failure")),
                        );
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
                if let Some(step) = &trace.buildkit_step {
                    ui.label(step);
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
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!busy, egui::Button::new(tr("col_deploy")))
                    .clicked()
                {
                    action = Some(Action::Deploy);
                }
                if ui
                    .add_enabled(!busy, egui::Button::new(tr("mysql_test_recreate")))
                    .clicked()
                {
                    action = Some(Action::Recreate);
                }
                if ui
                    .add_enabled(!busy, egui::Button::new(tr("mysql_test_validate")))
                    .clicked()
                {
                    action = Some(Action::Validate);
                }
                if ui.button(tr("btn_cancel")).clicked() {
                    app.ui.active = ActiveView::Main;
                }
            });
        });

    if !open {
        app.ui.active = ActiveView::Main;
    }
    if let Some(action) = action {
        let runner = app.runner.clone();
        let tx = app.main_log_tx.clone();
        let events = app.trace_tx.clone();
        let label = format!("{} MySQL Test", action.label());
        let path = crate::handlers::deploy_trace::operation_log_path();
        let trace_key = format!("{JOB_KEY}:{}", path.display());
        let worker_key = trace_key.clone();
        let worker_path = path.clone();
        if app.spawn_blocking_job(ctx, JOB_KEY, label, move || {
            crate::handlers::deploy_trace::run_logged(
                worker_key,
                worker_path,
                tx,
                events,
                |log, trace| {
                    action
                        .run(&runner, log, trace)
                        .map_err(|error| error.to_string())
                },
            )
        }) {
            app.track_operation(trace_key, "MySQL Test".into(), path);
        }
    }
    if let Some(trace) = &latest_trace {
        if open_log {
            if let Err(error) = crate::handlers::external_app::open_text_file(&trace.log_path) {
                let _ = app
                    .main_log_tx
                    .send(crate::handlers::log_types::LogLine::error(format!(
                        "No se pudo abrir el log: {error}"
                    )));
            }
        } else if copy_log {
            if let Ok(contents) = std::fs::read_to_string(&trace.log_path) {
                ctx.output_mut(|output| {
                    output.copied_text = crate::handlers::diagnostics::redact(&contents)
                });
            }
        }
    }
}
