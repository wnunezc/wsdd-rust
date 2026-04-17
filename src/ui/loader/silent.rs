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
//! Silent loader presentation for automatic prerequisite checks.

use egui::{Color32, RichText};

use crate::i18n::tr;

/// Renders the quiet verification screen used after setup has completed once.
pub(super) fn render(ctx: &egui::Context) {
    let time = ctx.input(|i| i.time);
    let dot_count = (time * 1.5) as usize % 4;
    let dots: String = ".".repeat(dot_count);
    let pad: String = " ".repeat(3usize.saturating_sub(dot_count));
    let app_name = tr("app_name");
    let checking = tr("loader_verifying_environment");

    egui::CentralPanel::default().show(ctx, |ui| {
        let avail_h = ui.available_height();
        let content_h = 280.0;
        let top_space = ((avail_h - content_h) / 2.0).max(0.0);

        ui.vertical_centered(|ui| {
            ui.add_space(top_space);

            ui.label(RichText::new("WSDD").size(104.0).strong().monospace());

            ui.add_space(4.0);

            ui.label(
                RichText::new(app_name)
                    .size(24.0)
                    .color(Color32::from_gray(140)),
            );

            ui.add_space(32.0);

            ui.add(egui::Spinner::new().size(40.0));

            ui.add_space(14.0);

            ui.label(
                RichText::new(format!("{checking}{dots}{pad}"))
                    .size(24.0)
                    .color(Color32::from_gray(160))
                    .monospace(),
            );
        });
    });
}
