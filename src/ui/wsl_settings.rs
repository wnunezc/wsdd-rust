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
use crate::i18n::{tr, Language};
use crate::ui::ActiveView;

pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    let copy = wsl_copy(app.settings.language);

    // Cargar draft al entrar por primera vez en esta visita
    if app.ui.wsl_draft.is_none() {
        app.ui.wsl_draft = Some(match wsl::read() {
            Ok(cfg) => cfg,
            Err(e) => {
                tracing::warn!("Unable to read .wslconfig: {e}. Using defaults.");
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
            ui.heading(tr("wsl_title"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(format!("  {}  ", tr("btn_cancel"))).clicked() {
                    cancel = true;
                }
                ui.add_space(4.0);
                if ui
                    .add(
                        egui::Button::new(format!("  {}  ", tr("btn_save")))
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
            egui::RichText::new(format!(
                "{} {}",
                copy.path_prefix,
                wsl::config_path_display()
            ))
            .size(11.0)
            .color(ui.visuals().weak_text_color()),
        );
        ui.separator();
        ui.add_space(6.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // ── Recursos ──────────────────────────────────────────────────
            egui::CollapsingHeader::new(egui::RichText::new(copy.system_resources).strong())
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(copy.resource_note)
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
                            ui.label(copy.cpu_cores);
                            ui.horizontal(|ui| {
                                // Opciones: Sin limite + pares hasta cpu_max
                                let current_label = draft
                                    .processors
                                    .map(|n| n.to_string())
                                    .unwrap_or_else(|| copy.no_limit.to_string());

                                egui::ComboBox::from_id_salt("wsl_cpu")
                                    .selected_text(&current_label)
                                    .width(120.0)
                                    .show_ui(ui, |ui| {
                                        if ui
                                            .selectable_label(
                                                draft.processors.is_none(),
                                                copy.no_limit,
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
                                    egui::RichText::new(copy.detected_cpu(cpu_max))
                                        .size(11.0)
                                        .color(ui.visuals().weak_text_color()),
                                );
                            });
                            ui.end_row();

                            // RAM
                            ui.label(copy.max_memory);
                            ui.horizontal(|ui| {
                                let current_label = draft
                                    .memory_gb
                                    .map(|n| format!("{n} GB"))
                                    .unwrap_or_else(|| copy.no_limit.to_string());

                                egui::ComboBox::from_id_salt("wsl_ram")
                                    .selected_text(&current_label)
                                    .width(120.0)
                                    .show_ui(ui, |ui| {
                                        if ui
                                            .selectable_label(
                                                draft.memory_gb.is_none(),
                                                copy.no_limit,
                                            )
                                            .clicked()
                                        {
                                            draft.memory_gb = None;
                                        }
                                        for &gb in &[1u32, 2, 4, 6, 8, 12, 16, 24, 32, 48, 64] {
                                            let selected = draft.memory_gb == Some(gb);
                                            if ui
                                                .selectable_label(selected, format!("{gb} GB"))
                                                .clicked()
                                            {
                                                draft.memory_gb = Some(gb);
                                            }
                                        }
                                    });

                                ui.label(
                                    egui::RichText::new(copy.ram_recommendation)
                                        .size(11.0)
                                        .color(ui.visuals().weak_text_color()),
                                );
                            });
                            ui.end_row();

                            // Swap
                            ui.label(tr("wsl_swap"));
                            ui.horizontal(|ui| {
                                let current_label = if draft.swap_gb == 0 {
                                    copy.swap_disabled.to_string()
                                } else {
                                    format!("{} GB", draft.swap_gb)
                                };

                                egui::ComboBox::from_id_salt("wsl_swap")
                                    .selected_text(&current_label)
                                    .width(120.0)
                                    .show_ui(ui, |ui| {
                                        if ui
                                            .selectable_label(
                                                draft.swap_gb == 0,
                                                copy.swap_disabled,
                                            )
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
                                    egui::RichText::new(copy.swap_recommendation)
                                        .size(11.0)
                                        .color(ui.visuals().weak_text_color()),
                                );
                            });
                            ui.end_row();
                        });
                });

            ui.add_space(8.0);

            // ── Rendimiento ───────────────────────────────────────────────
            egui::CollapsingHeader::new(egui::RichText::new(copy.performance_memory).strong())
                .default_open(true)
                .show(ui, |ui| {
                    egui::Grid::new("wsl_perf")
                        .num_columns(2)
                        .spacing([12.0, 10.0])
                        .min_col_width(180.0)
                        .show(ui, |ui| {
                            // Memory reclaim
                            ui.label(copy.memory_reclaim);
                            egui::ComboBox::from_id_salt("wsl_reclaim")
                                .selected_text(memory_reclaim_label(
                                    &draft.memory_reclaim,
                                    app.settings.language,
                                ))
                                .width(220.0)
                                .show_ui(ui, |ui| {
                                    for mode in MemoryReclaim::all() {
                                        let selected = &draft.memory_reclaim == mode;
                                        if ui
                                            .selectable_label(
                                                selected,
                                                memory_reclaim_label(mode, app.settings.language),
                                            )
                                            .clicked()
                                        {
                                            draft.memory_reclaim = mode.clone();
                                        }
                                    }
                                });
                            ui.end_row();

                            // GUI apps
                            ui.label(copy.gui_apps);
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut draft.gui_applications, "");
                                ui.label(
                                    egui::RichText::new(copy.gui_apps_hint)
                                        .size(11.0)
                                        .color(ui.visuals().weak_text_color()),
                                );
                            });
                            ui.end_row();
                        });
                });

            ui.add_space(8.0);

            // ── Red ───────────────────────────────────────────────────────
            egui::CollapsingHeader::new(egui::RichText::new(copy.network).strong())
                .default_open(true)
                .show(ui, |ui| {
                    egui::Grid::new("wsl_network")
                        .num_columns(2)
                        .spacing([12.0, 10.0])
                        .min_col_width(180.0)
                        .show(ui, |ui| {
                            // localhost forwarding
                            ui.label(copy.localhost_forwarding);
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut draft.localhost_forwarding, "");
                                ui.label(
                                    egui::RichText::new(copy.localhost_hint)
                                        .size(11.0)
                                        .color(ui.visuals().weak_text_color()),
                                );
                            });
                            ui.end_row();

                            // Networking mode
                            ui.label(copy.network_mode);
                            egui::ComboBox::from_id_salt("wsl_netmode")
                                .selected_text(networking_mode_label(
                                    &draft.networking_mode,
                                    app.settings.language,
                                ))
                                .width(260.0)
                                .show_ui(ui, |ui| {
                                    for mode in NetworkingMode::all() {
                                        let selected = &draft.networking_mode == mode;
                                        if ui
                                            .selectable_label(
                                                selected,
                                                networking_mode_label(mode, app.settings.language),
                                            )
                                            .clicked()
                                        {
                                            draft.networking_mode = mode.clone();
                                        }
                                    }
                                });
                            ui.end_row();

                            // Opciones solo para Mirrored
                            if draft.networking_mode == NetworkingMode::Mirrored {
                                ui.label(copy.dns_tunneling);
                                ui.checkbox(&mut draft.dns_tunneling, "");
                                ui.end_row();

                                ui.label(copy.windows_firewall);
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
                        egui::RichText::new(copy.restart_note)
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

struct WslCopy {
    path_prefix: &'static str,
    system_resources: &'static str,
    resource_note: &'static str,
    cpu_cores: &'static str,
    no_limit: &'static str,
    max_memory: &'static str,
    ram_recommendation: &'static str,
    swap_disabled: &'static str,
    swap_recommendation: &'static str,
    performance_memory: &'static str,
    memory_reclaim: &'static str,
    gui_apps: &'static str,
    gui_apps_hint: &'static str,
    network: &'static str,
    localhost_forwarding: &'static str,
    localhost_hint: &'static str,
    network_mode: &'static str,
    dns_tunneling: &'static str,
    windows_firewall: &'static str,
    restart_note: &'static str,
    detected_cpu_fmt: &'static str,
}

impl WslCopy {
    fn detected_cpu(&self, cpu_max: u32) -> String {
        self.detected_cpu_fmt
            .replace("{count}", &cpu_max.to_string())
    }
}

fn wsl_copy(language: Language) -> WslCopy {
    match language {
        Language::Es => WslCopy {
            path_prefix: "→",
            system_resources: "Recursos del sistema",
            resource_note: "WSDD recomienda no superar el 60-70% de la RAM y CPU fisicas del sistema para mantener fluidez en el host.",
            cpu_cores: "Nucleos de CPU:",
            no_limit: "Sin limite",
            max_memory: "RAM maxima:",
            ram_recommendation: "(recomendado: 60% de la RAM fisica)",
            swap_disabled: "Deshabilitado",
            swap_recommendation: "(recomendado: 0 con RAM suficiente)",
            performance_memory: "Rendimiento y memoria",
            memory_reclaim: "Recuperacion de memoria:",
            gui_apps: "Aplicaciones GUI (WSLg):",
            gui_apps_hint: "Desactivar mejora el rendimiento si no se usa Linux GUI",
            network: "Red",
            localhost_forwarding: "Localhost forwarding:",
            localhost_hint: "Acceder a servicios WSL2 via 127.0.0.1 desde Windows",
            network_mode: "Modo de red:",
            dns_tunneling: "DNS Tunneling:",
            windows_firewall: "Firewall de Windows:",
            restart_note: "⚠  Los cambios requieren reiniciar WSL2 para aplicarse.\n    Ejecutar en PowerShell: wsl --shutdown",
            detected_cpu_fmt: "(detectados: {count} logicos)",
        },
        _ => WslCopy {
            path_prefix: "→",
            system_resources: "System resources",
            resource_note: "WSDD recommends staying below 60-70% of physical RAM and CPU to keep the Windows host responsive.",
            cpu_cores: "CPU cores:",
            no_limit: "No limit",
            max_memory: "Max RAM:",
            ram_recommendation: "(recommended: 60% of physical RAM)",
            swap_disabled: "Disabled",
            swap_recommendation: "(recommended: 0 when RAM is sufficient)",
            performance_memory: "Performance and memory",
            memory_reclaim: "Memory reclaim:",
            gui_apps: "GUI applications (WSLg):",
            gui_apps_hint: "Disabling this improves performance if you do not use Linux GUI apps",
            network: "Network",
            localhost_forwarding: "Localhost forwarding:",
            localhost_hint: "Access WSL2 services through 127.0.0.1 from Windows",
            network_mode: "Network mode:",
            dns_tunneling: "DNS tunneling:",
            windows_firewall: "Windows firewall:",
            restart_note: "⚠  Changes require restarting WSL2 before they apply.\n    Run in PowerShell: wsl --shutdown",
            detected_cpu_fmt: "(detected: {count} logical)",
        },
    }
}

fn networking_mode_label(mode: &NetworkingMode, language: Language) -> &'static str {
    match (language, mode) {
        (Language::Es, NetworkingMode::Nat) => "NAT (recomendado)",
        (Language::Es, NetworkingMode::Mirrored) => "Mirrored (experimental, Win11 23H2+)",
        (_, NetworkingMode::Nat) => "NAT (recommended)",
        (_, NetworkingMode::Mirrored) => "Mirrored (experimental, Win11 23H2+)",
    }
}

fn memory_reclaim_label(mode: &MemoryReclaim, language: Language) -> &'static str {
    match (language, mode) {
        (Language::Es, MemoryReclaim::Disabled) => "Deshabilitado",
        (Language::Es, MemoryReclaim::Gradual) => "Gradual (recomendado)",
        (Language::Es, MemoryReclaim::DropCache) => "Drop Cache (agresivo)",
        (_, MemoryReclaim::Disabled) => "Disabled",
        (_, MemoryReclaim::Gradual) => "Gradual (recommended)",
        (_, MemoryReclaim::DropCache) => "Drop cache (aggressive)",
    }
}
