use crate::app::WsddApp;
use crate::handlers::setting::WebminCredentials;
use crate::i18n::{tr, trf};
use crate::ui::ActiveView;

/// Renders global credential prompts that can appear above any view.
pub(super) fn render(ctx: &egui::Context, app: &mut WsddApp) {
    render_prereq_credentials_dialog(ctx, app);
    render_webmin_credentials_dialog(ctx, app);
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

    super::modal::render_modal_backdrop(ctx, "prereq_credentials_backdrop");

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            if let Some(prompt) = app.ui.prereq_prompt.as_mut() {
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
                    ui.label(
                        egui::RichText::new(error).color(egui::Color32::from_rgb(220, 80, 80)),
                    );
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
            }
        });

    if save {
        save_prereq_credentials(app);
    } else if cancel {
        app.ui.prereq_prompt = None;
        if app.ui.active == ActiveView::Loader && !app.settings.setup_completed {
            app.ui.active = ActiveView::Welcome;
        }
    }
}

fn save_prereq_credentials(app: &mut WsddApp) {
    let Some(mut prompt) = app.ui.prereq_prompt.take() else {
        return;
    };

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

    if let Err(e) = crate::handlers::docker_deploy::sync_prerequisite_compose_sync(&app.settings) {
        prompt.error = Some(format!("Error preparando init.yml: {e}"));
        app.ui.prereq_prompt = Some(prompt);
    }
}

fn render_webmin_credentials_dialog(ctx: &egui::Context, app: &mut WsddApp) {
    if app.ui.webmin_prompt.is_none() {
        return;
    }

    let Some(prompt_ref) = app.ui.webmin_prompt.as_ref() else {
        return;
    };
    let title = trf(
        "dialog_webmin_title",
        &[("php", prompt_ref.project.php_version.display_name())],
    );
    let note = tr("dialog_webmin_note");
    let user_label = format!("{}:", tr("settings_webmin_user"));
    let password_label = format!("{}:", tr("settings_webmin_password"));
    let cancel_label = tr("btn_cancel");
    let save_label = tr("btn_save");

    let mut save = false;
    let mut cancel = false;

    super::modal::render_modal_backdrop(ctx, "webmin_credentials_backdrop");

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            if let Some(prompt) = app.ui.webmin_prompt.as_mut() {
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
                    ui.label(
                        egui::RichText::new(error).color(egui::Color32::from_rgb(220, 80, 80)),
                    );
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
            }
        });

    if save {
        save_webmin_credentials(ctx, app);
    } else if cancel {
        app.ui.webmin_prompt = None;
    }
}

fn save_webmin_credentials(ctx: &egui::Context, app: &mut WsddApp) {
    let Some(mut prompt) = app.ui.webmin_prompt.take() else {
        return;
    };

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
