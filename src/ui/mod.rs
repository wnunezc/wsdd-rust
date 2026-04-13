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
use crate::handlers::setting::{AppSettings, PrereqCredentials, WebminCredentials};
use crate::handlers::wsl::WslConfig;
use crate::i18n::{tr, trf};
use crate::models::project::{EntryPoint, PhpVersion, Project};

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

pub struct PrereqCredentialsPromptState {
    pub draft: PrereqCredentials,
    pub error: Option<String>,
}

pub struct WebminCredentialsPromptState {
    pub project: Project,
    pub add_project_to_list: bool,
    pub username: String,
    pub password: String,
    pub error: Option<String>,
}

/// Estado de la capa de UI compartido entre vistas.
///
/// Vive en `WsddApp` para que todas las vistas puedan leerlo y modificarlo.
pub struct UiState {
    pub active: ActiveView,
    /// Log acumulado del proceso de requirements (mostrado en el terminal del Loader).
    pub requirement_log: Vec<LogLine>,
    pub readme_checked: bool,
    pub welcome_error: Option<String>,
    pub prereq_prompt: Option<PrereqCredentialsPromptState>,
    pub webmin_prompt: Option<WebminCredentialsPromptState>,
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
    /// Error visible de validación/guardado en Settings.
    pub settings_error: Option<String>,

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
            welcome_error: None,
            prereq_prompt: None,
            webmin_prompt: None,
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
            settings_error: None,
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

    pub fn open_prereq_prompt(&mut self, current: &PrereqCredentials) {
        self.prereq_prompt = Some(PrereqCredentialsPromptState {
            draft: current.clone(),
            error: None,
        });
    }

    pub fn open_webmin_prompt(&mut self, project: Project, add_project_to_list: bool) {
        self.webmin_prompt = Some(WebminCredentialsPromptState {
            project,
            add_project_to_list,
            username: "admin".to_string(),
            password: String::new(),
            error: None,
        });
    }
}

/// Punto de entrada del renderizado. Equivalente al mensaje WM_PAINT en WinForms.
pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    match &app.ui.active.clone() {
        ActiveView::Welcome => welcome::render(ctx, app),
        ActiveView::Loader => loader::render(ctx, app),
        ActiveView::Main => main_window::render(ctx, app),
        ActiveView::Settings => {
            main_window::render(ctx, app);
            settings::render(ctx, app);
        }
        ActiveView::About => {
            main_window::render(ctx, app);
            about::render(ctx, app);
        }
        ActiveView::Helps => helps::render(ctx, app),
        ActiveView::WslSettings => {
            main_window::render(ctx, app);
            wsl_settings::render(ctx, app);
        }
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

    render_prereq_credentials_dialog(ctx, app);
    render_webmin_credentials_dialog(ctx, app);
}

pub(crate) fn render_modal_backdrop(ctx: &egui::Context, id: &'static str) {
    let screen_rect = ctx.screen_rect();

    egui::Area::new(egui::Id::new(id))
        .order(egui::Order::Middle)
        .fixed_pos(screen_rect.min)
        .interactable(true)
        .show(ctx, |ui| {
            ui.set_min_size(screen_rect.size());
            let rect = ui.max_rect();
            let response = ui.allocate_rect(rect, egui::Sense::click());
            ui.painter()
                .rect_filled(rect, 0.0, egui::Color32::from_black_alpha(160));

            if response.clicked() {
                ui.ctx().request_repaint();
            }
        });
}

fn render_prereq_credentials_dialog(ctx: &egui::Context, app: &mut WsddApp) {
    if app.ui.prereq_prompt.is_none() {
        return;
    }

    let title = tr("dialog_prereq_title");
    let note = tr("dialog_prereq_note");
    let mysql_database_label = format!("{}:", tr("settings_mysql_database"));
    let mysql_user_label = format!("{}:", tr("settings_mysql_user"));
    let mysql_password_label = format!("{}:", tr("settings_mysql_password"));
    let mysql_root_password_label = format!("{}:", tr("settings_mysql_root_password"));
    let cancel_label = tr("btn_cancel");
    let save_label = tr("btn_save");

    let mut save = false;
    let mut cancel = false;

    render_modal_backdrop(ctx, "prereq_credentials_backdrop");

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            let prompt = app
                .ui
                .prereq_prompt
                .as_mut()
                .expect("missing prereq prompt");

            ui.label(
                egui::RichText::new(&note)
                    .size(11.0)
                    .color(ui.visuals().weak_text_color()),
            );
            ui.add_space(8.0);

            egui::Grid::new("prereq_credentials_dialog")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .min_col_width(190.0)
                .show(ui, |ui| {
                    ui.label(&mysql_database_label);
                    ui.add(
                        egui::TextEdit::singleline(&mut prompt.draft.mysql_database)
                            .desired_width(220.0)
                            .hint_text("wsdd-database"),
                    );
                    ui.end_row();

                    ui.label(&mysql_user_label);
                    ui.add(
                        egui::TextEdit::singleline(&mut prompt.draft.mysql_user)
                            .desired_width(220.0)
                            .hint_text("tester"),
                    );
                    ui.end_row();

                    ui.label(&mysql_password_label);
                    ui.add(
                        egui::TextEdit::singleline(&mut prompt.draft.mysql_password)
                            .desired_width(220.0)
                            .password(true)
                            .hint_text("required"),
                    );
                    ui.end_row();

                    ui.label(&mysql_root_password_label);
                    ui.add(
                        egui::TextEdit::singleline(&mut prompt.draft.mysql_root_password)
                            .desired_width(220.0)
                            .password(true)
                            .hint_text("required"),
                    );
                    ui.end_row();
                });

            if let Some(error) = &prompt.error {
                ui.add_space(6.0);
                ui.label(egui::RichText::new(error).color(egui::Color32::from_rgb(220, 80, 80)));
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button(&save_label).clicked() {
                    save = true;
                }
                if ui.button(&cancel_label).clicked() {
                    cancel = true;
                }
            });
        });

    if save {
        if let Some(mut prompt) = app.ui.prereq_prompt.take() {
            if let Err(e) = prompt.draft.validate_for_save() {
                prompt.error = Some(e.to_string());
                app.ui.prereq_prompt = Some(prompt);
                return;
            }

            let draft = prompt.draft.clone();
            app.settings.prereq_credentials = draft;
            if let Err(e) = app.settings.save() {
                prompt.error = Some(format!("Error guardando configuracion: {e}"));
                app.ui.prereq_prompt = Some(prompt);
                return;
            }

            if let Err(e) =
                crate::handlers::docker_deploy::sync_prerequisite_compose_sync(&app.settings)
            {
                prompt.error = Some(format!("Error preparando init.yml: {e}"));
                app.ui.prereq_prompt = Some(prompt);
            }
        }
    } else if cancel {
        app.ui.prereq_prompt = None;
        if app.ui.active == ActiveView::Loader && !app.settings.setup_completed {
            app.ui.active = ActiveView::Welcome;
        }
    }
}

fn render_webmin_credentials_dialog(ctx: &egui::Context, app: &mut WsddApp) {
    if app.ui.webmin_prompt.is_none() {
        return;
    }

    let title = {
        let prompt = app
            .ui
            .webmin_prompt
            .as_ref()
            .expect("missing webmin prompt");
        trf(
            "dialog_webmin_title",
            &[("php", prompt.project.php_version.display_name())],
        )
    };
    let note = tr("dialog_webmin_note");
    let user_label = format!("{}:", tr("settings_webmin_user"));
    let password_label = format!("{}:", tr("settings_webmin_password"));
    let cancel_label = tr("btn_cancel");
    let save_label = tr("btn_save");

    let mut save = false;
    let mut cancel = false;

    render_modal_backdrop(ctx, "webmin_credentials_backdrop");

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            let prompt = app
                .ui
                .webmin_prompt
                .as_mut()
                .expect("missing webmin prompt");

            ui.label(
                egui::RichText::new(&note)
                    .size(11.0)
                    .color(ui.visuals().weak_text_color()),
            );
            ui.add_space(8.0);

            egui::Grid::new("webmin_credentials_dialog")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .min_col_width(190.0)
                .show(ui, |ui| {
                    ui.label(&user_label);
                    ui.add(
                        egui::TextEdit::singleline(&mut prompt.username)
                            .desired_width(220.0)
                            .hint_text("admin"),
                    );
                    ui.end_row();

                    ui.label(&password_label);
                    ui.add(
                        egui::TextEdit::singleline(&mut prompt.password)
                            .desired_width(220.0)
                            .password(true)
                            .hint_text("required"),
                    );
                    ui.end_row();
                });

            if let Some(error) = &prompt.error {
                ui.add_space(6.0);
                ui.label(egui::RichText::new(error).color(egui::Color32::from_rgb(220, 80, 80)));
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button(&save_label).clicked() {
                    save = true;
                }
                if ui.button(&cancel_label).clicked() {
                    cancel = true;
                }
            });
        });

    if save {
        if let Some(mut prompt) = app.ui.webmin_prompt.take() {
            let credentials = WebminCredentials {
                php_version: prompt.project.php_version.clone(),
                username: prompt.username.clone(),
                password: prompt.password.clone(),
            };

            if let Err(e) = app.settings.store_webmin_credentials(credentials) {
                prompt.error = Some(e.to_string());
                app.ui.webmin_prompt = Some(prompt);
                return;
            }

            if let Err(e) = app.settings.save() {
                prompt.error = Some(format!("Error guardando configuracion: {e}"));
                app.ui.webmin_prompt = Some(prompt);
                return;
            }

            if prompt.add_project_to_list
                && !app
                    .projects
                    .iter()
                    .any(|project| project.name == prompt.project.name)
            {
                app.projects.push(prompt.project.clone());
            }

            crate::ui::projects_panel::spawn_deploy(ctx, app, prompt.project);
            app.ui.form_error = None;
            app.ui.active = ActiveView::Main;
        }
    } else if cancel {
        app.ui.webmin_prompt = None;
    }
}
