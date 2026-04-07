// WebStack Deployer for Docker — Rust Edition
// Punto de entrada. Verificar privilegios, inicializar tracing y lanzar la UI.

#![windows_subsystem = "windows"]
#![allow(dead_code)] // handlers/models aún no conectados a la UI — se elimina al completar fases

mod app;
mod errors;
mod handlers;
mod models;
mod resources;
mod ui;

use anyhow::Result;

fn main() -> Result<()> {
    // Inicializar tracing (reemplaza env_logger)
    // RUST_LOG=wsdd=debug activa logs de debug; por defecto solo warn+
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("wsdd=info,warn")),
        )
        .init();

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "Iniciando WSDD");

    #[cfg(windows)]
    handlers::requirements::ensure_admin()?;

    resources::init()?;

    #[cfg(windows)]
    let title = "WebStack Deployer for Docker  ◈ Administrador";
    #[cfg(not(windows))]
    let title = "WebStack Deployer for Docker";

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(title)
            .with_min_inner_size([1000.0, 650.0])
            .with_maximized(true)
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "WSDD",
        options,
        Box::new(|cc| Ok(Box::new(app::WsddApp::new(cc)))),
    )
    .map_err(|e| anyhow::anyhow!("UI error: {e}"))
}

fn load_icon() -> egui::IconData {
    const ICON_BYTES: &[u8] = include_bytes!("../assets/WSDD-64.ico");
    match image::load_from_memory(ICON_BYTES) {
        Ok(img) => {
            let rgba = img.into_rgba8();
            let (width, height) = rgba.dimensions();
            egui::IconData { rgba: rgba.into_raw(), width, height }
        }
        Err(e) => {
            tracing::warn!("No se pudo cargar el icono: {e}");
            egui::IconData::default()
        }
    }
}
