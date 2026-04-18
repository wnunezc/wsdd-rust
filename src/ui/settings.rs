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
            sections::render_body(ui, draft, &copy);
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
