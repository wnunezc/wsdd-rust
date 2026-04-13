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
//! Tipos de comunicación entre handlers y la capa de UI.
//!
//! Define los canales mpsc y los mensajes usados para enviar output de
//! procesos en tiempo real hacia la UI (patrón handler → canal → render loop).
//!
//! # Responsabilidad
//! Tipos de log y canales. No contiene lógica de ejecución.

use std::sync::mpsc;

// ─── Niveles ──────────────────────────────────────────────────────────────────

/// Nivel de importancia de una línea de log.
#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Info,
    Success,
    Warn,
    Error,
}

impl LogLevel {
    fn icon(&self) -> &'static str {
        match self {
            Self::Info => "ℹ",
            Self::Success => "✓",
            Self::Warn => "⚠",
            Self::Error => "✗",
        }
    }
}

// ─── LogLine ──────────────────────────────────────────────────────────────────

/// Línea de log con texto y nivel.
///
/// Enviada por los handlers al canal `LogSender` para que la UI
/// la muestre en el terminal del Loader con el color apropiado.
///
/// El campo `key` permite actualizaciones in-place: si se recibe una línea
/// con la misma key que una ya existente, la UI la reemplaza en lugar de añadir
/// una nueva. Útil para progress bars por capa de Docker.
#[derive(Debug, Clone)]
pub struct LogLine {
    pub text: String,
    pub level: LogLevel,
    /// Clave de actualización in-place. `None` = append normal.
    /// Si dos líneas comparten la misma key, la segunda reemplaza a la primera.
    pub key: Option<String>,
}

impl LogLine {
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            text: normalize_text(LogLevel::Info, text.into()),
            level: LogLevel::Info,
            key: None,
        }
    }

    pub fn success(text: impl Into<String>) -> Self {
        Self {
            text: normalize_text(LogLevel::Success, text.into()),
            level: LogLevel::Success,
            key: None,
        }
    }

    pub fn warn(text: impl Into<String>) -> Self {
        Self {
            text: normalize_text(LogLevel::Warn, text.into()),
            level: LogLevel::Warn,
            key: None,
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            text: normalize_text(LogLevel::Error, text.into()),
            level: LogLevel::Error,
            key: None,
        }
    }

    /// Asigna una key para actualizaciones in-place.
    ///
    /// Dos `LogLine` con la misma key comparten la misma posición en el log:
    /// la segunda reemplaza a la primera en vez de añadir una línea nueva.
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }
}

fn normalize_text(level: LogLevel, text: String) -> String {
    if text.trim().is_empty() {
        return text;
    }

    if has_leading_level_icon(&text) {
        return text;
    }

    format!("{} {}", level.icon(), text)
}

fn has_leading_level_icon(text: &str) -> bool {
    let trimmed = text.trim_start();
    ["ℹ", "✓", "⚠", "✗"]
        .iter()
        .any(|icon| trimmed.starts_with(icon))
}

// ─── Tipos de canal ───────────────────────────────────────────────────────────

/// Canal para enviar `LogLine`s en tiempo real desde handlers hacia la UI.
pub type LogSender = mpsc::Sender<LogLine>;

/// Canal para enviar texto plano desde procesos hacia la UI.
///
/// Usado por handlers que trabajan con strings sin nivel de log.
/// Definido aquí para centralizar todos los tipos de canal.
pub type OutputSender = mpsc::Sender<String>;
