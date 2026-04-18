use crate::app::WsddApp;
use crate::handlers::docker_deploy::{self, OptionalServiceKind};
use crate::handlers::log_types::LogLine;
use crate::handlers::setting::AppSettings;
use crate::i18n;
use crate::ui::ActiveView;

/// Saves the current settings draft and synchronizes generated resources.
pub(super) fn save(ctx: &egui::Context, app: &mut WsddApp) {
    if let Some(mut draft) = app.ui.settings_draft.take() {
        let previous = app.settings.clone();
        let selected_language = draft.language;
        if draft.docker_path.as_deref() == Some("") {
            draft.docker_path = None;
        }
        if draft.wsl_distro.as_deref() == Some("") {
            draft.wsl_distro = None;
        }
        draft.optional_services.mailpit.virtual_host = draft
            .optional_services
            .mailpit
            .virtual_host
            .trim()
            .to_string();
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
        if let Err(e) = draft.validate_optional_services() {
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
            app.ui.settings_error = Some(format!("Error actualizando init.yml administrado: {e}"));
            app.ui.settings_draft = Some(app.settings.clone());
            return;
        }

        if let Err(e) = docker_deploy::sync_saved_php_version_resources_sync(&app.settings) {
            app.ui.settings_error = Some(format!("Error actualizando recursos PHP/Webmin: {e}"));
            app.ui.settings_draft = Some(app.settings.clone());
            return;
        }

        if let Err(e) = docker_deploy::sync_optional_service_resources_sync(&app.settings) {
            app.ui.settings_error = Some(format!("Error actualizando servicios opcionales: {e}"));
            app.ui.settings_draft = Some(app.settings.clone());
            return;
        }

        for change in optional_service_changes(&previous, &app.settings) {
            spawn_optional_service_change(ctx, app, change);
        }
    }
    app.ui.settings_error = None;
    app.ui.active = ActiveView::Main;
}

/// Discards the current settings draft and returns to the main view.
pub(super) fn cancel(app: &mut WsddApp) {
    app.ui.settings_draft = None;
    app.ui.settings_error = None;
    app.ui.active = ActiveView::Main;
}

#[derive(Debug, Clone, Copy)]
enum OptionalServiceChange {
    Deploy(OptionalServiceKind),
    Stop(OptionalServiceKind),
}

fn optional_service_changes(
    previous: &AppSettings,
    next: &AppSettings,
) -> Vec<OptionalServiceChange> {
    let mut changes = Vec::new();

    if next.optional_services.redis.enabled {
        if !previous.optional_services.redis.enabled
            || previous.optional_services.redis != next.optional_services.redis
        {
            changes.push(OptionalServiceChange::Deploy(OptionalServiceKind::Redis));
        }
    } else if previous.optional_services.redis.enabled {
        changes.push(OptionalServiceChange::Stop(OptionalServiceKind::Redis));
    }

    if next.optional_services.mailpit.enabled {
        if !previous.optional_services.mailpit.enabled
            || previous.optional_services.mailpit != next.optional_services.mailpit
        {
            changes.push(OptionalServiceChange::Deploy(OptionalServiceKind::Mailpit));
        }
    } else if previous.optional_services.mailpit.enabled {
        changes.push(OptionalServiceChange::Stop(OptionalServiceKind::Mailpit));
    }

    if next.optional_services.memcached.enabled {
        if !previous.optional_services.memcached.enabled
            || previous.optional_services.memcached != next.optional_services.memcached
        {
            changes.push(OptionalServiceChange::Deploy(
                OptionalServiceKind::Memcached,
            ));
        }
    } else if previous.optional_services.memcached.enabled {
        changes.push(OptionalServiceChange::Stop(OptionalServiceKind::Memcached));
    }

    changes
}

fn spawn_optional_service_change(
    ctx: &egui::Context,
    app: &mut WsddApp,
    change: OptionalServiceChange,
) {
    let settings = app.settings.clone();
    let runner = app.runner.clone();
    let tx = app.main_log_tx.clone();
    let kind = match change {
        OptionalServiceChange::Deploy(kind) | OptionalServiceChange::Stop(kind) => kind,
    };
    let action = match change {
        OptionalServiceChange::Deploy(_) => "deploy",
        OptionalServiceChange::Stop(_) => "stop",
    };
    let label = format!("Optional service {action} {}", kind.display_name());
    let job_key = format!(
        "optional:{}:{action}",
        kind.display_name().to_ascii_lowercase()
    );

    let started = app.spawn_blocking_job(ctx, job_key, label, move || {
        let result = match change {
            OptionalServiceChange::Deploy(kind) => {
                docker_deploy::deploy_optional_service_sync(&runner, &settings, kind, &tx)
            }
            OptionalServiceChange::Stop(kind) => {
                docker_deploy::stop_optional_service_sync(&runner, kind, &tx)
            }
        };
        result.map_err(|e| e.to_string())
    });

    if !started {
        let _ = app.main_log_tx.send(LogLine::warn(format!(
            "Ya hay una tarea activa para {}",
            kind.display_name()
        )));
    }
}
