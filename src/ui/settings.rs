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
//! WSDD settings modal renderer.

use crate::app::WsddApp;

mod actions;
mod sections;

/// Renders the WSDD settings modal and applies save/cancel actions.
pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    if app.ui.settings_draft.is_none() {
        app.ui.settings_draft = Some(app.settings.clone());
        app.ui.settings_error = None;
        app.ui.settings_recovery_task_status = Some(recovery_task_status(app));
    }

    let copy = sections::SettingsCopy::new();
    let current_error = app.ui.settings_error.clone();
    let mut save = false;
    let mut cancel = false;
    let mut open = true;

    crate::ui::render_modal_backdrop(ctx, "settings_backdrop");

    egui::Window::new(&copy.title)
        .collapsible(false)
        .resizable(true)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(860.0)
        .default_height(680.0)
        .min_width(760.0)
        .min_height(540.0)
        .open(&mut open)
        .show(ctx, |ui| {
            let Some(draft) = app.ui.settings_draft.as_mut() else {
                return;
            };

            sections::render_header(ui, &copy, &mut save, &mut cancel);
            ui.separator();
            ui.add_space(6.0);
            sections::render_error(ui, current_error.as_deref());
            sections::render_body(
                ui,
                draft,
                &copy,
                app.ui.settings_recovery_task_status.as_deref(),
            );
        });

    if !open && !save {
        cancel = true;
    }

    if save {
        actions::save(ctx, app);
    } else if cancel {
        actions::cancel(app);
    }
}

fn recovery_task_status(app: &WsddApp) -> String {
    let projects = crate::handlers::project::list_all().unwrap_or_default();
    let required = crate::handlers::scheduled_task::recovery_task_decision(
        app.settings.managed_native_linux_storage_enabled,
        &projects,
        app.settings.wsl_distro.as_deref(),
    ) != crate::handlers::scheduled_task::RecoveryTaskDecision::Remove;
    match crate::handlers::scheduled_task::status(&app.runner, required) {
        Ok(crate::handlers::scheduled_task::RecoveryTaskStatus::Installed) => {
            crate::i18n::tr("settings_recovery_task_installed")
        }
        Ok(crate::handlers::scheduled_task::RecoveryTaskStatus::Missing) => {
            crate::i18n::tr("settings_recovery_task_missing")
        }
        Ok(crate::handlers::scheduled_task::RecoveryTaskStatus::NotRequired) => {
            crate::i18n::tr("settings_recovery_task_not_required")
        }
        Ok(crate::handlers::scheduled_task::RecoveryTaskStatus::NeedsUpdate) => {
            crate::i18n::tr("settings_recovery_task_needs_update")
        }
        Err(error) => format!("{}: {error}", crate::i18n::tr("status_error")),
    }
}
