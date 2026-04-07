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
// Equivalente a Forms/DisplaySelector.cs — selector de monitor
use crate::app::WsddApp;
use crate::ui::ActiveView;

pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Seleccionar monitor");
        // TODO: migrar DisplaySelector.cs
        if ui.button("Cerrar").clicked() {
            app.ui.active = ActiveView::Main;
        }
    });
}
