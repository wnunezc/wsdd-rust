use crate::app::WsddApp;
use crate::handlers::docker_deploy;
use crate::i18n;
use crate::ui::ActiveView;

/// Saves the current settings draft and synchronizes generated resources.
pub(super) fn save(app: &mut WsddApp) {
    if let Some(mut draft) = app.ui.settings_draft.take() {
        let selected_language = draft.language;
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
            app.ui.settings_error = Some(format!("Error actualizando init.yml administrado: {e}"));
            app.ui.settings_draft = Some(app.settings.clone());
            return;
        }

        if let Err(e) = docker_deploy::sync_saved_php_version_resources_sync(&app.settings) {
            app.ui.settings_error = Some(format!("Error actualizando recursos PHP/Webmin: {e}"));
            app.ui.settings_draft = Some(app.settings.clone());
            return;
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
