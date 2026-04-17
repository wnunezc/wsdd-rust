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
//! Pantalla de ayuda de WSDD.
//! Renderiza la documentacion viva desde `docs/help`.

use egui_commonmark::CommonMarkViewer;

use crate::app::WsddApp;
use crate::i18n::{tr, Language};
use crate::ui::ActiveView;

const HELP_EN: &str = include_str!("../../docs/help/user-guide.en.md");
const HELP_ES: &str = include_str!("../../docs/help/user-guide.es.md");
const HELP_FR: &str = include_str!("../../docs/help/user-guide.fr.md");
const HELP_ZH: &str = include_str!("../../docs/help/user-guide.zh.md");

struct HelpSection<'a> {
    title: &'a str,
    markdown: &'a str,
}

fn help_markdown(language: Language) -> &'static str {
    match language {
        Language::Es => HELP_ES,
        Language::Fr => HELP_FR,
        Language::Hi => HELP_EN,
        Language::Zh => HELP_ZH,
        _ => HELP_EN,
    }
}

/// Renders the WSDD help screen.
pub fn render(ctx: &egui::Context, app: &mut WsddApp) {
    let close_label = format!("  {}  ", tr("btn_close"));
    let search_label = tr("help_search");
    let search_hint = tr("help_search_hint");
    let sections_found = tr("help_sections_found");
    let no_results = tr("help_no_results");
    let sections = parse_sections(help_markdown(app.settings.language));

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading(format!("{} — WSDD", tr("help_title")));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(&close_label).clicked() {
                    app.ui.active = ActiveView::Main;
                    app.ui.helps_search.clear();
                }
            });
        });

        ui.horizontal(|ui| {
            ui.label(&search_label);
            let search_field = ui.add(
                egui::TextEdit::singleline(&mut app.ui.helps_search)
                    .desired_width(280.0)
                    .hint_text(&search_hint),
            );
            if ui.button("✗").clicked() {
                app.ui.helps_search.clear();
            }
            if search_field.gained_focus() {
                app.ui.helps_search.clear();
            }
        });

        ui.separator();
        ui.add_space(4.0);

        let query = app.ui.helps_search.to_lowercase();
        let is_filtering = !query.is_empty();

        if is_filtering {
            let matches = sections
                .iter()
                .filter(|section| section_matches(section, &query))
                .count();
            let color = if matches == 0 {
                egui::Color32::from_rgb(200, 80, 80)
            } else {
                ui.visuals().weak_text_color()
            };
            ui.label(
                egui::RichText::new(format!("{matches} {sections_found}"))
                    .size(11.0)
                    .color(color),
            );
            ui.add_space(2.0);
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for section in &sections {
                    let matches = !is_filtering || section_matches(section, &query);
                    if !matches {
                        continue;
                    }

                    egui::CollapsingHeader::new(egui::RichText::new(section.title).strong())
                        .default_open(is_filtering)
                        .show(ui, |ui| {
                            CommonMarkViewer::new().show(
                                ui,
                                &mut app.ui.md_cache,
                                section.markdown,
                            );
                        });

                    ui.add_space(2.0);
                }

                if is_filtering
                    && !sections
                        .iter()
                        .any(|section| section_matches(section, &query))
                {
                    ui.add_space(20.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(&no_results).color(ui.visuals().weak_text_color()),
                        );
                    });
                }

                ui.add_space(16.0);
            });
    });
}

fn parse_sections(markdown: &'static str) -> Vec<HelpSection<'static>> {
    let mut headings = Vec::new();
    let mut offset = 0;

    for raw_line in markdown.split_inclusive('\n') {
        let line = raw_line.trim_end_matches(['\r', '\n']);
        if let Some(title) = line.strip_prefix("## ") {
            headings.push((offset, offset + raw_line.len(), title.trim()));
        }
        offset += raw_line.len();
    }

    headings
        .iter()
        .enumerate()
        .map(|(index, (_, body_start, title))| {
            let body_end = headings
                .get(index + 1)
                .map(|(heading_start, _, _)| *heading_start)
                .unwrap_or(markdown.len());
            HelpSection {
                title,
                markdown: markdown[*body_start..body_end].trim(),
            }
        })
        .collect()
}

fn section_matches(section: &HelpSection<'_>, query: &str) -> bool {
    section.title.to_lowercase().contains(query) || section.markdown.to_lowercase().contains(query)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_guides_have_parseable_sections() {
        for guide in [HELP_EN, HELP_ES, HELP_FR, HELP_ZH] {
            let sections = parse_sections(guide);
            assert_eq!(sections.len(), 13);
            assert!(sections.iter().all(|section| {
                !section.title.trim().is_empty() && !section.markdown.trim().is_empty()
            }));
        }
    }
}
