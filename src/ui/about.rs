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

//! Pantalla About — versión, autor, licencia y créditos de WSDD.
//! Equivalente a `Forms/About.cs`.

use crate::app::WsddApp;
use crate::ui::ActiveView;

pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.add_space(24.0);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("◈ WebStack Deployer for Docker")
                    .size(22.0)
                    .strong(),
            );
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION")))
                    .size(15.0)
                    .color(ui.visuals().hyperlink_color),
            );
            ui.label(
                egui::RichText::new("Rust Edition — egui 0.29")
                    .size(11.0)
                    .color(ui.visuals().weak_text_color()),
            );
        });

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(12.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // ── Descripcion ───────────────────────────────────────────────────
            egui::Frame::none()
                .fill(ui.visuals().faint_bg_color)
                .inner_margin(egui::Margin::symmetric(16.0, 10.0))
                .rounding(egui::Rounding::same(6.0))
                .show(ui, |ui| {
                    ui.set_max_width(560.0);
                    ui.label(
                        "Herramienta de gestión de stacks PHP + Docker para entornos de \
                         desarrollo local en Windows. Automatiza deploy, certificados SSL, \
                         configuración de hosts y gestión de contenedores.",
                    );
                });

            ui.add_space(14.0);

            // ── Info general ──────────────────────────────────────────────────
            egui::Grid::new("about_info")
                .num_columns(2)
                .spacing([20.0, 6.0])
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Autor").strong());
                    ui.label("Walter Nunez / Icaros Net S.A");
                    ui.end_row();

                    ui.label(egui::RichText::new("Copyright").strong());
                    ui.label("(c) 2026 Walter Nunez / Icaros Net S.A");
                    ui.end_row();

                    ui.label(egui::RichText::new("Licencia").strong());
                    ui.label("Propietaria — solo uso de desarrollo");
                    ui.end_row();

                    ui.label(egui::RichText::new("Plataforma").strong());
                    ui.label("Windows 10 / 11 (requiere privilegios de administrador)");
                    ui.end_row();

                    ui.label(egui::RichText::new("Migrado desde").strong());
                    ui.label("C# WinForms / .NET 8.0");
                    ui.end_row();

                    ui.label(egui::RichText::new("Framework GUI").strong());
                    ui.label("egui 0.29 / eframe (Rust stable)");
                    ui.end_row();

                    ui.label(egui::RichText::new("Fuentes").strong());
                    ui.label("JetBrains Mono v2.304 (OFL)  +  Noto Sans Symbols 2 v2.008 (OFL)");
                    ui.end_row();
                });

            ui.add_space(14.0);
            ui.separator();
            ui.add_space(10.0);

            // ── Dependencias ──────────────────────────────────────────────────
            ui.label(egui::RichText::new("Dependencias de codigo abierto").strong());
            ui.add_space(6.0);

            let deps: &[(&str, &str, &str)] = &[
                ("egui / eframe 0.29", "GUI immediate-mode", "MIT / Apache 2.0"),
                ("tokio 1", "Runtime async", "MIT"),
                ("serde / serde_json", "Serialización JSON", "MIT / Apache 2.0"),
                ("serde_yaml 0.9", "Serialización YAML", "MIT / Apache 2.0"),
                ("quick-xml 0.36", "XML parsing", "MIT"),
                ("anyhow / thiserror", "Manejo de errores", "MIT / Apache 2.0"),
                ("rfd 0.15", "File dialogs nativos", "MIT"),
                ("egui_commonmark 0.18", "Markdown rendering", "MIT"),
                ("egui_extras 0.29", "Extras de UI", "MIT / Apache 2.0"),
                ("zip 2", "Compresión ZIP (recursos)", "MIT"),
                ("walkdir 2", "Recorrido de directorios", "MIT / Unlicense"),
                ("image 0.25", "Decodificación de iconos", "MIT / Apache 2.0"),
                ("tracing / tracing-subscriber", "Observabilidad", "MIT"),
                ("windows 0.58", "Windows API (UAC, Registry)", "MIT / Apache 2.0"),
            ];

            for (name, purpose, license) in deps {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(format!("• {name}")).monospace().size(12.0));
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(format!("— {purpose}"))
                            .color(ui.visuals().weak_text_color())
                            .size(12.0),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(*license)
                                .color(ui.visuals().hyperlink_color)
                                .size(11.0),
                        );
                    });
                });
            }

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);

            // ── Herramientas externas ─────────────────────────────────────────
            ui.label(egui::RichText::new("Herramientas externas requeridas").strong());
            ui.add_space(6.0);

            let tools: &[(&str, &str)] = &[
                ("Docker Desktop", "Motor de contenedores — docker.com"),
                ("WSL 2", "Capa de compatibilidad Linux — Microsoft"),
                ("Chocolatey", "Gestor de paquetes Windows — chocolatey.org"),
                ("mkcert", "Certificados SSL locales — github.com/FiloSottile/mkcert"),
                ("PowerShell 7+", "Shell de automatización — microsoft.com"),
            ];

            for (tool, desc) in tools {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(format!("• {tool}")).strong().size(12.0));
                    ui.label(
                        egui::RichText::new(format!("— {desc}"))
                            .color(ui.visuals().weak_text_color())
                            .size(12.0),
                    );
                });
            }

            ui.add_space(24.0);
        });

        // ── Boton cerrar ──────────────────────────────────────────────────────
        ui.separator();
        ui.add_space(8.0);
        ui.vertical_centered(|ui| {
            if ui.button("  Cerrar  ").clicked() {
                app.ui.active = ActiveView::Main;
            }
        });
        ui.add_space(8.0);
    });
}
