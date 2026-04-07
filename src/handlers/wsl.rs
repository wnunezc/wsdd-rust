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
//! Handler para leer y escribir `%USERPROFILE%\.wslconfig`.
//! Equivalente a la lógica de WSLGeneralSetting.cs — extendido con
//! parámetros de rendimiento adicionales recomendados para WSL2.

use crate::errors::InfraError;
use std::path::PathBuf;

// ── Tipos ─────────────────────────────────────────────────────────────────────

/// Modo de red para WSL2.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum NetworkingMode {
    /// NAT clásico — compatibilidad máxima (recomendado).
    #[default]
    Nat,
    /// Mirrored — acceso directo a la red del host (experimental, Windows 11 23H2+).
    Mirrored,
}

impl NetworkingMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Nat => "NAT",
            Self::Mirrored => "mirrored",
        }
    }

    pub fn from_str(s: &str) -> Self {
        if s.eq_ignore_ascii_case("mirrored") {
            Self::Mirrored
        } else {
            Self::Nat
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Nat => "NAT (recomendado)",
            Self::Mirrored => "Mirrored (experimental, Win11 23H2+)",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Nat, Self::Mirrored]
    }
}

/// Configuración de rendimiento WSL2 representada como struct tipado.
/// Se persiste en `%USERPROFILE%\.wslconfig` formato INI.
#[derive(Debug, Clone)]
pub struct WslConfig {
    /// Núcleos de CPU virtuales asignados a WSL2.
    /// `None` = sin límite (usa todos los disponibles).
    pub processors: Option<u32>,

    /// RAM máxima asignada a WSL2 en GB.
    /// `None` = sin límite (WSL2 gestiona dinámicamente, puede consumir toda la RAM).
    pub memory_gb: Option<u32>,

    /// Espacio de swap en GB. 0 = deshabilitar swap.
    pub swap_gb: u32,

    /// Reenviar conexiones localhost del host al contenedor WSL2.
    /// Recomendado: true para acceder a servicios con 127.0.0.1.
    pub localhost_forwarding: bool,

    /// Habilitar soporte de apps GUI (WSLg — Linux gráfico desde Windows).
    /// Desactivar mejora el rendimiento si no se necesita.
    pub gui_applications: bool,

    /// Liberar memoria de vuelta al host cuando los procesos WSL terminen.
    /// `gradual` o `dropcache` — mejora la recuperación de RAM en el host.
    pub memory_reclaim: MemoryReclaim,

    /// Modo de red.
    pub networking_mode: NetworkingMode,

    /// Habilitar DNS a través de firewall del host (solo con networkingMode=mirrored).
    pub dns_tunneling: bool,

    /// Habilitar firewall de Windows para WSL2 (solo con networkingMode=mirrored).
    pub firewall: bool,
}

/// Política de recuperación de memoria de WSL2 hacia el host.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum MemoryReclaim {
    /// Sin recuperación automática.
    #[default]
    Disabled,
    /// Recuperación gradual — balanceado.
    Gradual,
    /// Liberar cache de disco inmediatamente.
    DropCache,
}

impl MemoryReclaim {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Disabled => "none",
            Self::Gradual => "gradual",
            Self::DropCache => "dropcache",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "gradual" => Self::Gradual,
            "dropcache" | "drop_cache" => Self::DropCache,
            _ => Self::Disabled,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Disabled => "Deshabilitado",
            Self::Gradual => "Gradual (recomendado)",
            Self::DropCache => "Drop Cache (agresivo)",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Disabled, Self::Gradual, Self::DropCache]
    }
}

impl Default for WslConfig {
    fn default() -> Self {
        Self {
            processors: None,
            memory_gb: None,
            swap_gb: 0,
            localhost_forwarding: true,
            gui_applications: false,
            memory_reclaim: MemoryReclaim::default(),
            networking_mode: NetworkingMode::default(),
            dns_tunneling: false,
            firewall: false,
        }
    }
}

// ── Ruta ─────────────────────────────────────────────────────────────────────

fn wslconfig_path() -> PathBuf {
    let profile =
        std::env::var("USERPROFILE").unwrap_or_else(|_| r"C:\Users\Default".to_string());
    PathBuf::from(profile).join(".wslconfig")
}

// ── API pública ───────────────────────────────────────────────────────────────

/// Lee `.wslconfig`. Si no existe retorna `WslConfig::default()`.
pub fn read() -> Result<WslConfig, InfraError> {
    let path = wslconfig_path();
    if !path.exists() {
        return Ok(WslConfig::default());
    }
    let content = std::fs::read_to_string(&path)?;
    Ok(parse(&content))
}

/// Escribe `.wslconfig` con los valores del struct.
pub fn write(config: &WslConfig) -> Result<(), InfraError> {
    let path = wslconfig_path();
    let content = serialize(config);
    std::fs::write(&path, content)?;
    Ok(())
}

/// Ruta absoluta del archivo .wslconfig activo.
pub fn config_path_display() -> String {
    wslconfig_path().display().to_string()
}

// ── Parse / Serialize ─────────────────────────────────────────────────────────

fn parse(content: &str) -> WslConfig {
    let mut cfg = WslConfig::default();
    let mut in_wsl2 = false;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_wsl2 = line.eq_ignore_ascii_case("[wsl2]");
            continue;
        }
        if !in_wsl2 || line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some((key, val)) = line.split_once('=') {
            let key = key.trim().to_lowercase();
            let val = val.trim();
            match key.as_str() {
                "processors" => cfg.processors = val.parse().ok(),
                "memory" => cfg.memory_gb = parse_size_gb(val),
                "swap" => cfg.swap_gb = parse_size_gb(val).unwrap_or(0),
                "localhostforwarding" => {
                    cfg.localhost_forwarding = val.eq_ignore_ascii_case("true")
                }
                "guiapplications" => cfg.gui_applications = val.eq_ignore_ascii_case("true"),
                "memoryreclaimfile" | "memoryreclam" | "memoryreclaim" => {
                    cfg.memory_reclaim = MemoryReclaim::from_str(val)
                }
                "networkingmode" => cfg.networking_mode = NetworkingMode::from_str(val),
                "dnstunneling" => cfg.dns_tunneling = val.eq_ignore_ascii_case("true"),
                "firewall" => cfg.firewall = val.eq_ignore_ascii_case("true"),
                _ => {}
            }
        }
    }
    cfg
}

/// Parsea valores como "4GB", "4G", "4096MB" → GB (u32).
fn parse_size_gb(val: &str) -> Option<u32> {
    let upper = val.to_uppercase();
    if let Some(s) = upper.strip_suffix("GB") {
        return s.trim().parse().ok();
    }
    if let Some(s) = upper.strip_suffix('G') {
        return s.trim().parse().ok();
    }
    if let Some(s) = upper.strip_suffix("MB") {
        let mb: u32 = s.trim().parse().ok()?;
        return Some(mb / 1024);
    }
    upper.parse().ok()
}

fn serialize(config: &WslConfig) -> String {
    let mut lines: Vec<String> = vec!["[wsl2]".to_string()];

    if let Some(p) = config.processors {
        lines.push(format!("processors={p}"));
    }
    if let Some(m) = config.memory_gb {
        lines.push(format!("memory={m}GB"));
    }
    if config.swap_gb > 0 {
        lines.push(format!("swap={}GB", config.swap_gb));
    } else {
        lines.push("swap=0".to_string());
    }
    lines.push(format!(
        "localhostForwarding={}",
        bool_str(config.localhost_forwarding)
    ));
    lines.push(format!(
        "guiApplications={}",
        bool_str(config.gui_applications)
    ));
    if config.memory_reclaim != MemoryReclaim::Disabled {
        lines.push(format!("memoryReclaimFile={}", config.memory_reclaim.as_str()));
    }
    lines.push(format!(
        "networkingMode={}",
        config.networking_mode.as_str()
    ));
    if config.networking_mode == NetworkingMode::Mirrored {
        lines.push(format!("dnsTunneling={}", bool_str(config.dns_tunneling)));
        lines.push(format!("firewall={}", bool_str(config.firewall)));
    }

    lines.join("\n") + "\n"
}

fn bool_str(b: bool) -> &'static str {
    if b { "true" } else { "false" }
}
