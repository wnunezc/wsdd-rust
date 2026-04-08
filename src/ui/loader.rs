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
//! Pantalla de requisitos. Equivalente a `Forms/Loader.cs`.
//!
//! # Comportamiento
//!
//! **Primer arranque** (`loader_silent = false`):
//!   - Muestra terminal con output en tiempo real
//!   - Al completar con éxito: botón "Abrir WSDD" → Main
//!   - Error bloqueante: botón "Salir"
//!   - Necesita reboot: botón "Reiniciar"
//!
//! **Arranques posteriores** (`loader_silent = true`):
//!   - Renderiza un panel en blanco mientras verifica
//!   - Si todo OK: transición automática a Main (sin intervención del usuario)
//!   - Si hay error: cambia a modo visible con el log de error

use std::sync::mpsc;

use egui::{Color32, Frame, Layout, Margin, RichText, ScrollArea};

use crate::app::WsddApp;
use crate::handlers::log_types::{LogLevel, LogLine};
use crate::handlers::requirements::{run_requirements, LoaderOutcome};
use crate::i18n::tr;
use crate::ui::ActiveView;

// ─── Punto de entrada ─────────────────────────────────────────────────────────

pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    // Iniciar el proceso de requirements en el primer frame del Loader
    if !app.requirements_started {
        start_requirements(ctx, app);
        app.requirements_started = true;
    }

    // Drenar canales
    drain_log(app);
    handle_outcome(app);

    // Modo silencioso: sin error → transición automática a Main al terminar
    if app.loader_silent && !app.loader_error && !app.loader_needs_reboot {
        if app.loader_done {
            app.ui.active = ActiveView::Main;
            return;
        }
        render_silent_loader(ctx);
        ctx.request_repaint();
        return;
    }

    // Modo visible: mostrar terminal completo
    render_terminal(ctx, app);
}

// ─── Inicialización ───────────────────────────────────────────────────────────

fn start_requirements(ctx: &egui::Context, app: &mut WsddApp) {
    let (log_tx, log_rx) = mpsc::channel::<LogLine>();
    let (outcome_tx, outcome_rx) = mpsc::channel::<LoaderOutcome>();

    app.requirement_rx = Some(log_rx);
    app.loader_outcome_rx = Some(outcome_rx);

    let ctx_clone = ctx.clone();
    let first_run = !app.loader_silent;

    std::thread::spawn(move || {
        run_requirements(log_tx, outcome_tx, first_run);
        // Notificar a egui que hay nuevo estado para renderizar
        ctx_clone.request_repaint();
    });
}

// ─── Drenado de canales ───────────────────────────────────────────────────────

fn drain_log(app: &mut WsddApp) {
    if let Some(rx) = &app.requirement_rx {
        while let Ok(line) = rx.try_recv() {
            if let Some(key) = line.key.clone() {
                // Actualización in-place: reemplazar la línea con la misma key
                // si ya existe, o añadirla al final si es nueva.
                // Usado para el progreso por capa de Docker (una línea por hash).
                if let Some(existing) = app
                    .ui
                    .requirement_log
                    .iter_mut()
                    .find(|l| l.key.as_deref() == Some(key.as_str()))
                {
                    *existing = line;
                } else {
                    app.ui.requirement_log.push(line);
                }
            } else {
                app.ui.requirement_log.push(line);
            }
        }
    }
}

fn handle_outcome(app: &mut WsddApp) {
    if app.loader_done {
        return; // Ya procesado
    }
    if let Some(rx) = &app.loader_outcome_rx {
        if let Ok(outcome) = rx.try_recv() {
            app.loader_done = true;
            match outcome {
                LoaderOutcome::BlockingError => app.loader_error = true,
                LoaderOutcome::NeedsReboot => app.loader_needs_reboot = true,
                LoaderOutcome::AllDone | LoaderOutcome::DoneWithContinue => {
                    // loader_error y loader_needs_reboot quedan en false → éxito
                }
            }
        }
    }
}

// ─── Loader silencioso ────────────────────────────────────────────────────────

fn render_silent_loader(ctx: &egui::Context) {
    let time = ctx.input(|i| i.time);
    let dot_count = (time * 1.5) as usize % 4;
    let dots: String = ".".repeat(dot_count);
    // Padding fijo para que el texto no se desplace al cambiar la longitud
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

// ─── Renderizado completo ─────────────────────────────────────────────────────

fn render_terminal(ctx: &egui::Context, app: &mut WsddApp) {
    let title = tr("loader_system_requirements");
    let processing = tr("loader_processing");
    let copy_label = tr("loader_copy_log");

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical(|ui| {
            // Cabecera
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.heading(RichText::new(&title).size(20.0).strong());
            });
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            // Terminal con líneas coloreadas
            let bottom_bar_height = if app.loader_done { 56.0 } else { 8.0 };
            let scroll_height = ui.available_height() - bottom_bar_height;

            let dark = ui.visuals().dark_mode;
            let frame_fill = if dark {
                Color32::from_rgb(18, 18, 18)
            } else {
                Color32::from_rgb(240, 240, 240)
            };
            Frame::none()
                .fill(frame_fill)
                .inner_margin(Margin::same(8.0))
                .show(ui, |ui| {
                    ScrollArea::vertical()
                        .max_height(scroll_height)
                        .auto_shrink([false, false])
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for line in &app.ui.requirement_log {
                                ui.label(
                                    RichText::new(&line.text)
                                        .color(level_color(&line.level, dark))
                                        .monospace(),
                                );
                            }

                            if !app.loader_done {
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.label(
                                        RichText::new(format!(" {processing}"))
                                            .color(Color32::GRAY)
                                            .monospace(),
                                    );
                                });
                            }
                        });
                });

            // Barra de botones (solo al terminar)
            if app.loader_done {
                ui.separator();
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    // Botón Copiar — izquierda
                    if ui.small_button(&copy_label).clicked() {
                        let text: String = app
                            .ui
                            .requirement_log
                            .iter()
                            .map(|l| l.text.as_str())
                            .collect::<Vec<_>>()
                            .join("\n");
                        ui.output_mut(|o| o.copied_text = text);
                    }
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        render_buttons(ui, app);
                    });
                });
                ui.add_space(8.0);
            }
        });
    });

    // Seguir repintando mientras el proceso está en curso
    if !app.loader_done {
        ctx.request_repaint();
    }
}

fn render_buttons(ui: &mut egui::Ui, app: &mut WsddApp) {
    if app.loader_error {
        // Error bloqueante — Docker no instalado
        if ui.button(format!("  {}  ", tr("menu_exit"))).clicked() {
            std::process::exit(0);
        }
    } else if app.loader_needs_reboot {
        // Reinicio necesario tras instalar Docker
        if ui
            .button(format!("  {}  ", tr("loader_restart_system")))
            .clicked()
        {
            relaunch_app();
        }
    } else {
        // Éxito — primer arranque muestra el botón (silent ya transitó antes)
        if ui
            .button(format!("  {}  ", tr("loader_open_wsdd")))
            .clicked()
        {
            app.ui.active = ActiveView::Main;
        }
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn level_color(level: &LogLevel, dark: bool) -> Color32 {
    if dark {
        match level {
            LogLevel::Info => Color32::from_rgb(200, 200, 200),
            LogLevel::Success => Color32::from_rgb(100, 220, 100),
            LogLevel::Warn => Color32::from_rgb(255, 200, 0),
            LogLevel::Error => Color32::from_rgb(255, 80, 80),
        }
    } else {
        match level {
            LogLevel::Info => Color32::from_rgb(50, 50, 50),
            LogLevel::Success => Color32::from_rgb(0, 130, 0),
            LogLevel::Warn => Color32::from_rgb(160, 80, 0),
            LogLevel::Error => Color32::from_rgb(180, 0, 0),
        }
    }
}

/// Relanza la aplicación con privilegios de administrador y cierra la instancia actual.
///
/// Usado cuando Docker requiere reinicio del sistema tras ser instalado.
fn relaunch_app() {
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::process::Command::new(exe).spawn();
    }
    std::process::exit(0);
}
