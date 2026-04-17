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
//! Main-window panel facade.

mod center;
mod log_panel;
mod status_bar;
mod surface;

use crate::app::WsddApp;

/// Renders the central content facade.
pub(super) fn render_center(ctx: &egui::Context, app: &mut WsddApp) {
    center::render_center(ctx, app);
}

/// Renders the log panel facade.
pub(super) fn render_log_panel(ctx: &egui::Context, app: &mut WsddApp) {
    log_panel::render_log_panel(ctx, app);
}

/// Renders the status bar facade.
pub(super) fn render_status_bar(ctx: &egui::Context, app: &mut WsddApp) {
    status_bar::render_status_bar(ctx, app);
}
