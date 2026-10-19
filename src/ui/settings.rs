//! Global application preferences. Service and runtime settings have their own dialogs.

use crate::app::WsddApp;
use crate::i18n::{self, tr, Language};
use crate::ui::ActiveView;

/// Renders the language preference without changing Docker or WSL resources.
pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    if app.ui.settings_draft.is_none() {
        app.ui.settings_draft = Some(app.settings.clone());
        app.ui.settings_error = None;
    }

    let mut save = false;
    let mut cancel = false;
    let mut open = true;
    crate::ui::render_modal_backdrop(ctx, "settings_backdrop");
    egui::Window::new(tr("settings_title"))
        .collapsible(false)
        .resizable(false)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(350.0)
        .open(&mut open)
        .show(ctx, |ui| {
            if let Some(error) = &app.ui.settings_error {
                ui.colored_label(ui.visuals().error_fg_color, error);
            }
            if let Some(draft) = app.ui.settings_draft.as_mut() {
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", tr("settings_language")));
                    egui::ComboBox::from_id_salt("settings_language")
                        .selected_text(draft.language.native_name())
                        .show_ui(ui, |ui| {
                            for &language in Language::all() {
                                ui.selectable_value(
                                    &mut draft.language,
                                    language,
                                    language.native_name(),
                                );
                            }
                        });
                });
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    save = ui.button(tr("btn_save")).clicked();
                    cancel = ui.button(tr("btn_cancel")).clicked();
                });
            }
        });

    if save {
        if let Some(draft) = app.ui.settings_draft.as_ref() {
            let mut next = app.settings.clone();
            next.language = draft.language;
            match next.save_public_config() {
                Ok(()) => {
                    app.settings = next;
                    i18n::set_language(app.settings.language);
                    app.ui.settings_draft = None;
                    app.ui.settings_error = None;
                    app.ui.active = ActiveView::Main;
                }
                Err(error) => app.ui.settings_error = Some(error.to_string()),
            }
        }
    } else if cancel || !open {
        app.ui.settings_draft = None;
        app.ui.settings_error = None;
        app.ui.active = ActiveView::Main;
    }
}
