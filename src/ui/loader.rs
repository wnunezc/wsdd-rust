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
//! Requirements loader view. Equivalent to `Forms/Loader.cs`.
//!
//! First-run mode shows a terminal with live output and final action buttons.
//! Silent mode verifies prerequisites in the background and transitions to the
//! main view automatically when the environment is ready.

mod process;
mod silent;
mod terminal;

use crate::{app::WsddApp, ui::ActiveView};

/// Renders and advances the requirements loader workflow.
pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    if !app.requirements_started {
        if app.settings.prereq_credentials.is_complete() {
            process::start_requirements(ctx, app);
            app.requirements_started = true;
        } else if app.ui.prereq_prompt.is_none() {
            app.ui.open_prereq_prompt(&app.settings.prereq_credentials);
        }
    }

    process::drain_log(app);
    process::handle_outcome(app);

    if app.loader_silent && !app.loader_error {
        if app.loader_done {
            app.ui.active = ActiveView::Main;
            return;
        }
        silent::render(ctx);
        ctx.request_repaint();
        return;
    }

    terminal::render(ctx, app);
}
