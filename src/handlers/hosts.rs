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
//! Windows hosts file facade for WSDD domains.

use anyhow::{Context, Result};
use std::fs;

use crate::config::environment::path_config;
use crate::handlers::log_types::{LogLine, LogSender};

mod antivirus;
mod block;
mod elevation;
mod file;

/// Removes every WSDD-managed entry from the Windows hosts file.
pub fn remove_wsdd_entries() -> Result<()> {
    let content = fs::read_to_string(path_config().hosts_file())?;
    let cleaned = block::normalize_crlf(block::remove_wsdd_block(&content));
    file::backup_hosts()?;
    file::write_hosts_file(cleaned.as_bytes(), None)?;
    Ok(())
}

/// Updates the WSDD block with base service domains and optional project domains.
///
/// Existing WSDD entries are preserved and merged with the requested domains.
/// Callers must not persist setup completion when this function returns an error.
pub fn update_host(extra_domains: Option<&[&str]>, tx: &LogSender) -> Result<()> {
    let _ = tx.send(LogLine::info("Verificando archivo hosts de Windows..."));

    #[cfg(windows)]
    if !elevation::is_elevated() {
        let _ = tx.send(LogLine::warn(
            "Advertencia: el proceso NO está elevado — hosts.rs necesita privilegios de administrador",
        ));
    }

    let mut domains = block::default_domains(extra_domains);
    let content = fs::read_to_string(path_config().hosts_file())
        .context("No se pudo leer el archivo hosts")?;
    let updated = block::upsert_wsdd_block(&content, &mut domains);

    if let Err(e) = file::backup_hosts() {
        let _ = tx.send(LogLine::warn(format!(
            "Advertencia: no se pudo crear backup de hosts: {e:#}"
        )));
    }

    file::write_hosts_file(updated.as_bytes(), Some(tx))?;

    let _ = tx.send(LogLine::success(
        "✓ Archivo hosts actualizado correctamente",
    ));
    for domain in &domains {
        let _ = tx.send(LogLine::info(format!("  → http://{domain}")));
    }
    Ok(())
}

/// Captures the current hosts file bytes for rollback.
pub fn capture_snapshot() -> Result<Vec<u8>> {
    file::capture_snapshot()
}

/// Restores a previously captured hosts file snapshot.
pub fn restore_snapshot(snapshot: &[u8], tx: Option<&LogSender>) -> Result<()> {
    file::write_hosts_file(snapshot, tx)
}

/// Removes specific domains from the WSDD-managed hosts block.
pub fn remove_domains(domains_to_remove: &[&str], tx: &LogSender) -> Result<()> {
    let _ = tx.send(LogLine::info("Verificando archivo hosts de Windows..."));

    let content = fs::read_to_string(path_config().hosts_file())
        .context("No se pudo leer el archivo hosts")?;
    let updated = block::remove_domains_from_block(&content, domains_to_remove);

    if updated.replace("\r\n", "\n") == content.replace("\r\n", "\n") {
        let _ = tx.send(LogLine::info(
            "No hubo cambios en hosts para los dominios solicitados.",
        ));
        return Ok(());
    }

    if let Err(e) = file::backup_hosts() {
        let _ = tx.send(LogLine::warn(format!(
            "Advertencia: no se pudo crear backup de hosts: {e:#}"
        )));
    }

    let normalized = block::normalize_crlf(updated);
    file::write_hosts_file(normalized.as_bytes(), Some(tx))?;
    let _ = tx.send(LogLine::success(
        "✓ Archivo hosts actualizado correctamente",
    ));
    Ok(())
}
