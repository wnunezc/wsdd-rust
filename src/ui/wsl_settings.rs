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
//! WSL2 settings modal renderer.

use crate::app::WsddApp;
use crate::config::environment::path_config;
use crate::handlers::managed_storage;
use crate::handlers::setting::AppSettings;
use crate::handlers::wsl::{self, MemoryReclaim, NetworkingMode, WslConfig};
use crate::i18n::tr;
use crate::ui::ActiveView;

mod copy;

/// Renders the WSL settings modal and persists `.wslconfig` changes.
pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    let copy = copy::wsl_copy(app.settings.language);

    if app.ui.wsl_draft.is_none() {
        app.ui.wsl_draft = Some(match wsl::read() {
            Ok(cfg) => {
                app.ui.wsl_initial = Some(cfg.clone());
                cfg
            }
            Err(e) => {
                app.ui.wsl_config_error = Some(format!("No se pudo leer .wslconfig: {e}"));
                WslConfig::default()
            }
        });
        app.ui.wsl_storage_draft = Some(app.settings.clone());
        app.ui.wsl_recovery_status = Some(recovery_task_status(app));
    }

    let mut save = false;
    let mut save_storage = false;
    let mut cancel = false;
    let mut open = true;
    let cpu_max = std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(16);

    crate::ui::render_modal_backdrop(ctx, "wsl_settings_backdrop");

    egui::Window::new(tr("wsl_title"))
        .collapsible(false)
        .resizable(true)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(820.0)
        .default_height(660.0)
        .min_width(720.0)
        .min_height(520.0)
        .open(&mut open)
        .show(ctx, |ui| {
            let storage_error = app.ui.wsl_storage_error.clone();
            let storage_notice = app.ui.wsl_storage_notice.clone();
            let recovery_status = app.ui.wsl_recovery_status.clone();
            let config_error = app.ui.wsl_config_error.clone();
            let Some(draft) = app.ui.wsl_draft.as_mut() else {
                return;
            };

            render_header(ui, &mut save, &mut cancel, app.ui.wsl_initial.is_some());
            if let Some(error) = &config_error {
                ui.colored_label(ui.visuals().error_fg_color, error);
            }
            render_config_path(ui, &copy);
            ui.separator();
            ui.add_space(6.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(storage_draft) = app.ui.wsl_storage_draft.as_mut() {
                    render_storage_section(
                        ui,
                        storage_draft,
                        recovery_status.as_deref(),
                        storage_error.as_deref(),
                        storage_notice.as_deref(),
                        &mut save_storage,
                    );
                    ui.add_space(8.0);
                }
                render_resource_section(ui, draft, &copy, cpu_max);
                ui.add_space(8.0);
                render_performance_section(ui, draft, &copy, app.settings.language);
                ui.add_space(8.0);
                render_network_section(ui, draft, &copy, app.settings.language);
                ui.add_space(8.0);
                render_restart_note(ui, &copy);
                ui.add_space(16.0);
            });
        });

    if !open && !save {
        cancel = true;
    }

    if save_storage {
        save_storage_settings(app);
    } else if save {
        if let (Some(draft), Some(initial)) = (&app.ui.wsl_draft, &app.ui.wsl_initial) {
            if draft != initial {
                if let Err(error) = wsl::write(draft) {
                    app.ui.wsl_config_error =
                        Some(format!("Error escribiendo .wslconfig: {error}"));
                    return;
                }
            }
        }
        close(app);
    } else if cancel {
        close(app);
    }
}

fn render_header(ui: &mut egui::Ui, save: &mut bool, cancel: &mut bool, can_save: bool) {
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(format!("  {}  ", tr("btn_cancel"))).clicked() {
                *cancel = true;
            }
            ui.add_space(4.0);
            if ui
                .add_enabled(
                    can_save,
                    egui::Button::new(format!("  {}  ", tr("wsl_save_config")))
                        .fill(egui::Color32::from_rgb(34, 139, 34)),
                )
                .clicked()
            {
                *save = true;
            }
        });
    });
}

fn render_storage_section(
    ui: &mut egui::Ui,
    draft: &mut AppSettings,
    recovery_status: Option<&str>,
    error: Option<&str>,
    notice: Option<&str>,
    save: &mut bool,
) {
    egui::CollapsingHeader::new(tr("wsl_storage_section"))
        .default_open(true)
        .show(ui, |ui| {
            ui.label(tr("wsl_storage_save_note"));
            if let Some(error) = error {
                ui.colored_label(ui.visuals().error_fg_color, error);
            }
            if let Some(notice) = notice {
                ui.label(notice);
            }
            egui::Grid::new("wsl_storage_settings")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    ui.label(tr("settings_wsl_distro"));
                    let distro = draft.wsl_distro.get_or_insert_with(String::new);
                    ui.add(egui::TextEdit::singleline(distro).desired_width(210.0));
                    ui.end_row();

                    ui.label(tr("settings_managed_storage_enabled"));
                    ui.checkbox(&mut draft.managed_native_linux_storage_enabled, "");
                    ui.end_row();

                    ui.label(tr("wsl_projects_alias_root"));
                    ui.add(
                        egui::TextEdit::singleline(&mut draft.projects_path)
                            .desired_width(300.0)
                            .hint_text(path_config().default_projects_root().display().to_string()),
                    );
                    ui.end_row();

                    ui.label(tr("settings_recovery_task"));
                    ui.label(recovery_status.unwrap_or("-"));
                    ui.end_row();
                });
            *save = ui.button(tr("wsl_save_storage")).clicked();
        });
}

fn save_storage_settings(app: &mut WsddApp) {
    app.ui.wsl_storage_error = None;
    app.ui.wsl_storage_notice = None;
    let Some(draft) = app.ui.wsl_storage_draft.as_ref() else {
        return;
    };
    let next = updated_storage_settings(&app.settings, draft);

    let task_change = next.managed_native_linux_storage_enabled
        != app.settings.managed_native_linux_storage_enabled
        || next.wsl_distro != app.settings.wsl_distro;
    let storage_change = task_change || next.projects_path != app.settings.projects_path;
    if !storage_change && !app.ui.wsl_storage_task_retry {
        app.ui.wsl_storage_error = None;
        app.ui.wsl_storage_notice = Some(tr("wsl_storage_no_changes"));
        return;
    }

    if next.managed_native_linux_storage_enabled
        && !app.settings.managed_native_linux_storage_enabled
    {
        if let Err(error) =
            managed_storage::provision_managed_storage(&app.runner, &app.main_log_tx)
        {
            app.ui.wsl_storage_error = Some(error.to_string());
            return;
        }
    }
    if task_change && next.managed_native_linux_storage_enabled {
        if let Err(error) = crate::handlers::scheduled_task::reconcile(&app.runner, &next) {
            app.ui.wsl_storage_error = Some(error.to_string());
            return;
        }
    }
    if let Err(error) = next.save_public_config() {
        app.ui.wsl_storage_error = Some(error.to_string());
        return;
    }
    app.settings = next.clone();
    app.ui.wsl_storage_draft = Some(next);
    app.ui.wsl_storage_error = None;
    app.ui.wsl_storage_notice = Some(tr("wsl_storage_saved"));

    if (task_change || app.ui.wsl_storage_task_retry)
        && !app.settings.managed_native_linux_storage_enabled
    {
        if let Err(error) = crate::handlers::scheduled_task::reconcile(&app.runner, &app.settings) {
            app.ui.wsl_storage_error = Some(format!("{}: {error}", tr("wsl_storage_task_pending")));
            app.ui.wsl_storage_task_retry = true;
        } else {
            app.ui.wsl_storage_task_retry = false;
        }
    }
    app.ui.wsl_recovery_status = Some(recovery_task_status(app));
}

fn updated_storage_settings(current: &AppSettings, draft: &AppSettings) -> AppSettings {
    let mut next = current.clone();
    next.projects_path = draft.projects_path.trim().to_string();
    if next.projects_path.is_empty() {
        next.projects_path = path_config().default_projects_root().display().to_string();
    }
    next.wsl_distro = draft
        .wsl_distro
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    next.managed_native_linux_storage_enabled = draft.managed_native_linux_storage_enabled;
    next.optional_services.managed_native_linux_storage.enabled =
        next.managed_native_linux_storage_enabled;
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wsl_storage_edit_preserves_other_preferences_and_credentials() {
        let mut current = AppSettings::default();
        current.ngrok_authtoken = "saved-token".into();
        current.optional_services.redis.host_port = 6380;
        current.php_timezone = "America/Panama".into();
        let mut draft = current.clone();
        draft.projects_path = "  D:\\Projects  ".into();
        draft.wsl_distro = Some("  Ubuntu  ".into());
        draft.managed_native_linux_storage_enabled = true;
        draft.ngrok_authtoken = "unrelated-edit".into();

        let next = updated_storage_settings(&current, &draft);
        assert_eq!(next.projects_path, "D:\\Projects");
        assert_eq!(next.wsl_distro.as_deref(), Some("Ubuntu"));
        assert!(next.managed_native_linux_storage_enabled);
        assert!(next.optional_services.managed_native_linux_storage.enabled);
        assert_eq!(next.ngrok_authtoken, "saved-token");
        assert_eq!(next.optional_services.redis.host_port, 6380);
        assert_eq!(next.php_timezone, "America/Panama");
    }
}

fn recovery_task_status(app: &WsddApp) -> String {
    let projects = match crate::handlers::project::list_all() {
        Ok(projects) => projects,
        Err(error) => return format!("{}: {error}", tr("status_error")),
    };
    let required = crate::handlers::scheduled_task::recovery_task_decision(
        app.settings.managed_native_linux_storage_enabled,
        &projects,
        app.settings.wsl_distro.as_deref(),
    ) != crate::handlers::scheduled_task::RecoveryTaskDecision::Remove;
    match crate::handlers::scheduled_task::status(&app.runner, required) {
        Ok(crate::handlers::scheduled_task::RecoveryTaskStatus::Installed) => {
            tr("settings_recovery_task_installed")
        }
        Ok(crate::handlers::scheduled_task::RecoveryTaskStatus::Missing) => {
            tr("settings_recovery_task_missing")
        }
        Ok(crate::handlers::scheduled_task::RecoveryTaskStatus::NotRequired) => {
            tr("settings_recovery_task_not_required")
        }
        Ok(crate::handlers::scheduled_task::RecoveryTaskStatus::NeedsUpdate) => {
            tr("settings_recovery_task_needs_update")
        }
        Err(error) => format!("{}: {error}", tr("status_error")),
    }
}

fn close(app: &mut WsddApp) {
    app.ui.wsl_draft = None;
    app.ui.wsl_initial = None;
    app.ui.wsl_config_error = None;
    app.ui.wsl_storage_draft = None;
    app.ui.wsl_storage_error = None;
    app.ui.wsl_storage_notice = None;
    app.ui.wsl_storage_task_retry = false;
    app.ui.wsl_recovery_status = None;
    app.ui.active = ActiveView::Main;
}

fn render_config_path(ui: &mut egui::Ui, copy: &copy::WslCopy) {
    ui.label(
        egui::RichText::new(format!(
            "{} {}",
            copy.path_prefix,
            wsl::config_path_display()
        ))
        .size(11.0)
        .color(ui.visuals().weak_text_color()),
    );
}

fn render_resource_section(
    ui: &mut egui::Ui,
    draft: &mut WslConfig,
    copy: &copy::WslCopy,
    cpu_max: u32,
) {
    egui::CollapsingHeader::new(egui::RichText::new(copy.system_resources).strong())
        .default_open(true)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(copy.resource_note)
                    .size(11.0)
                    .color(ui.visuals().weak_text_color()),
            );
            ui.add_space(6.0);

            egui::Grid::new("wsl_resources")
                .num_columns(2)
                .spacing([12.0, 10.0])
                .min_col_width(180.0)
                .show(ui, |ui| {
                    render_cpu_row(ui, draft, copy, cpu_max);
                    render_memory_row(ui, draft, copy);
                    render_swap_row(ui, draft, copy);
                });
        });
}

fn render_cpu_row(ui: &mut egui::Ui, draft: &mut WslConfig, copy: &copy::WslCopy, cpu_max: u32) {
    ui.label(copy.cpu_cores);
    ui.horizontal(|ui| {
        let current_label = draft
            .processors
            .map(|n| n.to_string())
            .unwrap_or_else(|| copy.no_limit.to_string());

        egui::ComboBox::from_id_salt("wsl_cpu")
            .selected_text(&current_label)
            .width(120.0)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(draft.processors.is_none(), copy.no_limit)
                    .clicked()
                {
                    draft.processors = None;
                }
                let mut n = 1u32;
                while n <= cpu_max {
                    let selected = draft.processors == Some(n);
                    if ui.selectable_label(selected, n.to_string()).clicked() {
                        draft.processors = Some(n);
                    }
                    n = if n < 4 { n + 1 } else { n + (n / 2).max(2) };
                }
            });

        ui.label(
            egui::RichText::new(copy.detected_cpu(cpu_max))
                .size(11.0)
                .color(ui.visuals().weak_text_color()),
        );
    });
    ui.end_row();
}

fn render_memory_row(ui: &mut egui::Ui, draft: &mut WslConfig, copy: &copy::WslCopy) {
    ui.label(copy.max_memory);
    ui.horizontal(|ui| {
        let current_label = draft
            .memory_gb
            .map(|n| format!("{n} GB"))
            .unwrap_or_else(|| copy.no_limit.to_string());

        egui::ComboBox::from_id_salt("wsl_ram")
            .selected_text(&current_label)
            .width(120.0)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(draft.memory_gb.is_none(), copy.no_limit)
                    .clicked()
                {
                    draft.memory_gb = None;
                }
                for &gb in &[1u32, 2, 4, 6, 8, 12, 16, 24, 32, 48, 64] {
                    let selected = draft.memory_gb == Some(gb);
                    if ui.selectable_label(selected, format!("{gb} GB")).clicked() {
                        draft.memory_gb = Some(gb);
                    }
                }
            });

        ui.label(
            egui::RichText::new(copy.ram_recommendation)
                .size(11.0)
                .color(ui.visuals().weak_text_color()),
        );
    });
    ui.end_row();
}

fn render_swap_row(ui: &mut egui::Ui, draft: &mut WslConfig, copy: &copy::WslCopy) {
    ui.label(tr("wsl_swap"));
    ui.horizontal(|ui| {
        let current_label = if draft.swap_gb == 0 {
            copy.swap_disabled.to_string()
        } else {
            format!("{} GB", draft.swap_gb)
        };

        egui::ComboBox::from_id_salt("wsl_swap")
            .selected_text(&current_label)
            .width(120.0)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(draft.swap_gb == 0, copy.swap_disabled)
                    .clicked()
                {
                    draft.swap_gb = 0;
                }
                for &gb in &[1u32, 2, 4, 8, 16] {
                    let selected = draft.swap_gb == gb;
                    if ui.selectable_label(selected, format!("{gb} GB")).clicked() {
                        draft.swap_gb = gb;
                    }
                }
            });

        ui.label(
            egui::RichText::new(copy.swap_recommendation)
                .size(11.0)
                .color(ui.visuals().weak_text_color()),
        );
    });
    ui.end_row();
}

fn render_performance_section(
    ui: &mut egui::Ui,
    draft: &mut WslConfig,
    copy: &copy::WslCopy,
    language: crate::i18n::Language,
) {
    egui::CollapsingHeader::new(egui::RichText::new(copy.performance_memory).strong())
        .default_open(true)
        .show(ui, |ui| {
            egui::Grid::new("wsl_perf")
                .num_columns(2)
                .spacing([12.0, 10.0])
                .min_col_width(180.0)
                .show(ui, |ui| {
                    ui.label(copy.memory_reclaim);
                    egui::ComboBox::from_id_salt("wsl_reclaim")
                        .selected_text(copy::memory_reclaim_label(&draft.memory_reclaim, language))
                        .width(220.0)
                        .show_ui(ui, |ui| {
                            for mode in MemoryReclaim::all() {
                                let selected = &draft.memory_reclaim == mode;
                                if ui
                                    .selectable_label(
                                        selected,
                                        copy::memory_reclaim_label(mode, language),
                                    )
                                    .clicked()
                                {
                                    draft.memory_reclaim = mode.clone();
                                }
                            }
                        });
                    ui.end_row();

                    ui.label(copy.gui_apps);
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut draft.gui_applications, "");
                        ui.label(
                            egui::RichText::new(copy.gui_apps_hint)
                                .size(11.0)
                                .color(ui.visuals().weak_text_color()),
                        );
                    });
                    ui.end_row();
                });
        });
}

fn render_network_section(
    ui: &mut egui::Ui,
    draft: &mut WslConfig,
    copy: &copy::WslCopy,
    language: crate::i18n::Language,
) {
    egui::CollapsingHeader::new(egui::RichText::new(copy.network).strong())
        .default_open(true)
        .show(ui, |ui| {
            egui::Grid::new("wsl_network")
                .num_columns(2)
                .spacing([12.0, 10.0])
                .min_col_width(180.0)
                .show(ui, |ui| {
                    ui.label(copy.localhost_forwarding);
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut draft.localhost_forwarding, "");
                        ui.label(
                            egui::RichText::new(copy.localhost_hint)
                                .size(11.0)
                                .color(ui.visuals().weak_text_color()),
                        );
                    });
                    ui.end_row();

                    ui.label(copy.network_mode);
                    egui::ComboBox::from_id_salt("wsl_netmode")
                        .selected_text(copy::networking_mode_label(
                            &draft.networking_mode,
                            language,
                        ))
                        .width(260.0)
                        .show_ui(ui, |ui| {
                            for mode in NetworkingMode::all() {
                                let selected = &draft.networking_mode == mode;
                                if ui
                                    .selectable_label(
                                        selected,
                                        copy::networking_mode_label(mode, language),
                                    )
                                    .clicked()
                                {
                                    draft.networking_mode = mode.clone();
                                }
                            }
                        });
                    ui.end_row();

                    if draft.networking_mode == NetworkingMode::Mirrored {
                        ui.label(copy.dns_tunneling);
                        ui.checkbox(&mut draft.dns_tunneling, "");
                        ui.end_row();

                        ui.label(copy.windows_firewall);
                        ui.checkbox(&mut draft.firewall, "");
                        ui.end_row();
                    }
                });
        });
}

fn render_restart_note(ui: &mut egui::Ui, copy: &copy::WslCopy) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(80, 60, 10))
        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
        .rounding(egui::Rounding::same(4.0))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(copy.restart_note)
                    .size(12.0)
                    .color(egui::Color32::from_rgb(255, 210, 80)),
            );
        });
}
