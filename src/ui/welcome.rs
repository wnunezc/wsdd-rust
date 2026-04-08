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
// Welcome Wizard — primer arranque. Equivalente a Forms/Wellcome.cs
//
// Flujo:
//   1. Muestra el README embebido en un scroll area
//   2. Checkbox "He leído las instrucciones" habilita el botón Comenzar
//   3. Comenzar → ActiveView::Loader (Fase 3); por ahora pasa a Main
//   4. Salir → termina el proceso

use crate::app::WsddApp;
use crate::i18n::{tr, Language};
use crate::ui::ActiveView;
use egui_commonmark::CommonMarkViewer;

static README_EN: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"));
static README_ES: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.es.md"));
static README_FR: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.fr.md"));
static README_HI: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.hi.md"));
static README_ZH: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.zh.md"));

fn readme_for(language: Language) -> &'static str {
    match language {
        Language::En => README_EN,
        Language::Es => README_ES,
        Language::Fr => README_FR,
        Language::Hi => README_HI,
        Language::Zh => README_ZH,
    }
}

pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    let readme = readme_for(app.settings.language);
    let app_name = tr("app_name");
    let subtitle = tr("welcome_subtitle");
    let checkbox = tr("welcome_checkbox_accept");
    let exit_label = format!("  {}  ", tr("menu_exit"));
    let continue_label = format!("  {}  ", tr("btn_continue"));

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical(|ui| {
            // ── Cabecera ──────────────────────────────────────────────────────
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.heading(egui::RichText::new(app_name).size(22.0).strong());
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(egui::RichText::new(subtitle).color(ui.visuals().weak_text_color()));
            });
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            // ── README scrollable ─────────────────────────────────────────────
            // Reservamos espacio para el separador inferior + checkbox + botones
            let bottom_bar_height = 72.0;
            let scroll_height = ui.available_height() - bottom_bar_height;

            egui::ScrollArea::vertical()
                .max_height(scroll_height)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(4.0);
                    CommonMarkViewer::new().show(ui, &mut app.ui.md_cache, readme);
                    ui.add_space(8.0);
                });

            // ── Barra inferior ────────────────────────────────────────────────
            ui.separator();
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.add_space(8.0);

                // Checkbox — habilita el botón Comenzar
                ui.checkbox(&mut app.ui.readme_checked, checkbox);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);

                    // Botón Salir — siempre habilitado
                    if ui.button(&exit_label).clicked() {
                        std::process::exit(0);
                    }

                    ui.add_space(8.0);

                    // Botón Comenzar — solo si checkbox marcado
                    let comenzar =
                        ui.add_enabled(app.ui.readme_checked, egui::Button::new(&continue_label));
                    if comenzar.clicked() {
                        app.settings.setup_completed = true;
                        app.ui.active = ActiveView::Loader;
                        // El proceso de requirements arranca en loader::render() en el primer frame
                    }
                });
            });

            ui.add_space(8.0);
        });
    });
}
