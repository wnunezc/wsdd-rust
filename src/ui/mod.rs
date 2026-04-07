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
// Capa de UI. Equivalente a Forms/ en C#.
// Cada módulo corresponde a un Form del proyecto original.

pub mod about;
pub mod add_project;
pub mod containers_panel;
pub mod display_selector;
pub mod helps;
pub mod loader;
pub mod main_window;
pub mod projects_panel;
pub mod settings;
pub mod theme;
pub mod toolbox_container;
pub mod toolbox_project;
pub mod welcome;
pub mod wsl_settings;

use std::sync::mpsc;

use crate::app::WsddApp;
use crate::handlers::log_types::LogLine;
use crate::handlers::setting::AppSettings;
use crate::handlers::wsl::WslConfig;
use crate::models::project::{EntryPoint, PhpVersion};

/// Vista activa de la aplicación.
///
/// Equivalente al formulario visible en WinForms.
#[derive(Debug, Clone, PartialEq)]
pub enum ActiveView {
    Welcome,
    Loader,
    Main,
    Settings,
    AddProject,
    About,
    Helps,
    WslSettings,
    ToolboxProject,
    ToolboxContainer,
}

/// Tab activa dentro del panel principal.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum MainTab {
    #[default]
    Containers,
    Projects,
}

/// Estado de la capa de UI compartido entre vistas.
///
/// Vive en `WsddApp` para que todas las vistas puedan leerlo y modificarlo.
pub struct UiState {
    pub active: ActiveView,
    /// Log acumulado del proceso de requirements (mostrado en el terminal del Loader).
    pub requirement_log: Vec<LogLine>,
    pub readme_checked: bool,
    pub md_cache: egui_commonmark::CommonMarkCache,
    /// Tab activa en el panel principal.
    pub active_main_tab: MainTab,
    /// Proyecto pendiente de confirmación para eliminar (nombre).
    pub confirm_remove_project: Option<String>,
    /// Nombre del proyecto seleccionado para el Toolbox.
    pub toolbox_project_name: Option<String>,
    /// Nombre del contenedor seleccionado para el Toolbox.
    pub toolbox_container_name: Option<String>,

    // ── Formulario AddProject ─────────────────────────────────────────────
    pub form_name: String,
    pub form_domain: String,
    pub form_php: PhpVersion,
    pub form_work_path: String,
    pub form_entry: EntryPoint,
    pub form_entry_custom: String,
    pub form_ssl: bool,
    pub form_error: Option<String>,
    /// Canal para recibir la carpeta seleccionada por el file dialog nativo.
    pub folder_pick_rx: Option<mpsc::Receiver<Option<String>>>,

    // ── Settings ──────────────────────────────────────────────────────────
    /// Copia de trabajo de AppSettings mientras el usuario edita en la pantalla Settings.
    /// None cuando no se está en la pantalla Settings.
    pub settings_draft: Option<AppSettings>,

    // ── WSL Settings ──────────────────────────────────────────────────────
    /// Copia de trabajo de WslConfig mientras el usuario edita en la pantalla WslSettings.
    /// None cuando no se está en la pantalla WslSettings.
    pub wsl_draft: Option<WslConfig>,

    // ── Helps ─────────────────────────────────────────────────────────────
    /// Texto de búsqueda en el panel de ayuda.
    pub helps_search: String,
}

impl UiState {
    pub fn new(initial: ActiveView) -> Self {
        Self {
            active: initial,
            requirement_log: Vec::new(),
            readme_checked: false,
            md_cache: egui_commonmark::CommonMarkCache::default(),
            active_main_tab: MainTab::default(),
            confirm_remove_project: None,
            toolbox_project_name: None,
            toolbox_container_name: None,
            form_name: String::new(),
            form_domain: String::new(),
            form_php: PhpVersion::default(),
            form_work_path: String::new(),
            form_entry: EntryPoint::default(),
            form_entry_custom: String::new(),
            form_ssl: true,
            form_error: None,
            folder_pick_rx: None,
            settings_draft: None,
            wsl_draft: None,
            helps_search: String::new(),
        }
    }

    /// Resetea los campos del formulario AddProject.
    pub fn reset_add_project_form(&mut self) {
        self.form_name = String::new();
        self.form_domain = String::new();
        self.form_php = PhpVersion::default();
        self.form_work_path = String::new();
        self.form_entry = EntryPoint::default();
        self.form_entry_custom = String::new();
        self.form_ssl = true;
        self.form_error = None;
        self.folder_pick_rx = None;
    }
}

/// Punto de entrada del renderizado. Equivalente al mensaje WM_PAINT en WinForms.
pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    match &app.ui.active.clone() {
        ActiveView::Welcome => welcome::render(ctx, app),
        ActiveView::Loader => loader::render(ctx, app),
        ActiveView::Main => main_window::render(ctx, app),
        ActiveView::Settings => settings::render(ctx, app),
        ActiveView::About => about::render(ctx, app),
        ActiveView::Helps => helps::render(ctx, app),
        ActiveView::WslSettings => wsl_settings::render(ctx, app),
        // Overlays modales — renderiza main_window como fondo y muestra el modal encima
        ActiveView::AddProject => {
            main_window::render(ctx, app);
            add_project::render(ctx, app);
        }
        ActiveView::ToolboxProject => {
            main_window::render(ctx, app);
            toolbox_project::render(ctx, app);
        }
        ActiveView::ToolboxContainer => {
            main_window::render(ctx, app);
            toolbox_container::render(ctx, app);
        }
    }
}
