use crate::config::environment::{path_config, path_to_string, DEFAULT_DOCKER_DESKTOP_EXE};
use crate::handlers::setting::{AppSettings, WebminCredentials};
use crate::i18n::{tr, Language};
use crate::models::project::PhpVersion;

/// Labels and copy used by the Settings modal.
pub(super) struct SettingsCopy {
    pub title: String,
    cancel_label: String,
    save_label: String,
    general_title: String,
    php_title: String,
    tools_title: String,
    projects_path_label: String,
    language_label: String,
    docker_path_label: String,
    wsl_distro_label: String,
    log_lines_label: String,
    auto_start_label: String,
    php_note: String,
    php_memory_label: String,
    php_upload_label: String,
    php_timezone_label: String,
    xdebug_enabled_label: String,
    xdebug_note: String,
    optional_services_title: String,
    optional_services_note: String,
    redis_enabled_label: String,
    redis_auto_start_label: String,
    redis_port_label: String,
    memcached_enabled_label: String,
    memcached_auto_start_label: String,
    memcached_port_label: String,
    memcached_memory_label: String,
    mailpit_enabled_label: String,
    mailpit_auto_start_label: String,
    mailpit_smtp_port_label: String,
    mailpit_ui_port_label: String,
    mailpit_virtual_host_label: String,
    prereq_title: String,
    prereq_note: String,
    prereq_runtime_note: String,
    mysql_database_label: String,
    mysql_user_label: String,
    mysql_password_label: String,
    mysql_root_password_label: String,
    webmin_credentials_title: String,
    webmin_credentials_note: String,
    webmin_runtime_note: String,
    webmin_user_label: String,
    webmin_password_label: String,
    webmin_version_label: String,
}

impl SettingsCopy {
    /// Builds translated labels for the current language.
    pub(super) fn new() -> Self {
        Self {
            title: format!("⚙ {}", tr("settings_title")),
            cancel_label: format!("  {}  ", tr("btn_cancel")),
            save_label: format!("  {}  ", tr("btn_save")),
            general_title: tr("settings_general"),
            php_title: tr("settings_php"),
            tools_title: tr("settings_tools_section"),
            projects_path_label: format!("{}:", tr("settings_projects_path")),
            language_label: format!("{}:", tr("settings_language")),
            docker_path_label: format!("{}:", tr("settings_docker_path")),
            wsl_distro_label: format!("{}:", tr("settings_wsl_distro")),
            log_lines_label: format!("{}:", tr("settings_log_lines")),
            auto_start_label: format!("{}:", tr("settings_auto_start")),
            php_note: tr("settings_apply_new_php_note"),
            php_memory_label: format!("{}:", tr("settings_php_memory")),
            php_upload_label: format!("{}:", tr("settings_php_upload")),
            php_timezone_label: format!("{}:", tr("settings_php_timezone")),
            xdebug_enabled_label: format!("{}:", tr("settings_xdebug_enabled")),
            xdebug_note: tr("settings_xdebug_note"),
            optional_services_title: tr("settings_optional_services_section"),
            optional_services_note: tr("settings_optional_services_note"),
            redis_enabled_label: format!("{}:", tr("settings_redis_enabled")),
            redis_auto_start_label: format!("{}:", tr("settings_redis_auto_start")),
            redis_port_label: format!("{}:", tr("settings_redis_port")),
            memcached_enabled_label: format!("{}:", tr("settings_memcached_enabled")),
            memcached_auto_start_label: format!("{}:", tr("settings_memcached_auto_start")),
            memcached_port_label: format!("{}:", tr("settings_memcached_port")),
            memcached_memory_label: format!("{}:", tr("settings_memcached_memory")),
            mailpit_enabled_label: format!("{}:", tr("settings_mailpit_enabled")),
            mailpit_auto_start_label: format!("{}:", tr("settings_mailpit_auto_start")),
            mailpit_smtp_port_label: format!("{}:", tr("settings_mailpit_smtp_port")),
            mailpit_ui_port_label: format!("{}:", tr("settings_mailpit_ui_port")),
            mailpit_virtual_host_label: format!("{}:", tr("settings_mailpit_virtual_host")),
            prereq_title: tr("settings_prereq_section"),
            prereq_note: tr("settings_prereq_note"),
            prereq_runtime_note: tr("settings_prereq_runtime_note"),
            mysql_database_label: format!("{}:", tr("settings_mysql_database")),
            mysql_user_label: format!("{}:", tr("settings_mysql_user")),
            mysql_password_label: format!("{}:", tr("settings_mysql_password")),
            mysql_root_password_label: format!("{}:", tr("settings_mysql_root_password")),
            webmin_credentials_title: tr("settings_webmin_credentials_section"),
            webmin_credentials_note: tr("settings_webmin_credentials_note"),
            webmin_runtime_note: tr("settings_webmin_credentials_runtime_note"),
            webmin_user_label: tr("settings_webmin_user"),
            webmin_password_label: tr("settings_webmin_password"),
            webmin_version_label: format!("{}:", tr("settings_webmin_version")),
        }
    }
}

pub(super) fn render_header(
    ui: &mut egui::Ui,
    copy: &SettingsCopy,
    save: &mut bool,
    cancel: &mut bool,
) {
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(&copy.cancel_label).clicked() {
                *cancel = true;
            }
            ui.add_space(4.0);
            if ui
                .add(egui::Button::new(&copy.save_label).fill(egui::Color32::from_rgb(34, 139, 34)))
                .clicked()
            {
                *save = true;
            }
        });
    });
}

pub(super) fn render_error(ui: &mut egui::Ui, error: Option<&str>) {
    if let Some(error) = error {
        ui.label(egui::RichText::new(error).color(egui::Color32::from_rgb(220, 80, 80)));
        ui.add_space(6.0);
    }
}

pub(super) fn render_body(ui: &mut egui::Ui, draft: &mut AppSettings, copy: &SettingsCopy) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        render_general_section(ui, draft, copy);
        ui.add_space(8.0);
        render_php_section(ui, draft, copy);
        ui.add_space(8.0);
        render_optional_services_section(ui, draft, copy);
        ui.add_space(8.0);
        render_prereq_section(ui, draft, copy);
        ui.add_space(8.0);
        render_webmin_credentials_section(ui, draft, copy);
        ui.add_space(8.0);
        render_tools_section(ui, draft, copy);
        ui.add_space(16.0);
    });
}

fn render_general_section(ui: &mut egui::Ui, draft: &mut AppSettings, copy: &SettingsCopy) {
    egui::CollapsingHeader::new(egui::RichText::new(&copy.general_title).strong())
        .default_open(true)
        .show(ui, |ui| {
            egui::Grid::new("settings_general")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .min_col_width(180.0)
                .show(ui, |ui| {
                    ui.label(&copy.projects_path_label);
                    ui.add(
                        egui::TextEdit::singleline(&mut draft.projects_path)
                            .desired_width(300.0)
                            .hint_text(path_to_string(path_config().default_projects_root())),
                    );
                    ui.end_row();

                    ui.label(&copy.language_label);
                    egui::ComboBox::from_id_salt("settings_language")
                        .selected_text(draft.language.native_name())
                        .width(180.0)
                        .show_ui(ui, |ui| {
                            for &language in Language::all() {
                                ui.selectable_value(
                                    &mut draft.language,
                                    language,
                                    language.native_name(),
                                );
                            }
                        });
                    ui.end_row();

                    ui.label(&copy.docker_path_label);
                    let docker_val = draft.docker_path.get_or_insert_with(String::new);
                    ui.add(
                        egui::TextEdit::singleline(docker_val)
                            .desired_width(300.0)
                            .hint_text(format!("{DEFAULT_DOCKER_DESKTOP_EXE} (opcional)")),
                    );
                    ui.end_row();

                    ui.label(&copy.wsl_distro_label);
                    let distro_val = draft.wsl_distro.get_or_insert_with(String::new);
                    ui.add(
                        egui::TextEdit::singleline(distro_val)
                            .desired_width(200.0)
                            .hint_text("Ubuntu-22.04 (opcional)"),
                    );
                    ui.end_row();

                    ui.label(&copy.log_lines_label);
                    ui.add(
                        egui::DragValue::new(&mut draft.log_max_lines)
                            .range(100..=10000)
                            .speed(10.0),
                    );
                    ui.end_row();

                    ui.label(&copy.auto_start_label);
                    ui.checkbox(&mut draft.auto_start_containers, "");
                    ui.end_row();
                });
        });
}

fn render_php_section(ui: &mut egui::Ui, draft: &mut AppSettings, copy: &SettingsCopy) {
    egui::CollapsingHeader::new(egui::RichText::new(&copy.php_title).strong())
        .default_open(true)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(&copy.php_note)
                    .size(11.0)
                    .color(ui.visuals().weak_text_color()),
            );
            ui.add_space(6.0);

            egui::Grid::new("settings_php")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .min_col_width(180.0)
                .show(ui, |ui| {
                    ui.label(&copy.php_memory_label);
                    ui.add(
                        egui::TextEdit::singleline(&mut draft.php_memory_limit)
                            .desired_width(100.0)
                            .hint_text("512M"),
                    );
                    ui.end_row();

                    ui.label(&copy.php_upload_label);
                    ui.add(
                        egui::TextEdit::singleline(&mut draft.php_upload_max_filesize)
                            .desired_width(100.0)
                            .hint_text("256M"),
                    );
                    ui.end_row();

                    ui.label(&copy.php_timezone_label);
                    ui.add(
                        egui::TextEdit::singleline(&mut draft.php_timezone)
                            .desired_width(200.0)
                            .hint_text("UTC"),
                    );
                    ui.end_row();

                    ui.label(&copy.xdebug_enabled_label);
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut draft.xdebug_enabled, "");
                        ui.label(
                            egui::RichText::new(&copy.xdebug_note)
                                .size(11.0)
                                .color(ui.visuals().weak_text_color()),
                        );
                    });
                    ui.end_row();
                });
        });
}

fn render_optional_services_section(
    ui: &mut egui::Ui,
    draft: &mut AppSettings,
    copy: &SettingsCopy,
) {
    egui::CollapsingHeader::new(egui::RichText::new(&copy.optional_services_title).strong())
        .default_open(true)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(&copy.optional_services_note)
                    .size(11.0)
                    .color(ui.visuals().weak_text_color()),
            );
            ui.add_space(6.0);

            egui::Grid::new("settings_optional_services")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .min_col_width(180.0)
                .show(ui, |ui| {
                    ui.label(&copy.redis_enabled_label);
                    ui.checkbox(&mut draft.optional_services.redis.enabled, "");
                    ui.end_row();

                    if draft.optional_services.redis.enabled {
                        ui.label(&copy.redis_auto_start_label);
                        ui.checkbox(&mut draft.optional_services.redis.auto_start, "");
                        ui.end_row();

                        ui.label(&copy.redis_port_label);
                        ui.add(
                            egui::DragValue::new(&mut draft.optional_services.redis.host_port)
                                .range(1..=65535)
                                .speed(1.0),
                        );
                        ui.end_row();
                    }

                    ui.label(&copy.memcached_enabled_label);
                    ui.checkbox(&mut draft.optional_services.memcached.enabled, "");
                    ui.end_row();

                    if draft.optional_services.memcached.enabled {
                        ui.label(&copy.memcached_auto_start_label);
                        ui.checkbox(&mut draft.optional_services.memcached.auto_start, "");
                        ui.end_row();

                        ui.label(&copy.memcached_port_label);
                        ui.add(
                            egui::DragValue::new(&mut draft.optional_services.memcached.host_port)
                                .range(1..=65535)
                                .speed(1.0),
                        );
                        ui.end_row();

                        ui.label(&copy.memcached_memory_label);
                        ui.add(
                            egui::DragValue::new(
                                &mut draft.optional_services.memcached.memory_limit_mb,
                            )
                            .range(16..=4096)
                            .speed(16.0),
                        );
                        ui.end_row();
                    }

                    ui.label(&copy.mailpit_enabled_label);
                    ui.checkbox(&mut draft.optional_services.mailpit.enabled, "");
                    ui.end_row();

                    if draft.optional_services.mailpit.enabled {
                        ui.label(&copy.mailpit_auto_start_label);
                        ui.checkbox(&mut draft.optional_services.mailpit.auto_start, "");
                        ui.end_row();

                        ui.label(&copy.mailpit_smtp_port_label);
                        ui.add(
                            egui::DragValue::new(
                                &mut draft.optional_services.mailpit.smtp_host_port,
                            )
                            .range(1..=65535)
                            .speed(1.0),
                        );
                        ui.end_row();

                        ui.label(&copy.mailpit_ui_port_label);
                        ui.add(
                            egui::DragValue::new(&mut draft.optional_services.mailpit.ui_host_port)
                                .range(1..=65535)
                                .speed(1.0),
                        );
                        ui.end_row();

                        ui.label(&copy.mailpit_virtual_host_label);
                        ui.add(
                            egui::TextEdit::singleline(
                                &mut draft.optional_services.mailpit.virtual_host,
                            )
                            .desired_width(220.0)
                            .hint_text("mailpit.wsdd.dock"),
                        );
                        ui.end_row();
                    }
                });
        });
}

fn render_prereq_section(ui: &mut egui::Ui, draft: &mut AppSettings, copy: &SettingsCopy) {
    egui::CollapsingHeader::new(egui::RichText::new(&copy.prereq_title).strong())
        .default_open(true)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(&copy.prereq_note)
                    .size(11.0)
                    .color(ui.visuals().weak_text_color()),
            );
            ui.label(
                egui::RichText::new(&copy.prereq_runtime_note)
                    .size(11.0)
                    .color(ui.visuals().weak_text_color()),
            );
            ui.add_space(6.0);

            egui::Grid::new("settings_prereq")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .min_col_width(180.0)
                .show(ui, |ui| {
                    ui.label(&copy.mysql_database_label);
                    ui.add(
                        egui::TextEdit::singleline(&mut draft.prereq_credentials.mysql_database)
                            .desired_width(180.0)
                            .hint_text("wsdd-database"),
                    );
                    ui.end_row();

                    ui.label(&copy.mysql_user_label);
                    ui.add(
                        egui::TextEdit::singleline(&mut draft.prereq_credentials.mysql_user)
                            .desired_width(180.0)
                            .hint_text("tester"),
                    );
                    ui.end_row();

                    ui.label(&copy.mysql_password_label);
                    ui.add(
                        egui::TextEdit::singleline(&mut draft.prereq_credentials.mysql_password)
                            .desired_width(180.0)
                            .password(true)
                            .hint_text("required"),
                    );
                    ui.end_row();

                    ui.label(&copy.mysql_root_password_label);
                    ui.add(
                        egui::TextEdit::singleline(
                            &mut draft.prereq_credentials.mysql_root_password,
                        )
                        .desired_width(180.0)
                        .password(true)
                        .hint_text("required"),
                    );
                    ui.end_row();
                });
        });
}

fn render_webmin_credentials_section(
    ui: &mut egui::Ui,
    draft: &mut AppSettings,
    copy: &SettingsCopy,
) {
    egui::CollapsingHeader::new(egui::RichText::new(&copy.webmin_credentials_title).strong())
        .default_open(true)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(&copy.webmin_credentials_note)
                    .size(11.0)
                    .color(ui.visuals().weak_text_color()),
            );
            ui.label(
                egui::RichText::new(&copy.webmin_runtime_note)
                    .size(11.0)
                    .color(ui.visuals().weak_text_color()),
            );
            ui.add_space(6.0);

            egui::Grid::new("settings_webmin_credentials")
                .num_columns(3)
                .spacing([12.0, 8.0])
                .min_col_width(150.0)
                .show(ui, |ui| {
                    ui.strong(tr("col_php"));
                    ui.strong(&copy.webmin_user_label);
                    ui.strong(&copy.webmin_password_label);
                    ui.end_row();

                    for php_version in PhpVersion::all() {
                        let current = draft
                            .webmin_credentials_entry(&php_version)
                            .cloned()
                            .unwrap_or_else(|| WebminCredentials {
                                php_version: php_version.clone(),
                                username: String::new(),
                                password: String::new(),
                            });

                        let mut username = current.username;
                        let mut password = current.password;

                        ui.label(php_version.display_name());
                        let user_changed = ui
                            .add(
                                egui::TextEdit::singleline(&mut username)
                                    .desired_width(150.0)
                                    .hint_text("admin"),
                            )
                            .changed();
                        let password_changed = ui
                            .add(
                                egui::TextEdit::singleline(&mut password)
                                    .desired_width(150.0)
                                    .password(true)
                                    .hint_text("required"),
                            )
                            .changed();
                        ui.end_row();

                        if user_changed || password_changed {
                            draft.set_webmin_credentials_draft(php_version, username, password);
                        }
                    }
                });
        });
}

fn render_tools_section(ui: &mut egui::Ui, draft: &mut AppSettings, copy: &SettingsCopy) {
    egui::CollapsingHeader::new(egui::RichText::new(&copy.tools_title).strong())
        .default_open(true)
        .show(ui, |ui| {
            egui::Grid::new("settings_tools")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .min_col_width(180.0)
                .show(ui, |ui| {
                    ui.label(&copy.webmin_version_label);
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut draft.webmin_version)
                                .desired_width(100.0)
                                .hint_text("2.630"),
                        );
                        ui.label(
                            egui::RichText::new("(PHP Dockerfiles)")
                                .size(11.0)
                                .color(ui.visuals().weak_text_color()),
                        );
                    });
                    ui.end_row();
                });
        });
}
