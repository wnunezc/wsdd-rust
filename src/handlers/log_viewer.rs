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
// Equivalente a Handlers/HandlerLogViewer.cs
// Lectura de logs de contenedores Docker en tiempo real

use anyhow::Result;
use std::process::Command;

/// Obtiene los ultimos N lines de logs de un contenedor.
pub fn get_logs(container_name: &str, tail: usize) -> Result<String> {
    let output = Command::new("docker")
        .args(["logs", "--tail", &tail.to_string(), container_name])
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
