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
//! UI layer facade.

pub mod about;
pub mod add_project;
pub mod containers_panel;
pub mod helps;
pub mod loader;
pub mod main_window;
pub mod projects_panel;
pub mod settings;
pub mod theme;
pub mod toolbox_container;
pub mod toolbox_project;
pub mod welcome;
pub mod wsl_settings;

mod credential_dialogs;
mod modal;
mod state;

use crate::app::WsddApp;

pub(crate) use modal::render_modal_backdrop;
#[allow(unused_imports)]
pub use state::{
    ActiveView, MainTab, PrereqCredentialsPromptState, UiState, WebminCredentialsPromptState,
};

/// Renders the active WSDD view and global modal dialogs.
pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    match &app.ui.active.clone() {
        ActiveView::Welcome => welcome::render(ctx, app),
        ActiveView::Loader => loader::render(ctx, app),
        ActiveView::Main => main_window::render(ctx, app),
        ActiveView::Settings => {
            main_window::render(ctx, app);
            settings::render(ctx, app);
        }
        ActiveView::About => {
            main_window::render(ctx, app);
            about::render(ctx, app);
        }
        ActiveView::Helps => helps::render(ctx, app),
        ActiveView::WslSettings => {
            main_window::render(ctx, app);
            wsl_settings::render(ctx, app);
        }
        ActiveView::AddProject => {
            main_window::render(ctx, app);
            add_project::render(ctx, app);
        }
        ActiveView::ToolboxProject => {
            main_window::render(ctx, app);
            toolbox_project::render(ctx, app);
        }
        ActiveView::ToolboxContainer => {
            main_window::render(ctx, app);
            toolbox_container::render(ctx, app);
        }
    }

    credential_dialogs::render(ctx, app);
}
