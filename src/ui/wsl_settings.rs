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
//! Pantalla de configuración WSL2 para WSDD.
//! Equivalente a `Forms/WSLGeneralSetting.cs` — extendido con parámetros de rendimiento.
//!
//! Lee y escribe `%USERPROFILE%\.wslconfig`.
//! Edita una copia (draft) y aplica al guardar.

use crate::app::WsddApp;
use crate::handlers::wsl::{self, MemoryReclaim, NetworkingMode, WslConfig};
use crate::ui::ActiveView;

pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    // Cargar draft al entrar por primera vez en esta visita
    if app.ui.wsl_draft.is_none() {
        app.ui.wsl_draft = Some(match wsl::read() {
            Ok(cfg) => cfg,
            Err(e) => {
                tracing::warn!("No se pudo leer .wslconfig: {e}. Usando defaults.");
                WslConfig::default()
            }
        });
    }

    let mut save = false;
    let mut cancel = false;

    // Calcular límites del sistema una sola vez
    let cpu_max = std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(16);

    egui::CentralPanel::default().show(ctx, |ui| {
        let draft = app.ui.wsl_draft.as_mut().unwrap();

        // ── Cabecera ──────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.heading("WSL2 Settings");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("  Cancelar  ").clicked() {
                    cancel = true;
                }
                ui.add_space(4.0);
                if ui
                    .add(
                        egui::Button::new("  Guardar  ")
                            .fill(egui::Color32::from_rgb(34, 139, 34)),
                    )
                    .clicked()
                {
                    save = true;
                }
            });
        });

        // Ruta del archivo
        ui.label(
            egui::RichText::new(format!("→ {}", wsl::config_path_display()))
                .size(11.0)
                .color(ui.visuals().weak_text_color()),
        );
        ui.separator();
        ui.add_space(6.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // ── Recursos ──────────────────────────────────────────────────
            egui::CollapsingHeader::new(egui::RichText::new("Recursos del sistema").strong())
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(
                            "WSDD recomienda no superar el 60-70% de la RAM y CPU fisicas \
                             del sistema para mantener fluidez en el host.",
                        )
                        .size(11.0)
                        .color(ui.visuals().weak_text_color()),
                    );
                    ui.add_space(6.0);

                    egui::Grid::new("wsl_resources")
                        .num_columns(2)
                        .spacing([12.0, 10.0])
                        .min_col_width(180.0)
                        .show(ui, |ui| {
                            // CPU
                            ui.label("Nucleos de CPU:");
                            ui.horizontal(|ui| {
                                // Opciones: Sin limite + pares hasta cpu_max
                                let current_label = draft
                                    .processors
                                    .map(|n| n.to_string())
                                    .unwrap_or_else(|| "Sin limite".to_string());

                                egui::ComboBox::from_id_salt("wsl_cpu")
                                    .selected_text(&current_label)
                                    .width(120.0)
                                    .show_ui(ui, |ui| {
                                        if ui
                                            .selectable_label(
                                                draft.processors.is_none(),
                                                "Sin limite",
                                            )
                                            .clicked()
                                        {
                                            draft.processors = None;
                                        }
                                        let mut n = 1u32;
                                        while n <= cpu_max {
                                            let selected = draft.processors == Some(n);
                                            if ui
                                                .selectable_label(selected, n.to_string())
                                                .clicked()
                                            {
                                                draft.processors = Some(n);
                                            }
                                            // Saltos: 1,2,4,6,8,12,16,24,32...
                                            n = if n < 4 { n + 1 } else { n + (n / 2).max(2) };
                                        }
                                    });

                                ui.label(
                                    egui::RichText::new(format!("(detectados: {cpu_max} logicos)"))
                                        .size(11.0)
                                        .color(ui.visuals().weak_text_color()),
                                );
                            });
                            ui.end_row();

                            // RAM
                            ui.label("RAM maxima:");
                            ui.horizontal(|ui| {
                                let current_label = draft
                                    .memory_gb
                                    .map(|n| format!("{n} GB"))
                                    .unwrap_or_else(|| "Sin limite".to_string());

                                egui::ComboBox::from_id_salt("wsl_ram")
                                    .selected_text(&current_label)
                                    .width(120.0)
                                    .show_ui(ui, |ui| {
                                        if ui
                                            .selectable_label(
                                                draft.memory_gb.is_none(),
                                                "Sin limite",
                                            )
                                            .clicked()
                                        {
                                            draft.memory_gb = None;
                                        }
                                        for &gb in &[1u32, 2, 4, 6, 8, 12, 16, 24, 32, 48, 64] {
                                            let selected = draft.memory_gb == Some(gb);
                                            if ui
                                                .selectable_label(
                                                    selected,
                                                    format!("{gb} GB"),
                                                )
                                                .clicked()
                                            {
                                                draft.memory_gb = Some(gb);
                                            }
                                        }
                                    });

                                ui.label(
                                    egui::RichText::new("(recomendado: 60% de la RAM fisica)")
                                        .size(11.0)
                                        .color(ui.visuals().weak_text_color()),
                                );
                            });
                            ui.end_row();

                            // Swap
                            ui.label("Swap:");
                            ui.horizontal(|ui| {
                                let current_label = if draft.swap_gb == 0 {
                                    "Deshabilitado".to_string()
                                } else {
                                    format!("{} GB", draft.swap_gb)
                                };

                                egui::ComboBox::from_id_salt("wsl_swap")
                                    .selected_text(&current_label)
                                    .width(120.0)
                                    .show_ui(ui, |ui| {
                                        if ui
                                            .selectable_label(draft.swap_gb == 0, "Deshabilitado")
                                            .clicked()
                                        {
                                            draft.swap_gb = 0;
                                        }
                                        for &gb in &[1u32, 2, 4, 8, 16] {
                                            let selected = draft.swap_gb == gb;
                                            if ui
                                                .selectable_label(selected, format!("{gb} GB"))
                                                .clicked()
                                            {
                                                draft.swap_gb = gb;
                                            }
                                        }
                                    });

                                ui.label(
                                    egui::RichText::new("(recomendado: 0 con RAM suficiente)")
                                        .size(11.0)
                                        .color(ui.visuals().weak_text_color()),
                                );
                            });
                            ui.end_row();
                        });
                });

            ui.add_space(8.0);

            // ── Rendimiento ───────────────────────────────────────────────
            egui::CollapsingHeader::new(egui::RichText::new("Rendimiento y memoria").strong())
                .default_open(true)
                .show(ui, |ui| {
                    egui::Grid::new("wsl_perf")
                        .num_columns(2)
                        .spacing([12.0, 10.0])
                        .min_col_width(180.0)
                        .show(ui, |ui| {
                            // Memory reclaim
                            ui.label("Recuperacion de memoria:");
                            egui::ComboBox::from_id_salt("wsl_reclaim")
                                .selected_text(draft.memory_reclaim.display_name())
                                .width(220.0)
                                .show_ui(ui, |ui| {
                                    for mode in MemoryReclaim::all() {
                                        let selected = &draft.memory_reclaim == mode;
                                        if ui
                                            .selectable_label(selected, mode.display_name())
                                            .clicked()
                                        {
                                            draft.memory_reclaim = mode.clone();
                                        }
                                    }
                                });
                            ui.end_row();

                            // GUI apps
                            ui.label("Aplicaciones GUI (WSLg):");
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut draft.gui_applications, "");
                                ui.label(
                                    egui::RichText::new(
                                        "Desactivar mejora el rendimiento si no se usa Linux GUI",
                                    )
                                    .size(11.0)
                                    .color(ui.visuals().weak_text_color()),
                                );
                            });
                            ui.end_row();
                        });
                });

            ui.add_space(8.0);

            // ── Red ───────────────────────────────────────────────────────
            egui::CollapsingHeader::new(egui::RichText::new("Red").strong())
                .default_open(true)
                .show(ui, |ui| {
                    egui::Grid::new("wsl_network")
                        .num_columns(2)
                        .spacing([12.0, 10.0])
                        .min_col_width(180.0)
                        .show(ui, |ui| {
                            // localhost forwarding
                            ui.label("Localhost forwarding:");
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut draft.localhost_forwarding, "");
                                ui.label(
                                    egui::RichText::new(
                                        "Acceder a servicios WSL2 via 127.0.0.1 desde Windows",
                                    )
                                    .size(11.0)
                                    .color(ui.visuals().weak_text_color()),
                                );
                            });
                            ui.end_row();

                            // Networking mode
                            ui.label("Modo de red:");
                            egui::ComboBox::from_id_salt("wsl_netmode")
                                .selected_text(draft.networking_mode.display_name())
                                .width(260.0)
                                .show_ui(ui, |ui| {
                                    for mode in NetworkingMode::all() {
                                        let selected = &draft.networking_mode == mode;
                                        if ui
                                            .selectable_label(selected, mode.display_name())
                                            .clicked()
                                        {
                                            draft.networking_mode = mode.clone();
                                        }
                                    }
                                });
                            ui.end_row();

                            // Opciones solo para Mirrored
                            if draft.networking_mode == NetworkingMode::Mirrored {
                                ui.label("DNS Tunneling:");
                                ui.checkbox(&mut draft.dns_tunneling, "");
                                ui.end_row();

                                ui.label("Firewall de Windows:");
                                ui.checkbox(&mut draft.firewall, "");
                                ui.end_row();
                            }
                        });
                });

            ui.add_space(8.0);

            // ── Nota de reinicio ──────────────────────────────────────────
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(80, 60, 10))
                .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                .rounding(egui::Rounding::same(4.0))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(
                            "⚠  Los cambios requieren reiniciar WSL2 para aplicarse.\
                             \n    Ejecutar en PowerShell: wsl --shutdown",
                        )
                        .size(12.0)
                        .color(egui::Color32::from_rgb(255, 210, 80)),
                    );
                });

            ui.add_space(16.0);
        });
    });

    // ── Aplicar accion fuera del closure ─────────────────────────────────────
    if save {
        if let Some(draft) = app.ui.wsl_draft.take() {
            if let Err(e) = wsl::write(&draft) {
                tracing::error!("Error escribiendo .wslconfig: {e}");
            }
        }
        app.ui.active = ActiveView::Main;
    } else if cancel {
        app.ui.wsl_draft = None;
        app.ui.active = ActiveView::Main;
    }
}
