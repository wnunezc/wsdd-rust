use crate::app::WsddApp;
use crate::handlers::log_types::LogLine;
use crate::handlers::setting::WebminCredentials;
use crate::i18n::tr;
use crate::models::project::PhpVersion;

pub(crate) struct DeployPhpPrompt {
    version: PhpVersion,
    username: String,
    password: String,
    error: Option<String>,
}

impl Default for DeployPhpPrompt {
    fn default() -> Self {
        Self {
            version: PhpVersion::default(),
            username: "admin".into(),
            password: String::new(),
            error: None,
        }
    }
}

pub(super) fn render(ctx: &egui::Context, app: &mut WsddApp) {
    let Some(mut prompt) = app.ui.deploy_php_prompt.take() else {
        return;
    };
    let mut open = true;
    let mut deploy = false;
    let mut cancel = false;
    let busy = super::projects_panel::project_operations_busy(app);
    super::render_modal_backdrop(ctx, "deploy_php_backdrop");
    egui::Window::new(tr("toolbar_deploy_php"))
        .id(egui::Id::new("deploy_php_dialog"))
        .collapsible(false)
        .resizable(false)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut open)
        .show(ctx, |ui| {
            let before = prompt.version.clone();
            ui.horizontal(|ui| {
                ui.label(tr("col_php"));
                egui::ComboBox::from_id_salt("deploy_php_version")
                    .width(160.0)
                    .selected_text(prompt.version.display_name())
                    .show_ui(ui, |ui| {
                        for version in PhpVersion::all() {
                            let label = version.display_name();
                            ui.selectable_value(&mut prompt.version, version, label);
                        }
                    });
            });
            if before != prompt.version {
                prompt.password.clear();
                prompt.error = None;
            }
            let name = format!("WSDD-Web-Server-{}", prompt.version.runtime_container_tag());
            let exists = app
                .containers
                .iter()
                .any(|container| container.name == name);
            if exists {
                ui.label(tr("deploy_php_exists"));
            } else if app
                .settings
                .webmin_credentials_for(&prompt.version)
                .is_none()
            {
                ui.add_space(8.0);
                egui::Grid::new("deploy_php_credentials").show(ui, |ui| {
                    ui.label(tr("settings_webmin_user"));
                    ui.add(egui::TextEdit::singleline(&mut prompt.username).desired_width(180.0));
                    ui.end_row();
                    ui.label(tr("settings_webmin_password"));
                    ui.add(
                        egui::TextEdit::singleline(&mut prompt.password)
                            .password(true)
                            .desired_width(180.0),
                    );
                    ui.end_row();
                });
            }
            if let Some(error) = &prompt.error {
                ui.colored_label(ui.visuals().error_fg_color, error);
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                deploy = ui
                    .add_enabled(!busy && !exists, egui::Button::new(tr("col_deploy")))
                    .clicked();
                cancel = ui.button(tr("btn_cancel")).clicked();
            });
        });
    if !open || cancel {
        return;
    }
    if deploy {
        if let Err(error) = start(ctx, app, &prompt) {
            prompt.error = Some(error);
        } else {
            return;
        }
    }
    app.ui.deploy_php_prompt = Some(prompt);
}

fn start(ctx: &egui::Context, app: &mut WsddApp, prompt: &DeployPhpPrompt) -> Result<(), String> {
    if super::projects_panel::project_operations_busy(app) {
        return Err(tr("deploy_php_busy"));
    }
    if app
        .settings
        .webmin_credentials_for(&prompt.version)
        .is_none()
    {
        let mut settings = app.settings.clone();
        settings
            .store_webmin_credentials(WebminCredentials {
                php_version: prompt.version.clone(),
                username: prompt.username.clone(),
                password: prompt.password.clone(),
            })
            .map_err(|e| e.to_string())?;
        settings.save().map_err(|e| e.to_string())?;
        app.settings = settings;
    }
    let version = prompt.version.clone();
    let runner = app.runner.clone();
    let settings = app.settings.clone();
    let tx = app.main_log_tx.clone();
    let key = format!("container:deploy:{}", version.compose_tag());
    if !app.spawn_blocking_job(ctx, key, "Deploy PHP", move || {
        crate::handlers::deploy::deploy_php_version(&version, &settings, &runner, &tx).map_err(
            |e| {
                let message = format!("[Deploy PHP] Error: {e}");
                let _ = tx.send(LogLine::error(&message));
                message
            },
        )
    }) {
        return Err(tr("deploy_php_busy"));
    }
    Ok(())
}
