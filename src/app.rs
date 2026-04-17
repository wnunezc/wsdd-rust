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

use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Instant;

use tokio::runtime::Runtime;

use crate::handlers::docker::{ContainerInfo, ContainerPollSnapshot, DockerDesktopStatus};
use crate::handlers::log_types::LogLine;
use crate::handlers::ps_script::PsRunner;
use crate::handlers::requirements::LoaderOutcome;
use crate::handlers::setting::AppSettings;
use crate::models::project::Project;
use crate::ui::{ActiveView, UiState};

mod channels;
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
    pub main_log_tx: mpsc::Sender<LogLine>,
    main_log_rx: mpsc::Receiver<LogLine>,
    pub container_poll_rx: Option<mpsc::Receiver<ContainerPollSnapshot>>,
    pub container_poll_active: bool,
    pub last_container_poll: Instant,
    pub docker_status: DockerDesktopStatus,
    job_event_tx: mpsc::Sender<BackgroundJobEvent>,
    job_event_rx: mpsc::Receiver<BackgroundJobEvent>,
    pub jobs: HashMap<String, BackgroundJob>,
}

impl WsddApp {
    /// Creates a new WSDD app instance and initializes UI/runtime state.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        settings: AppSettings,
        job_runtime: JobRuntime,
    ) -> Self {
        fonts::setup_fonts(&cc.egui_ctx);

        crate::i18n::set_language(settings.language);
        let first_run = !settings.setup_completed;
        let initial_view = if first_run {
            ActiveView::Welcome
        } else {
            ActiveView::Loader
        };

        let (main_log_tx, main_log_rx) = mpsc::channel::<LogLine>();
        let (job_event_tx, job_event_rx) = mpsc::channel::<BackgroundJobEvent>();

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
            main_log_tx,
            main_log_rx,
            container_poll_rx: None,
            container_poll_active: false,
            last_container_poll: Instant::now()
                .checked_sub(std::time::Duration::from_secs(10))
                .unwrap_or_else(Instant::now),
            docker_status: DockerDesktopStatus::default(),
            job_event_tx,
            job_event_rx,
            jobs: HashMap::new(),
        }
    }
}

impl eframe::App for WsddApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        crate::ui::theme::apply(ctx, self.settings.theme);
        self.drain_channels();
        crate::ui::render(ctx, self);
    }
}
