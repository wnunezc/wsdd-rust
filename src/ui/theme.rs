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
//! Paletas de color de la UI.
//!
//! Tres temas oscuros y un tema claro.
//! Se aplican en cada frame antes del render principal.

use egui::{Color32, Rounding, Stroke, Visuals};

use crate::handlers::setting::AppTheme;

/// Aplica la paleta activa al contexto egui.
/// Llamar al inicio de cada frame (antes de renderizar la UI).
pub fn apply(ctx: &egui::Context, theme: AppTheme) {
    ctx.set_visuals(build(theme));
}

fn build(theme: AppTheme) -> Visuals {
    match theme {
        AppTheme::NeutralDark => neutral_dark(),
        AppTheme::BlueTint => blue_tint(),
        AppTheme::WarmDark => warm_dark(),
        AppTheme::Light => light(),
    }
}

// ─── Opción A — Dark Neutral ──────────────────────────────────────────────────
//
// Gris oscuro puro, estilo VS Code.
// Texto blanco, acento azul, sin tinte de color en los fondos.

fn neutral_dark() -> Visuals {
    let mut v = Visuals::dark();

    v.panel_fill         = Color32::from_rgb(26, 26, 26);
    v.window_fill        = Color32::from_rgb(30, 30, 30);
    v.faint_bg_color     = Color32::from_rgb(35, 35, 35);
    v.extreme_bg_color   = Color32::from_rgb(18, 18, 18);
    v.hyperlink_color    = Color32::from_rgb(74, 159, 222);
    v.warn_fg_color      = Color32::from_rgb(255, 200, 87);
    v.error_fg_color     = Color32::from_rgb(255, 85, 85);
    v.window_stroke      = Stroke::new(1.0, Color32::from_rgb(55, 55, 55));
    v.window_rounding    = Rounding::same(6.0);

    // Widget inactivo (estado base de botones, inputs, etc.)
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(220, 220, 220));
    v.widgets.noninteractive.bg_fill   = Color32::from_rgb(26, 26, 26);
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(55, 55, 55));

    v.widgets.inactive.fg_stroke  = Stroke::new(1.0, Color32::WHITE);
    v.widgets.inactive.bg_fill    = Color32::from_rgb(42, 42, 42);
    v.widgets.inactive.weak_bg_fill = Color32::from_rgb(42, 42, 42);
    v.widgets.inactive.bg_stroke  = Stroke::new(1.0, Color32::from_rgb(68, 68, 68));
    v.widgets.inactive.rounding   = Rounding::same(4.0);

    v.widgets.hovered.fg_stroke   = Stroke::new(1.5, Color32::WHITE);
    v.widgets.hovered.bg_fill     = Color32::from_rgb(58, 58, 58);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(58, 58, 58);
    v.widgets.hovered.bg_stroke   = Stroke::new(1.0, Color32::from_rgb(100, 148, 220));
    v.widgets.hovered.rounding    = Rounding::same(4.0);

    v.widgets.active.fg_stroke    = Stroke::new(1.5, Color32::WHITE);
    v.widgets.active.bg_fill      = Color32::from_rgb(74, 74, 74);
    v.widgets.active.weak_bg_fill = Color32::from_rgb(74, 74, 74);
    v.widgets.active.bg_stroke    = Stroke::new(1.0, Color32::from_rgb(100, 148, 220));
    v.widgets.active.rounding     = Rounding::same(4.0);

    v.widgets.open.fg_stroke      = Stroke::new(1.5, Color32::WHITE);
    v.widgets.open.bg_fill        = Color32::from_rgb(50, 50, 50);
    v.widgets.open.bg_stroke      = Stroke::new(1.0, Color32::from_rgb(100, 148, 220));

    v.selection.bg_fill           = Color32::from_rgb(74, 159, 222).linear_multiply(0.4);
    v.selection.stroke            = Stroke::new(1.0, Color32::from_rgb(74, 159, 222));

    v
}

// ─── Opción B — Dark Blue Tint ────────────────────────────────────────────────
//
// Fondo con leve tinte índigo, estilo DevOps/terminal.
// Texto blanco, acento azul índigo.

fn blue_tint() -> Visuals {
    let mut v = Visuals::dark();

    v.panel_fill         = Color32::from_rgb(19, 21, 31);
    v.window_fill        = Color32::from_rgb(26, 29, 46);
    v.faint_bg_color     = Color32::from_rgb(22, 25, 38);
    v.extreme_bg_color   = Color32::from_rgb(13, 14, 22);
    v.hyperlink_color    = Color32::from_rgb(91, 138, 245);
    v.warn_fg_color      = Color32::from_rgb(251, 176, 64);
    v.error_fg_color     = Color32::from_rgb(248, 113, 113);
    v.window_stroke      = Stroke::new(1.0, Color32::from_rgb(42, 45, 64));
    v.window_rounding    = Rounding::same(6.0);

    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(200, 208, 224));
    v.widgets.noninteractive.bg_fill   = Color32::from_rgb(19, 21, 31);
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(42, 45, 64));

    v.widgets.inactive.fg_stroke  = Stroke::new(1.0, Color32::WHITE);
    v.widgets.inactive.bg_fill    = Color32::from_rgb(30, 34, 53);
    v.widgets.inactive.weak_bg_fill = Color32::from_rgb(30, 34, 53);
    v.widgets.inactive.bg_stroke  = Stroke::new(1.0, Color32::from_rgb(58, 64, 96));
    v.widgets.inactive.rounding   = Rounding::same(4.0);

    v.widgets.hovered.fg_stroke   = Stroke::new(1.5, Color32::WHITE);
    v.widgets.hovered.bg_fill     = Color32::from_rgb(42, 48, 80);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(42, 48, 80);
    v.widgets.hovered.bg_stroke   = Stroke::new(1.0, Color32::from_rgb(91, 138, 245));
    v.widgets.hovered.rounding    = Rounding::same(4.0);

    v.widgets.active.fg_stroke    = Stroke::new(1.5, Color32::WHITE);
    v.widgets.active.bg_fill      = Color32::from_rgb(53, 61, 101);
    v.widgets.active.weak_bg_fill = Color32::from_rgb(53, 61, 101);
    v.widgets.active.bg_stroke    = Stroke::new(1.0, Color32::from_rgb(91, 138, 245));
    v.widgets.active.rounding     = Rounding::same(4.0);

    v.widgets.open.fg_stroke      = Stroke::new(1.5, Color32::WHITE);
    v.widgets.open.bg_fill        = Color32::from_rgb(38, 44, 72);
    v.widgets.open.bg_stroke      = Stroke::new(1.0, Color32::from_rgb(91, 138, 245));

    v.selection.bg_fill           = Color32::from_rgb(91, 138, 245).linear_multiply(0.35);
    v.selection.stroke            = Stroke::new(1.0, Color32::from_rgb(91, 138, 245));

    v
}

// ─── Opción C — Dark Warm ─────────────────────────────────────────────────────
//
// Tinte cálido, estilo Monokai/Dracula.
// Texto blanco cálido (#F8F8F2), acento cyan/lima.

fn warm_dark() -> Visuals {
    let mut v = Visuals::dark();

    v.panel_fill         = Color32::from_rgb(28, 28, 26);
    v.window_fill        = Color32::from_rgb(36, 36, 32);
    v.faint_bg_color     = Color32::from_rgb(32, 32, 29);
    v.extreme_bg_color   = Color32::from_rgb(18, 18, 16);
    v.hyperlink_color    = Color32::from_rgb(102, 217, 232);  // cyan Monokai
    v.warn_fg_color      = Color32::from_rgb(230, 219, 116);  // amarillo Monokai
    v.error_fg_color     = Color32::from_rgb(249, 38, 114);   // magenta Monokai
    v.window_stroke      = Stroke::new(1.0, Color32::from_rgb(54, 54, 48));
    v.window_rounding    = Rounding::same(6.0);

    let text_warm = Color32::from_rgb(248, 248, 242);         // #F8F8F2

    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(224, 221, 213));
    v.widgets.noninteractive.bg_fill   = Color32::from_rgb(28, 28, 26);
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(54, 54, 48));

    v.widgets.inactive.fg_stroke  = Stroke::new(1.0, text_warm);
    v.widgets.inactive.bg_fill    = Color32::from_rgb(45, 45, 40);
    v.widgets.inactive.weak_bg_fill = Color32::from_rgb(45, 45, 40);
    v.widgets.inactive.bg_stroke  = Stroke::new(1.0, Color32::from_rgb(74, 74, 64));
    v.widgets.inactive.rounding   = Rounding::same(4.0);

    v.widgets.hovered.fg_stroke   = Stroke::new(1.5, text_warm);
    v.widgets.hovered.bg_fill     = Color32::from_rgb(61, 61, 53);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(61, 61, 53);
    v.widgets.hovered.bg_stroke   = Stroke::new(1.0, Color32::from_rgb(166, 226, 46));   // lima
    v.widgets.hovered.rounding    = Rounding::same(4.0);

    v.widgets.active.fg_stroke    = Stroke::new(1.5, text_warm);
    v.widgets.active.bg_fill      = Color32::from_rgb(80, 80, 69);
    v.widgets.active.weak_bg_fill = Color32::from_rgb(80, 80, 69);
    v.widgets.active.bg_stroke    = Stroke::new(1.0, Color32::from_rgb(166, 226, 46));
    v.widgets.active.rounding     = Rounding::same(4.0);

    v.widgets.open.fg_stroke      = Stroke::new(1.5, text_warm);
    v.widgets.open.bg_fill        = Color32::from_rgb(55, 55, 48);
    v.widgets.open.bg_stroke      = Stroke::new(1.0, Color32::from_rgb(102, 217, 232));

    v.selection.bg_fill           = Color32::from_rgb(102, 217, 232).linear_multiply(0.3);
    v.selection.stroke            = Stroke::new(1.0, Color32::from_rgb(102, 217, 232));

    v
}

// ─── Opción D — Light ─────────────────────────────────────────────────────────
//
// Fondo blanco/gris claro, texto oscuro.
// Acento azul estándar, estilo aplicación de escritorio.

fn light() -> Visuals {
    let mut v = Visuals::light();

    let text_dark    = Color32::from_rgb(30, 30, 30);
    let accent_blue  = Color32::from_rgb(0, 102, 204);
    let border       = Color32::from_rgb(200, 200, 200);

    v.panel_fill         = Color32::from_rgb(245, 245, 245);
    v.window_fill        = Color32::WHITE;
    v.faint_bg_color     = Color32::from_rgb(235, 235, 235);
    v.extreme_bg_color   = Color32::from_rgb(255, 255, 255);
    v.hyperlink_color    = accent_blue;
    v.warn_fg_color      = Color32::from_rgb(180, 100, 0);
    v.error_fg_color     = Color32::from_rgb(200, 30, 30);
    v.window_stroke      = Stroke::new(1.0, border);
    v.window_rounding    = Rounding::same(6.0);

    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text_dark);
    v.widgets.noninteractive.bg_fill   = Color32::from_rgb(245, 245, 245);
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border);

    v.widgets.inactive.fg_stroke   = Stroke::new(1.0, text_dark);
    v.widgets.inactive.bg_fill     = Color32::from_rgb(240, 240, 240);
    v.widgets.inactive.weak_bg_fill = Color32::from_rgb(240, 240, 240);
    v.widgets.inactive.bg_stroke   = Stroke::new(1.0, border);
    v.widgets.inactive.rounding    = Rounding::same(4.0);

    v.widgets.hovered.fg_stroke    = Stroke::new(1.5, text_dark);
    v.widgets.hovered.bg_fill      = Color32::from_rgb(220, 235, 255);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(220, 235, 255);
    v.widgets.hovered.bg_stroke    = Stroke::new(1.0, accent_blue);
    v.widgets.hovered.rounding     = Rounding::same(4.0);

    v.widgets.active.fg_stroke     = Stroke::new(1.5, text_dark);
    v.widgets.active.bg_fill       = Color32::from_rgb(190, 220, 255);
    v.widgets.active.weak_bg_fill  = Color32::from_rgb(190, 220, 255);
    v.widgets.active.bg_stroke     = Stroke::new(1.0, accent_blue);
    v.widgets.active.rounding      = Rounding::same(4.0);

    v.widgets.open.fg_stroke       = Stroke::new(1.5, text_dark);
    v.widgets.open.bg_fill         = Color32::from_rgb(210, 228, 252);
    v.widgets.open.bg_stroke       = Stroke::new(1.0, accent_blue);

    v.selection.bg_fill            = accent_blue.linear_multiply(0.25);
    v.selection.stroke             = Stroke::new(1.0, accent_blue);

    v
}
