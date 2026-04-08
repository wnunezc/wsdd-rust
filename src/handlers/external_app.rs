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
// Equivalente a Handlers/HandlerExternalApp.cs
use anyhow::Result;
use std::process::Command;

/// Abre una URL en el navegador por defecto.
pub fn open_url(url: &str) -> Result<()> {
    // `start` interpreta el primer argumento entre comillas como título de
    // ventana; pasar `""` evita que la URL se trate incorrectamente.
    Command::new("cmd").args(["/c", "start", "", url]).spawn()?;
    Ok(())
}

/// Abre el explorador de archivos en la ruta indicada.
pub fn open_explorer(path: &str) -> Result<()> {
    Command::new("explorer").arg(path).spawn()?;
    Ok(())
}
