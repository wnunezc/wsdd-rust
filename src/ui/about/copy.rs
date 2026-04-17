use crate::i18n::Language;

type DepRow = (&'static str, &'static str, &'static str);
type ToolRow = (&'static str, &'static str);

/// Localized static content used by the About dialog.
pub(super) struct AboutCopy {
    pub edition_line: &'static str,
    pub description: &'static str,
    pub copyright_label: &'static str,
    pub license_value: &'static str,
    pub platform_label: &'static str,
    pub platform_value: &'static str,
    pub migrated_from_label: &'static str,
    pub migrated_from_value: &'static str,
    pub gui_framework_label: &'static str,
    pub gui_framework_value: &'static str,
    pub fonts_label: &'static str,
    pub fonts_value: &'static str,
    pub dependencies_title: &'static str,
    pub external_tools_title: &'static str,
    pub dependencies: &'static [DepRow],
    pub tools: &'static [ToolRow],
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

const ABOUT_TOOLS_ZH: &[ToolRow] = &[
    ("Docker Desktop", "容器引擎 — docker.com"),
    ("WSL 2", "Linux 兼容层 — Microsoft"),
    ("Chocolatey", "Windows 包管理器 — chocolatey.org"),
    ("mkcert", "本地 SSL 证书 — github.com/FiloSottile/mkcert"),
    ("PowerShell 7+", "自动化 Shell — microsoft.com"),
];

/// Returns static localized copy for the About dialog.
pub(super) fn about_copy(language: Language) -> AboutCopy {
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
