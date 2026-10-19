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
//! Global application state facade.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use tokio::runtime::Runtime;

use crate::handlers::deploy_trace::TraceEvent;
use crate::handlers::docker::{ContainerInfo, ContainerPollSnapshot, DockerDesktopStatus};
use crate::handlers::log_types::LogLine;
use crate::handlers::ngrok::NgrokPublication;
use crate::handlers::ps_script::PsRunner;
use crate::handlers::requirements::LoaderOutcome;
use crate::handlers::setting::AppSettings;
use crate::models::project::Project;
use crate::ui::{ActiveView, UiState};

mod channels;
mod container_logs;
mod fonts;
mod jobs;
mod runtime;

pub use runtime::create_job_runtime;

pub type JobRuntime = &'static Runtime;

/// Current lifecycle state for a background job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackgroundJobStatus {
    Running,
    Succeeded,
    Failed,
}

/// Background job metadata shown and queried by UI actions.
#[derive(Debug, Clone)]
pub struct BackgroundJob {
    pub key: String,
    pub label: String,
    pub status: BackgroundJobStatus,
    pub started_at: Instant,
    pub finished_at: Option<Instant>,
    pub last_error: Option<String>,
}

/// One confirmed stage of a deployment operation.
#[derive(Debug, Clone)]
pub struct OperationStage {
    pub label: &'static str,
    pub started_at: Instant,
    pub finished_at: Option<Instant>,
    pub success: bool,
}

/// UI metadata for one operation and its complete, redacted diagnostic file.
#[derive(Debug, Clone)]
pub struct OperationTrace {
    pub key: String,
    pub label: String,
    pub log_path: PathBuf,
    pub stages: Vec<OperationStage>,
    pub buildkit_step: Option<String>,
    pub finished_at: Option<Instant>,
    pub success: bool,
}

/// One live log line emitted by a Docker container.
#[derive(Debug, Clone)]
pub struct ContainerLogEntry {
    pub container_name: String,
    pub text: String,
}

#[derive(Debug)]
struct ContainerLogWatcher {
    stop_tx: mpsc::Sender<()>,
}

#[derive(Debug)]
pub enum NgrokPublicationEvent {
    Published(NgrokPublication),
    Stopped { project_name: String },
}

#[derive(Debug)]
enum BackgroundJobEvent {
    Finished { key: String, error: Option<String> },
}

/// Shared state for the WSDD egui application.
pub struct WsddApp {
    pub settings: AppSettings,
    pub ui: UiState,
    pub first_run: bool,
    pub job_runtime: JobRuntime,
    pub requirement_rx: Option<mpsc::Receiver<LogLine>>,
    pub loader_outcome_rx: Option<mpsc::Receiver<LoaderOutcome>>,
    pub loader_done: bool,
    pub loader_error: bool,
    pub loader_silent: bool,
    pub requirements_started: bool,
    pub runner: PsRunner,
    pub containers: Vec<ContainerInfo>,
    pub projects: Vec<Project>,
    pub main_log: Vec<LogLine>,
    pub operation_traces: Vec<OperationTrace>,
    pub trace_tx: mpsc::Sender<TraceEvent>,
    trace_rx: mpsc::Receiver<TraceEvent>,
    pub main_log_tx: mpsc::Sender<LogLine>,
    main_log_rx: mpsc::Receiver<LogLine>,
    pub container_logs: Vec<ContainerLogEntry>,
    container_log_tx: mpsc::Sender<ContainerLogEntry>,
    container_log_rx: mpsc::Receiver<ContainerLogEntry>,
    container_log_done_tx: mpsc::Sender<String>,
    container_log_done_rx: mpsc::Receiver<String>,
    pub ngrok_publications: HashMap<String, NgrokPublication>,
    pub ngrok_publication_sync_attempted: HashSet<String>,
    pub ngrok_publication_tx: mpsc::Sender<NgrokPublicationEvent>,
    ngrok_publication_rx: mpsc::Receiver<NgrokPublicationEvent>,
    container_log_watchers: HashMap<String, ContainerLogWatcher>,
    container_log_retry_after: HashMap<String, Instant>,
    pub container_poll_rx: Option<mpsc::Receiver<ContainerPollSnapshot>>,
    pub container_poll_active: bool,
    pub last_container_poll: Instant,
    pub docker_status: DockerDesktopStatus,
    job_event_tx: mpsc::Sender<BackgroundJobEvent>,
    job_event_rx: mpsc::Receiver<BackgroundJobEvent>,
    pub jobs: HashMap<String, BackgroundJob>,
    main_window: Option<crate::handlers::window_state::MainWindowHandle>,
    last_main_window_save: Instant,
}

impl WsddApp {
    /// Creates a new WSDD app instance and initializes UI/runtime state.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        settings: AppSettings,
        job_runtime: JobRuntime,
    ) -> Self {
        fonts::setup_fonts(&cc.egui_ctx);
        let main_window = match crate::handlers::window_state::capture_main_window(cc) {
            Ok(window) => {
                if let Err(error) = crate::handlers::window_state::initialize_main_window(window) {
                    tracing::warn!("No se pudo restaurar la ventana principal: {error}");
                }
                Some(window)
            }
            Err(error) => {
                tracing::warn!("No se pudo capturar la ventana principal: {error}");
                None
            }
        };

        crate::i18n::set_language(settings.language);
        let first_run = !settings.setup_completed;
        let initial_view = if first_run {
            ActiveView::Welcome
        } else {
            ActiveView::Loader
        };

        let (main_log_tx, main_log_rx) = mpsc::channel::<LogLine>();
        let (trace_tx, trace_rx) = mpsc::channel::<TraceEvent>();
        let (container_log_tx, container_log_rx) = mpsc::channel::<ContainerLogEntry>();
        let (container_log_done_tx, container_log_done_rx) = mpsc::channel::<String>();
        let (ngrok_publication_tx, ngrok_publication_rx) = mpsc::channel::<NgrokPublicationEvent>();
        let (job_event_tx, job_event_rx) = mpsc::channel::<BackgroundJobEvent>();
        crate::handlers::diagnostics::register_settings(&settings);

        Self {
            first_run,
            ui: UiState::new(initial_view),
            settings,
            job_runtime,
            requirement_rx: None,
            loader_outcome_rx: None,
            loader_done: false,
            loader_error: false,
            loader_silent: !first_run,
            requirements_started: false,
            runner: PsRunner::new(),
            containers: Vec::new(),
            projects: crate::handlers::project::list_all().unwrap_or_default(),
            main_log: Vec::new(),
            operation_traces: Vec::new(),
            trace_tx,
            trace_rx,
            main_log_tx,
            main_log_rx,
            container_logs: Vec::new(),
            container_log_tx,
            container_log_rx,
            container_log_done_tx,
            container_log_done_rx,
            ngrok_publications: HashMap::new(),
            ngrok_publication_sync_attempted: HashSet::new(),
            ngrok_publication_tx,
            ngrok_publication_rx,
            container_log_watchers: HashMap::new(),
            container_log_retry_after: HashMap::new(),
            container_poll_rx: None,
            container_poll_active: false,
            last_container_poll: Instant::now()
                .checked_sub(std::time::Duration::from_secs(10))
                .unwrap_or_else(Instant::now),
            docker_status: DockerDesktopStatus::default(),
            job_event_tx,
            job_event_rx,
            jobs: HashMap::new(),
            main_window,
            last_main_window_save: Instant::now(),
        }
    }

    pub fn track_operation(&mut self, key: String, label: String, log_path: PathBuf) {
        self.ui.selected_trace_key = Some(key.clone());
        self.ui.show_main_log_detail = false;
        let label = format!("{label} · #{}", self.operation_traces.len() + 1);
        self.operation_traces.push(OperationTrace {
            key,
            label,
            log_path,
            stages: Vec::new(),
            buildkit_step: None,
            finished_at: None,
            success: false,
        });
    }

    fn save_main_window_state(&mut self) {
        let Some(main_window) = self.main_window else {
            return;
        };

        if let Err(error) = crate::handlers::window_state::save_main_window(main_window) {
            tracing::warn!("No se pudo guardar el estado de ventana principal: {error}");
        }
        self.last_main_window_save = Instant::now();
    }
}

impl eframe::App for WsddApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        crate::ui::theme::apply(ctx, self.settings.theme);
        if ctx.input(|input| input.viewport().close_requested()) {
            self.save_main_window_state();
        }
        if self.last_main_window_save.elapsed() >= Duration::from_millis(750) {
            self.save_main_window_state();
        }
        ctx.request_repaint_after(Duration::from_millis(750));
        self.drain_channels();
        self.sync_container_log_watchers(ctx);
        crate::ui::render(ctx, self);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_main_window_state();
    }
}
