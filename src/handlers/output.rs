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
// Equivalente a HandlerOutput.cs + OutputToTerminal.cs
// Canal de salida hacia la UI — usa un Vec<String> compartido via Arc<Mutex>

use std::sync::{Arc, Mutex};

pub type TerminalBuffer = Arc<Mutex<Vec<String>>>;

pub fn new_buffer() -> TerminalBuffer {
    Arc::new(Mutex::new(Vec::new()))
}

pub fn append(buffer: &TerminalBuffer, line: impl Into<String>) {
    if let Ok(mut buf) = buffer.lock() {
        buf.push(line.into());
    }
}
