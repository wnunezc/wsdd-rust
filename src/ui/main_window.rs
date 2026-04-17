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
//! Main WSDD window facade.
//!
//! The public `render` entry point is kept stable while menu/toolbar chrome,
//! background actions, modal dialogs, and panels live in focused submodules.

mod actions;
mod chrome;
mod dialogs;
mod panels;

use std::time::Duration;

use crate::app::WsddApp;

const POLL_INTERVAL: Duration = Duration::from_secs(3);
const SUPPORT_ISSUES_URL: &str = "https://github.com/wnunezc/wsdd-rust/issues/new";

/// Renders the main application window.
pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    actions::poll_containers(ctx, app);

    chrome::render_menu_bar(ctx, app);
    chrome::render_toolbar(ctx, app);
    panels::render_status_bar(ctx, app);
    panels::render_log_panel(ctx, app);
    dialogs::render_confirm_dialog(ctx, app);
    panels::render_center(ctx, app);

    ctx.request_repaint_after(POLL_INTERVAL);
}
