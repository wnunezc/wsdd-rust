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
// Estado global de la aplicación. Equivalente a HandlerWSDD.cs.
// Mantiene estado entre frames de egui y coordina los handlers.

use std::sync::mpsc;
use std::time::Instant;

use crate::handlers::docker::{ContainerInfo, ContainerPollSnapshot, DockerDesktopStatus};
use crate::handlers::log_types::LogLine;
use crate::handlers::ps_script::PsRunner;
use crate::handlers::requirements::LoaderOutcome;
use crate::handlers::setting::AppSettings;
use crate::models::project::Project;
use crate::ui::{ActiveView, UiState};

pub struct WsddApp {
    pub settings: AppSettings,
    pub ui: UiState,
    pub first_run: bool,

    // ── Fase 3: proceso de requirements ──────────────────────────────────
    /// Canal de log: líneas enviadas por los handlers al Loader.
    pub requirement_rx: Option<mpsc::Receiver<LogLine>>,
    /// Canal de control: resultado final del proceso de requirements.
    pub loader_outcome_rx: Option<mpsc::Receiver<LoaderOutcome>>,
    /// `true` cuando el proceso de requirements terminó.
    pub loader_done: bool,
    /// `true` si Docker no está instalado (error bloqueante).
    pub loader_error: bool,
    /// `true` si el sistema necesita reiniciarse tras instalar Docker.
    pub loader_needs_reboot: bool,
    /// `true` en arranques posteriores (silencia el Loader si todo está OK).
    pub loader_silent: bool,
    /// Evita iniciar el proceso de requirements dos veces.
    pub requirements_started: bool,

    // ── Fase 5: panel principal ───────────────────────────────────────────
    /// Runner cloneable para operaciones en background threads.
    pub runner: PsRunner,
    /// Lista de contenedores WSDD (actualizada por polling).
    pub containers: Vec<ContainerInfo>,
    /// Lista de proyectos guardados en disco.
    pub projects: Vec<Project>,
    /// Log acumulado de operaciones del panel principal.
    pub main_log: Vec<LogLine>,
    /// Sender del canal de log principal (clonable para background threads).
    pub main_log_tx: mpsc::Sender<LogLine>,
    /// Receiver del canal de log principal (privado — solo drenado en drain_channels).
    main_log_rx: mpsc::Receiver<LogLine>,
    /// Receiver del polling de contenedores (presente mientras hay un poll activo).
    pub container_poll_rx: Option<mpsc::Receiver<ContainerPollSnapshot>>,
    /// `true` mientras un thread de polling está corriendo.
    pub container_poll_active: bool,
    /// Momento del último poll completado (usado para el intervalo de 3s).
    pub last_container_poll: Instant,
    /// Estado resumido de Docker Desktop usado por la barra inferior.
    pub docker_status: DockerDesktopStatus,
}

impl WsddApp {
    pub fn new(cc: &eframe::CreationContext<'_>, settings: AppSettings) -> Self {
        Self::setup_fonts(&cc.egui_ctx);

        crate::i18n::set_language(settings.language);
        let first_run = !settings.setup_completed;

        // Primera vez: Welcome → Loader
        // Arranques posteriores: Loader (silencioso) → Main o Loader visible si hay error
        let initial_view = if first_run {
            ActiveView::Welcome
        } else {
            ActiveView::Loader
        };

        let (main_log_tx, main_log_rx) = mpsc::channel::<LogLine>();

        Self {
            first_run,
            ui: UiState::new(initial_view),
            settings,
            requirement_rx: None,
            loader_outcome_rx: None,
            loader_done: false,
            loader_error: false,
            loader_needs_reboot: false,
            loader_silent: !first_run,
            requirements_started: false,
            runner: PsRunner::new(),
            containers: Vec::new(),
            projects: Vec::new(),
            main_log: Vec::new(),
            main_log_tx,
            main_log_rx,
            container_poll_rx: None,
            container_poll_active: false,
            last_container_poll: Instant::now()
                .checked_sub(std::time::Duration::from_secs(10))
                .unwrap_or_else(Instant::now),
            docker_status: DockerDesktopStatus::default(),
        }
    }

    /// Registra fuentes embebidas en el binario (OFL — libres para distribución).
    ///
    /// Estrategia de familias:
    /// - `Proportional`: Inter (egui default) → Noto Sans Symbols 2 (fallback símbolos)
    /// - `Monospace`:    JetBrains Mono → Noto Sans Symbols 2 (fallback)
    ///
    /// Noto Sans Symbols 2 cubre los rangos ausentes en Inter/JetBrains:
    /// flechas (→ ↺ ⟳), dingbats (✓ ✗), símbolos misc (⚡ ⬛ ⚙), etc.
    fn setup_fonts(ctx: &egui::Context) {
        const JETBRAINS_MONO: &[u8] = include_bytes!("../assets/JetBrainsMono-Regular.ttf");
        const NOTO_SYMBOLS: &[u8] = include_bytes!("../assets/NotoSansSymbols2-Regular.ttf");

        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "jetbrains_mono".to_owned(),
            egui::FontData::from_static(JETBRAINS_MONO),
        );
        fonts.font_data.insert(
            "noto_symbols".to_owned(),
            egui::FontData::from_static(NOTO_SYMBOLS),
        );
        add_windows_font_if_exists(&mut fonts, "windows_cjk", r"C:\Windows\Fonts\msyh.ttc", 0);
        add_windows_font_if_exists(
            &mut fonts,
            "windows_indic",
            r"C:\Windows\Fonts\Nirmala.ttc",
            0,
        );
        add_windows_font_if_exists(&mut fonts, "windows_ui", r"C:\Windows\Fonts\segoeui.ttf", 0);

        // Monospace: JetBrains Mono primero, Noto Symbols como fallback
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "jetbrains_mono".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("noto_symbols".to_owned());
        push_if_present(&mut fonts, egui::FontFamily::Monospace, "windows_cjk");
        push_if_present(&mut fonts, egui::FontFamily::Monospace, "windows_indic");

        // Proportional: preferir Segoe UI en Windows para una lectura mas limpia,
        // manteniendo fallbacks para simbolos y alfabetos adicionales.
        if fonts.font_data.contains_key("windows_ui") {
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "windows_ui".to_owned());
        }
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("noto_symbols".to_owned());
        push_if_present(&mut fonts, egui::FontFamily::Proportional, "windows_cjk");
        push_if_present(&mut fonts, egui::FontFamily::Proportional, "windows_indic");

        ctx.set_fonts(fonts);
    }

    /// Drena todos los canales pendientes. Llamar al inicio de cada frame.
    pub fn drain_channels(&mut self) {
        // Log principal — sobreescritura in-place por key (igual que loader)
        while let Ok(line) = self.main_log_rx.try_recv() {
            if let Some(ref key) = line.key.clone() {
                if let Some(existing) = self
                    .main_log
                    .iter_mut()
                    .find(|l| l.key.as_deref() == Some(key.as_str()))
                {
                    *existing = line;
                } else {
                    self.main_log.push(line);
                }
            } else {
                let is_dup = self
                    .main_log
                    .last()
                    .is_some_and(|last| last.text == line.text);
                if !is_dup {
                    self.main_log.push(line);
                }
            }
        }

        // Resultado del polling de contenedores
        let mut poll_done = false;
        if let Some(rx) = &self.container_poll_rx {
            if let Ok(snapshot) = rx.try_recv() {
                self.containers = snapshot.containers;
                self.docker_status = snapshot.docker_status;
                poll_done = true;
            }
        }
        if poll_done {
            self.container_poll_rx = None;
            self.container_poll_active = false;
            self.last_container_poll = Instant::now();
        }
    }
}

impl eframe::App for WsddApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Aplicar tema antes de renderizar (inmediato — se aplica en este frame)
        crate::ui::theme::apply(ctx, self.settings.theme);
        self.drain_channels();
        crate::ui::render(ctx, self);
    }
}

fn add_windows_font_if_exists(
    fonts: &mut egui::FontDefinitions,
    name: &str,
    path: &str,
    index: u32,
) {
    if let Ok(bytes) = std::fs::read(path) {
        fonts.font_data.insert(
            name.to_owned(),
            egui::FontData {
                font: bytes.into(),
                index,
                tweak: Default::default(),
            },
        );
    }
}

fn push_if_present(fonts: &mut egui::FontDefinitions, family: egui::FontFamily, name: &str) {
    if fonts.font_data.contains_key(name) {
        fonts
            .families
            .entry(family)
            .or_default()
            .push(name.to_owned());
    }
}
