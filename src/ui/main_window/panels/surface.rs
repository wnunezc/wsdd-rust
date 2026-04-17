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
//! Shared panel surface styling for the main window.

/// Renders a grouped surface with the established main-window spacing.
pub(super) fn show_surface_panel<Contents>(ui: &mut egui::Ui, add_contents: Contents)
where
    Contents: FnOnce(&mut egui::Ui),
{
    egui::Frame::none()
        .inner_margin(egui::Margin::symmetric(10.0, 6.0))
        .show(ui, |ui| {
            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::symmetric(12.0, 10.0))
                .show(ui, |ui| add_contents(ui));
        });
}
