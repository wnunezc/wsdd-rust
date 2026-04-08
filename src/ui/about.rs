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
use crate::i18n::{tr, Language};
use crate::ui::ActiveView;

pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    let copy = about_copy(app.settings.language);

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
                egui::RichText::new(copy.edition_line)
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
                    ui.label(copy.description);
                });

            ui.add_space(14.0);

            // ── Info general ──────────────────────────────────────────────────
            egui::Grid::new("about_info")
                .num_columns(2)
                .spacing([20.0, 6.0])
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(tr("about_author")).strong());
                    ui.label("Walter Nunez / Icaros Net S.A");
                    ui.end_row();

                    ui.label(egui::RichText::new(copy.copyright_label).strong());
                    ui.label("(c) 2026 Walter Nunez / Icaros Net S.A");
                    ui.end_row();

                    ui.label(egui::RichText::new(tr("about_license")).strong());
                    ui.label(copy.license_value);
                    ui.end_row();

                    ui.label(egui::RichText::new(copy.platform_label).strong());
                    ui.label(copy.platform_value);
                    ui.end_row();

                    ui.label(egui::RichText::new(copy.migrated_from_label).strong());
                    ui.label(copy.migrated_from_value);
                    ui.end_row();

                    ui.label(egui::RichText::new(copy.gui_framework_label).strong());
                    ui.label(copy.gui_framework_value);
                    ui.end_row();

                    ui.label(egui::RichText::new(copy.fonts_label).strong());
                    ui.label(copy.fonts_value);
                    ui.end_row();
                });

            ui.add_space(14.0);
            ui.separator();
            ui.add_space(10.0);

            // ── Dependencias ──────────────────────────────────────────────────
            ui.label(egui::RichText::new(copy.dependencies_title).strong());
            ui.add_space(6.0);

            for (name, purpose, license) in copy.dependencies {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(format!("• {name}"))
                            .monospace()
                            .size(12.0),
                    );
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
            ui.label(egui::RichText::new(copy.external_tools_title).strong());
            ui.add_space(6.0);

            for (tool, desc) in copy.tools {
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
            if ui.button(format!("  {}  ", tr("btn_close"))).clicked() {
                app.ui.active = ActiveView::Main;
            }
        });
        ui.add_space(8.0);
    });
}

type DepRow = (&'static str, &'static str, &'static str);
type ToolRow = (&'static str, &'static str);

struct AboutCopy {
    edition_line: &'static str,
    description: &'static str,
    copyright_label: &'static str,
    license_value: &'static str,
    platform_label: &'static str,
    platform_value: &'static str,
    migrated_from_label: &'static str,
    migrated_from_value: &'static str,
    gui_framework_label: &'static str,
    gui_framework_value: &'static str,
    fonts_label: &'static str,
    fonts_value: &'static str,
    dependencies_title: &'static str,
    external_tools_title: &'static str,
    dependencies: &'static [DepRow],
    tools: &'static [ToolRow],
}

const ABOUT_DEPS_EN: &[DepRow] = &[
    (
        "egui / eframe 0.29",
        "Immediate-mode GUI",
        "MIT / Apache 2.0",
    ),
    ("tokio 1", "Async runtime", "MIT"),
    (
        "serde / serde_json",
        "JSON serialization",
        "MIT / Apache 2.0",
    ),
    ("serde_yaml 0.9", "YAML serialization", "MIT / Apache 2.0"),
    ("quick-xml 0.36", "XML parsing", "MIT"),
    ("anyhow / thiserror", "Error handling", "MIT / Apache 2.0"),
    ("rfd 0.15", "Native file dialogs", "MIT"),
    ("egui_commonmark 0.18", "Markdown rendering", "MIT"),
    ("egui_extras 0.29", "UI extras", "MIT / Apache 2.0"),
    ("zip 2", "ZIP compression (resources)", "MIT"),
    ("walkdir 2", "Directory traversal", "MIT / Unlicense"),
    ("image 0.25", "Icon decoding", "MIT / Apache 2.0"),
    ("tracing / tracing-subscriber", "Observability", "MIT"),
    (
        "windows 0.58",
        "Windows API (UAC, Registry)",
        "MIT / Apache 2.0",
    ),
];

const ABOUT_DEPS_ES: &[DepRow] = &[
    (
        "egui / eframe 0.29",
        "GUI immediate-mode",
        "MIT / Apache 2.0",
    ),
    ("tokio 1", "Runtime async", "MIT"),
    (
        "serde / serde_json",
        "Serializacion JSON",
        "MIT / Apache 2.0",
    ),
    ("serde_yaml 0.9", "Serializacion YAML", "MIT / Apache 2.0"),
    ("quick-xml 0.36", "XML parsing", "MIT"),
    (
        "anyhow / thiserror",
        "Manejo de errores",
        "MIT / Apache 2.0",
    ),
    ("rfd 0.15", "File dialogs nativos", "MIT"),
    ("egui_commonmark 0.18", "Markdown rendering", "MIT"),
    ("egui_extras 0.29", "Extras de UI", "MIT / Apache 2.0"),
    ("zip 2", "Compresion ZIP (recursos)", "MIT"),
    ("walkdir 2", "Recorrido de directorios", "MIT / Unlicense"),
    ("image 0.25", "Decodificacion de iconos", "MIT / Apache 2.0"),
    ("tracing / tracing-subscriber", "Observabilidad", "MIT"),
    (
        "windows 0.58",
        "Windows API (UAC, Registry)",
        "MIT / Apache 2.0",
    ),
];

const ABOUT_TOOLS_EN: &[ToolRow] = &[
    ("Docker Desktop", "Container engine — docker.com"),
    ("WSL 2", "Linux compatibility layer — Microsoft"),
    ("Chocolatey", "Windows package manager — chocolatey.org"),
    (
        "mkcert",
        "Local SSL certificates — github.com/FiloSottile/mkcert",
    ),
    ("PowerShell 7+", "Automation shell — microsoft.com"),
];

const ABOUT_TOOLS_ES: &[ToolRow] = &[
    ("Docker Desktop", "Motor de contenedores — docker.com"),
    ("WSL 2", "Capa de compatibilidad Linux — Microsoft"),
    ("Chocolatey", "Gestor de paquetes Windows — chocolatey.org"),
    (
        "mkcert",
        "Certificados SSL locales — github.com/FiloSottile/mkcert",
    ),
    ("PowerShell 7+", "Shell de automatizacion — microsoft.com"),
];

fn about_copy(language: Language) -> AboutCopy {
    match language {
        Language::Es => AboutCopy {
            edition_line: "Rust Edition — egui 0.29",
            description: "Herramienta de gestion de stacks PHP + Docker para entornos de \
desarrollo local en Windows. Automatiza deploy, certificados SSL, \
configuracion de hosts y gestion de contenedores.",
            copyright_label: "Copyright",
            license_value: "Propietaria — solo uso de desarrollo",
            platform_label: "Plataforma",
            platform_value: "Windows 10 / 11 (requiere privilegios de administrador)",
            migrated_from_label: "Migrado desde",
            migrated_from_value: "C# WinForms / .NET 8.0",
            gui_framework_label: "Framework GUI",
            gui_framework_value: "egui 0.29 / eframe (Rust stable)",
            fonts_label: "Fuentes",
            fonts_value: "JetBrains Mono v2.304 (OFL)  +  Noto Sans Symbols 2 v2.008 (OFL)  +  Windows fallbacks",
            dependencies_title: "Dependencias de codigo abierto",
            external_tools_title: "Herramientas externas requeridas",
            dependencies: ABOUT_DEPS_ES,
            tools: ABOUT_TOOLS_ES,
        },
        _ => AboutCopy {
            edition_line: "Rust Edition — egui 0.29",
            description: "PHP + Docker stack manager for local Windows development. \
It automates deploy flows, local SSL certificates, hosts configuration, \
and container management.",
            copyright_label: "Copyright",
            license_value: "Proprietary — development use only",
            platform_label: "Platform",
            platform_value: "Windows 10 / 11 (administrator privileges required)",
            migrated_from_label: "Migrated from",
            migrated_from_value: "C# WinForms / .NET 8.0",
            gui_framework_label: "GUI framework",
            gui_framework_value: "egui 0.29 / eframe (Rust stable)",
            fonts_label: "Fonts",
            fonts_value: "JetBrains Mono v2.304 (OFL)  +  Noto Sans Symbols 2 v2.008 (OFL)  +  Windows fallbacks",
            dependencies_title: "Open source dependencies",
            external_tools_title: "Required external tools",
            dependencies: ABOUT_DEPS_EN,
            tools: ABOUT_TOOLS_EN,
        },
    }
}
