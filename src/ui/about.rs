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

const ABOUT_DEPS_FR: &[DepRow] = &[
    (
        "egui / eframe 0.29",
        "Interface immediate-mode",
        "MIT / Apache 2.0",
    ),
    ("tokio 1", "Runtime asynchrone", "MIT"),
    (
        "serde / serde_json",
        "Serialisation JSON",
        "MIT / Apache 2.0",
    ),
    ("serde_yaml 0.9", "Serialisation YAML", "MIT / Apache 2.0"),
    ("quick-xml 0.36", "Analyse XML", "MIT"),
    (
        "anyhow / thiserror",
        "Gestion des erreurs",
        "MIT / Apache 2.0",
    ),
    ("rfd 0.15", "Dialogues de fichiers natifs", "MIT"),
    ("egui_commonmark 0.18", "Rendu Markdown", "MIT"),
    ("egui_extras 0.29", "Extensions UI", "MIT / Apache 2.0"),
    ("zip 2", "Compression ZIP (ressources)", "MIT"),
    ("walkdir 2", "Parcours de repertoires", "MIT / Unlicense"),
    ("image 0.25", "Decodage d'icones", "MIT / Apache 2.0"),
    ("tracing / tracing-subscriber", "Observabilite", "MIT"),
    (
        "windows 0.58",
        "API Windows (UAC, Registre)",
        "MIT / Apache 2.0",
    ),
];

const ABOUT_TOOLS_FR: &[ToolRow] = &[
    ("Docker Desktop", "Moteur de conteneurs — docker.com"),
    ("WSL 2", "Couche de compatibilite Linux — Microsoft"),
    (
        "Chocolatey",
        "Gestionnaire de paquets Windows — chocolatey.org",
    ),
    (
        "mkcert",
        "Certificats SSL locaux — github.com/FiloSottile/mkcert",
    ),
    ("PowerShell 7+", "Shell d'automatisation — microsoft.com"),
];

const ABOUT_DEPS_HI: &[DepRow] = &[
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
    ("zip 2", "ZIP compression", "MIT"),
    ("walkdir 2", "Directory traversal", "MIT / Unlicense"),
    ("image 0.25", "Icon decoding", "MIT / Apache 2.0"),
    ("tracing / tracing-subscriber", "Observability", "MIT"),
    (
        "windows 0.58",
        "Windows API (UAC, Registry)",
        "MIT / Apache 2.0",
    ),
];

const ABOUT_TOOLS_HI: &[ToolRow] = &[
    ("Docker Desktop", "Container engine — docker.com"),
    ("WSL 2", "Linux compatibility layer — Microsoft"),
    ("Chocolatey", "Windows package manager — chocolatey.org"),
    (
        "mkcert",
        "Local SSL certificates — github.com/FiloSottile/mkcert",
    ),
    ("PowerShell 7+", "Automation shell — microsoft.com"),
];

const ABOUT_DEPS_ZH: &[DepRow] = &[
    ("egui / eframe 0.29", "即时模式 GUI", "MIT / Apache 2.0"),
    ("tokio 1", "异步运行时", "MIT"),
    ("serde / serde_json", "JSON 序列化", "MIT / Apache 2.0"),
    ("serde_yaml 0.9", "YAML 序列化", "MIT / Apache 2.0"),
    ("quick-xml 0.36", "XML 解析", "MIT"),
    ("anyhow / thiserror", "错误处理", "MIT / Apache 2.0"),
    ("rfd 0.15", "原生文件对话框", "MIT"),
    ("egui_commonmark 0.18", "Markdown 渲染", "MIT"),
    ("egui_extras 0.29", "UI 扩展", "MIT / Apache 2.0"),
    ("zip 2", "ZIP 压缩", "MIT"),
    ("walkdir 2", "目录遍历", "MIT / Unlicense"),
    ("image 0.25", "图标解码", "MIT / Apache 2.0"),
    ("tracing / tracing-subscriber", "可观测性", "MIT"),
    (
        "windows 0.58",
        "Windows API (UAC, 注册表)",
        "MIT / Apache 2.0",
    ),
];

const ABOUT_TOOLS_ZH: &[ToolRow] = &[
    ("Docker Desktop", "容器引擎 — docker.com"),
    ("WSL 2", "Linux 兼容层 — Microsoft"),
    ("Chocolatey", "Windows 包管理器 — chocolatey.org"),
    ("mkcert", "本地 SSL 证书 — github.com/FiloSottile/mkcert"),
    ("PowerShell 7+", "自动化 Shell — microsoft.com"),
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
        Language::Fr => AboutCopy {
            edition_line: "Rust Edition — egui 0.29",
            description: "Gestionnaire de stacks PHP + Docker pour le developpement local \
sous Windows. Automatise le deploiement, les certificats SSL, \
la configuration des hosts et la gestion des conteneurs.",
            copyright_label: "Droits d'auteur",
            license_value: "Proprietaire — usage de developpement uniquement",
            platform_label: "Plateforme",
            platform_value: "Windows 10 / 11 (privileges administrateur requis)",
            migrated_from_label: "Migre depuis",
            migrated_from_value: "C# WinForms / .NET 8.0",
            gui_framework_label: "Framework GUI",
            gui_framework_value: "egui 0.29 / eframe (Rust stable)",
            fonts_label: "Polices",
            fonts_value: "JetBrains Mono v2.304 (OFL)  +  Noto Sans Symbols 2 v2.008 (OFL)  +  Windows fallbacks",
            dependencies_title: "Dependances open source",
            external_tools_title: "Outils externes requis",
            dependencies: ABOUT_DEPS_FR,
            tools: ABOUT_TOOLS_FR,
        },
        Language::Hi => AboutCopy {
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
            dependencies: ABOUT_DEPS_HI,
            tools: ABOUT_TOOLS_HI,
        },
        Language::Zh => AboutCopy {
            edition_line: "Rust Edition — egui 0.29",
            description: "用于 Windows 本地开发的 PHP + Docker 堆栈管理器。\
自动化部署流程、本地 SSL 证书、hosts 配置和容器管理。",
            copyright_label: "版权",
            license_value: "专有 — 仅供开发使用",
            platform_label: "平台",
            platform_value: "Windows 10 / 11 (需要管理员权限)",
            migrated_from_label: "迁移自",
            migrated_from_value: "C# WinForms / .NET 8.0",
            gui_framework_label: "GUI 框架",
            gui_framework_value: "egui 0.29 / eframe (Rust stable)",
            fonts_label: "字体",
            fonts_value: "JetBrains Mono v2.304 (OFL)  +  Noto Sans Symbols 2 v2.008 (OFL)  +  Windows fallbacks",
            dependencies_title: "开源依赖",
            external_tools_title: "所需外部工具",
            dependencies: ABOUT_DEPS_ZH,
            tools: ABOUT_TOOLS_ZH,
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
