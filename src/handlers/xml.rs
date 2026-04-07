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
// Equivalente a Handlers/HandlerXML.cs
// Lectura/escritura de configuracion XML (proyectos WSDD)

use anyhow::Result;
use quick_xml::de::from_str;
use quick_xml::se::to_string;
use serde::{Deserialize, Serialize};

pub fn read<T: for<'de> Deserialize<'de>>(path: &str) -> Result<T> {
    let content = std::fs::read_to_string(path)?;
    Ok(from_str(&content)?)
}

pub fn write<T: Serialize>(path: &str, value: &T) -> Result<()> {
    let xml = to_string(value)?;
    std::fs::write(path, xml)?;
    Ok(())
}
