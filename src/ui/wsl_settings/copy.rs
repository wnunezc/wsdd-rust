use crate::handlers::wsl::{MemoryReclaim, NetworkingMode};
use crate::i18n::Language;

/// Localized copy used by the WSL settings modal.
pub(super) struct WslCopy {
    pub path_prefix: &'static str,
    pub system_resources: &'static str,
    pub resource_note: &'static str,
    pub cpu_cores: &'static str,
    pub no_limit: &'static str,
    pub max_memory: &'static str,
    pub ram_recommendation: &'static str,
    pub swap_disabled: &'static str,
    pub swap_recommendation: &'static str,
    pub performance_memory: &'static str,
    pub memory_reclaim: &'static str,
    pub gui_apps: &'static str,
    pub gui_apps_hint: &'static str,
    pub network: &'static str,
    pub localhost_forwarding: &'static str,
    pub localhost_hint: &'static str,
    pub network_mode: &'static str,
    pub dns_tunneling: &'static str,
    pub windows_firewall: &'static str,
    pub restart_note: &'static str,
    detected_cpu_fmt: &'static str,
}

impl WslCopy {
    /// Formats the detected CPU count helper text.
    pub(super) fn detected_cpu(&self, cpu_max: u32) -> String {
        self.detected_cpu_fmt
            .replace("{count}", &cpu_max.to_string())
    }
}

/// Returns localized static copy for the WSL settings modal.
pub(super) fn wsl_copy(language: Language) -> WslCopy {
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
        Language::Fr => WslCopy {
            path_prefix: "→",
            system_resources: "Ressources systeme",
            resource_note: "WSDD recommande de ne pas depasser 60-70% de la RAM et du CPU physiques pour garder l'hote Windows reactif.",
            cpu_cores: "Coeurs CPU:",
            no_limit: "Sans limite",
            max_memory: "RAM max:",
            ram_recommendation: "(recommande: 60% de la RAM physique)",
            swap_disabled: "Desactive",
            swap_recommendation: "(recommande: 0 avec suffisamment de RAM)",
            performance_memory: "Performance et memoire",
            memory_reclaim: "Recuperation memoire:",
            gui_apps: "Applications GUI (WSLg):",
            gui_apps_hint: "Desactiver ameliore les performances si vous n'utilisez pas les apps Linux GUI",
            network: "Reseau",
            localhost_forwarding: "Localhost forwarding:",
            localhost_hint: "Acceder aux services WSL2 via 127.0.0.1 depuis Windows",
            network_mode: "Mode reseau:",
            dns_tunneling: "DNS Tunneling:",
            windows_firewall: "Pare-feu Windows:",
            restart_note: "⚠  Les modifications necessitent un redemarrage de WSL2.\n    Executer dans PowerShell: wsl --shutdown",
            detected_cpu_fmt: "(detectes: {count} logiques)",
        },
        Language::Hi => default_english_copy(),
        Language::Zh => WslCopy {
            path_prefix: "→",
            system_resources: "系统资源",
            resource_note: "WSDD 建议保持在物理 RAM 和 CPU 的 60-70% 以下，以保持 Windows 主机响应流畅。",
            cpu_cores: "CPU 核心:",
            no_limit: "无限制",
            max_memory: "最大 RAM:",
            ram_recommendation: "(建议: 物理 RAM 的 60%)",
            swap_disabled: "已禁用",
            swap_recommendation: "(建议: RAM 充足时设为 0)",
            performance_memory: "性能和内存",
            memory_reclaim: "内存回收:",
            gui_apps: "GUI 应用 (WSLg):",
            gui_apps_hint: "如果不使用 Linux GUI 应用，禁用此项可提高性能",
            network: "网络",
            localhost_forwarding: "Localhost 转发:",
            localhost_hint: "从 Windows 通过 127.0.0.1 访问 WSL2 服务",
            network_mode: "网络模式:",
            dns_tunneling: "DNS 隧道:",
            windows_firewall: "Windows 防火墙:",
            restart_note: "⚠  更改需要重启 WSL2 才能生效。\n    在 PowerShell 中运行: wsl --shutdown",
            detected_cpu_fmt: "(检测到: {count} 个逻辑核心)",
        },
        _ => default_english_copy(),
    }
}

/// Returns the localized display label for WSL networking mode.
pub(super) fn networking_mode_label(mode: &NetworkingMode, language: Language) -> &'static str {
    match (language, mode) {
        (Language::Es, NetworkingMode::Nat) => "NAT (recomendado)",
        (Language::Es, NetworkingMode::Mirrored) => "Mirrored (experimental, Win11 23H2+)",
        (Language::Fr, NetworkingMode::Nat) => "NAT (recommande)",
        (Language::Fr, NetworkingMode::Mirrored) => "Mirrored (experimental, Win11 23H2+)",
        (Language::Zh, NetworkingMode::Nat) => "NAT (推荐)",
        (Language::Zh, NetworkingMode::Mirrored) => "Mirrored (实验性, Win11 23H2+)",
        (_, NetworkingMode::Nat) => "NAT (recommended)",
        (_, NetworkingMode::Mirrored) => "Mirrored (experimental, Win11 23H2+)",
    }
}

/// Returns the localized display label for WSL memory reclaim mode.
pub(super) fn memory_reclaim_label(mode: &MemoryReclaim, language: Language) -> &'static str {
    match (language, mode) {
        (Language::Es, MemoryReclaim::Disabled) => "Deshabilitado",
        (Language::Es, MemoryReclaim::Gradual) => "Gradual (recomendado)",
        (Language::Es, MemoryReclaim::DropCache) => "Drop Cache (agresivo)",
        (Language::Fr, MemoryReclaim::Disabled) => "Desactive",
        (Language::Fr, MemoryReclaim::Gradual) => "Graduel (recommande)",
        (Language::Fr, MemoryReclaim::DropCache) => "Drop Cache (agressif)",
        (Language::Zh, MemoryReclaim::Disabled) => "禁用",
        (Language::Zh, MemoryReclaim::Gradual) => "渐进式 (推荐)",
        (Language::Zh, MemoryReclaim::DropCache) => "Drop Cache (激进)",
        (_, MemoryReclaim::Disabled) => "Disabled",
        (_, MemoryReclaim::Gradual) => "Gradual (recommended)",
        (_, MemoryReclaim::DropCache) => "Drop cache (aggressive)",
    }
}

fn default_english_copy() -> WslCopy {
    WslCopy {
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
    }
}
