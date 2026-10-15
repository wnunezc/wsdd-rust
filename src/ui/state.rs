use std::collections::BTreeSet;
use std::sync::mpsc;

use crate::handlers::log_types::LogLine;
use crate::handlers::setting::{AppSettings, PrereqCredentials};
use crate::handlers::wsl::WslConfig;
use crate::models::project::{EntryPoint, PhpVersion, Project, StorageMode};

/// Active application view.
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

/// Active tab inside the main dashboard.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum MainTab {
    #[default]
    Containers,
    Projects,
}

/// State for the prerequisite credential modal.
pub struct PrereqCredentialsPromptState {
    pub draft: PrereqCredentials,
    pub error: Option<String>,
}

/// State for the Webmin credential modal.
pub struct WebminCredentialsPromptState {
    pub project: Project,
    pub add_project_to_list: bool,
    pub username: String,
    pub password: String,
    pub error: Option<String>,
}

/// Targets shown in the removal confirmation dialog.
#[derive(Clone)]
pub struct RemovalRequest {
    pub project_names: Vec<String>,
    pub php_container: Option<String>,
}

/// Shared immediate-mode UI state.
pub struct UiState {
    pub active: ActiveView,
    pub requirement_log: Vec<LogLine>,
    pub readme_checked: bool,
    pub welcome_error: Option<String>,
    pub prereq_prompt: Option<PrereqCredentialsPromptState>,
    pub webmin_prompt: Option<WebminCredentialsPromptState>,
    pub deploy_php_prompt: Option<crate::ui::deploy_php::DeployPhpPrompt>,
    pub md_cache: egui_commonmark::CommonMarkCache,
    pub active_main_tab: MainTab,
    pub confirm_remove_projects: Option<RemovalRequest>,
    pub selected_projects: BTreeSet<String>,
    pub project_table: crate::ui::projects_panel::view::ProjectTable,
    pub toolbox_project_name: Option<String>,
    pub toolbox_container_name: Option<String>,
    pub form_name: String,
    pub form_domain: String,
    pub form_php: PhpVersion,
    pub form_work_path: String,
    pub form_entry: EntryPoint,
    pub form_entry_custom: String,
    pub form_ssl: bool,
    pub form_storage_mode: StorageMode,
    pub form_error: Option<String>,
    pub folder_pick_rx: Option<mpsc::Receiver<Option<String>>>,
    pub settings_draft: Option<AppSettings>,
    pub settings_error: Option<String>,
    pub settings_recovery_task_status: Option<String>,
    pub wsl_draft: Option<WslConfig>,
    pub helps_search: String,
}

impl UiState {
    /// Creates UI state for the initial active view.
    pub fn new(initial: ActiveView) -> Self {
        Self {
            active: initial,
            requirement_log: Vec::new(),
            readme_checked: false,
            welcome_error: None,
            prereq_prompt: None,
            webmin_prompt: None,
            deploy_php_prompt: None,
            md_cache: egui_commonmark::CommonMarkCache::default(),
            active_main_tab: MainTab::default(),
            confirm_remove_projects: None,
            selected_projects: BTreeSet::new(),
            project_table: Default::default(),
            toolbox_project_name: None,
            toolbox_container_name: None,
            form_name: String::new(),
            form_domain: String::new(),
            form_php: PhpVersion::default(),
            form_work_path: String::new(),
            form_entry: EntryPoint::default(),
            form_entry_custom: String::new(),
            form_ssl: true,
            form_storage_mode: StorageMode::default(),
            form_error: None,
            folder_pick_rx: None,
            settings_draft: None,
            settings_error: None,
            settings_recovery_task_status: None,
            wsl_draft: None,
            helps_search: String::new(),
        }
    }

    /// Resets the Add Project form fields.
    pub fn reset_add_project_form(&mut self) {
        self.form_name = String::new();
        self.form_domain = String::new();
        self.form_php = PhpVersion::default();
        self.form_work_path = String::new();
        self.form_entry = EntryPoint::default();
        self.form_entry_custom = String::new();
        self.form_ssl = true;
        self.form_storage_mode = StorageMode::default();
        self.form_error = None;
        self.folder_pick_rx = None;
    }

    /// Opens the prerequisite credential prompt with the current values.
    pub fn open_prereq_prompt(&mut self, current: &PrereqCredentials) {
        self.prereq_prompt = Some(PrereqCredentialsPromptState {
            draft: current.clone(),
            error: None,
        });
    }

    /// Opens the Webmin credential prompt for a project.
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
