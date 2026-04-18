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
//! Silent loader presentation for automatic prerequisite checks.

use egui::{Color32, Pos2, Rect, RichText, Stroke};

use crate::i18n::tr;

/// Renders the quiet verification screen used after setup has completed once.
pub(super) fn render(ctx: &egui::Context) {
    let time = ctx.input(|i| i.time);
    let dot_count = (time * 1.5) as usize % 4;
    let dots: String = ".".repeat(dot_count);
    let pad: String = " ".repeat(3usize.saturating_sub(dot_count));
    let checking = tr("loader_verifying_environment");

    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            paint_loader_background(ui.painter(), rect, time, ui.visuals().dark_mode);

            ui.vertical_centered(|ui| {
                let top_space =
                    (ui.available_height() - ui.available_height() * 0.12 - 36.0).max(0.0);
                ui.add_space(top_space);

                ui.label(
                    RichText::new(format!("{checking}{dots}{pad}").to_uppercase())
                        .size(16.0)
                        .color(Color32::from_rgba_unmultiplied(255, 184, 168, 226))
                        .monospace()
                        .strong(),
                );
            });
        });

    ctx.request_repaint();
}

fn paint_loader_background(painter: &egui::Painter, rect: Rect, time: f64, _dark: bool) {
    let base = Color32::from_rgb(1, 2, 3);
    painter.rect_filled(rect, 0.0, base);

    paint_radial_field(painter, rect);
    paint_far_stars(painter, rect, time);
    paint_energy_rays(painter, rect, time);
    paint_orbit_rings(painter, rect, time);
    paint_satellites(painter, rect, time);
    paint_core(painter, rect, time);
    paint_core_ejections(painter, rect, time);
    paint_particles(painter, rect, time);
}

fn paint_radial_field(painter: &egui::Painter, rect: Rect) {
    let center = rect.center();
    let max_radius = rect.width().hypot(rect.height()) * 0.52;
    let layers = 44;

    for i in (0..layers).rev() {
        let t = i as f32 / layers as f32;
        let radius = max_radius * t;
        let alpha = ((1.0 - t).powf(1.8) * 58.0) as u8;
        let color = Color32::from_rgba_unmultiplied(34, 2, 7, alpha);
        painter.circle_filled(center, radius, color);
    }

    for i in (0..18).rev() {
        let t = i as f32 / 18.0;
        let radius = max_radius * 0.34 * t;
        let alpha = ((1.0 - t).powf(1.2) * 76.0) as u8;
        painter.circle_filled(
            center,
            radius,
            Color32::from_rgba_unmultiplied(180, 10, 18, alpha),
        );
    }
}

fn paint_core(painter: &egui::Painter, rect: Rect, time: f64) {
    let center = rect.center();
    let t = time as f32;
    let pulse = 28.0 + (t * 2.1).sin() * 5.5;
    let crimson = Color32::from_rgba_unmultiplied(255, 42, 36, 235);

    for i in (1..=17).rev() {
        let radius = i as f32 * 11.0 + (t * 1.05 + i as f32).sin() * 2.6;
        let alpha = (54.0 * (1.0 - i as f32 / 18.0)) as u8;
        painter.circle_filled(
            center,
            radius,
            Color32::from_rgba_unmultiplied(170, 0, 12, alpha),
        );
    }

    painter.circle_filled(
        center,
        pulse + 18.0,
        Color32::from_rgba_unmultiplied(255, 38, 22, 88),
    );
    painter.circle_filled(
        center,
        pulse + 7.0,
        Color32::from_rgba_unmultiplied(255, 86, 42, 145),
    );
    painter.circle_filled(
        center,
        pulse,
        Color32::from_rgba_unmultiplied(255, 128, 72, 246),
    );
    painter.circle_filled(
        center,
        pulse * 0.46,
        Color32::from_rgba_unmultiplied(255, 238, 194, 250),
    );

    painter.circle_stroke(
        center,
        42.0 + (t * 1.3).sin() * 2.0,
        Stroke::new(1.4, with_alpha(crimson, 78)),
    );
    painter.circle_stroke(
        center,
        58.0 + (t * 1.1).cos() * 2.0,
        Stroke::new(1.0, with_alpha(crimson, 44)),
    );
    painter.circle_stroke(
        center,
        pulse + 30.0,
        Stroke::new(1.0, with_alpha(crimson, 34)),
    );
}

fn paint_orbit_rings(painter: &egui::Painter, rect: Rect, time: f64) {
    const SEGMENTS: usize = 72;
    let center = rect.center();
    let min_side = rect.width().min(rect.height());
    let base_color = Color32::from_rgba_unmultiplied(255, 72, 52, 94);

    for ring in 0..3 {
        let radius = min_side * (0.145 + ring as f32 * 0.05);
        let spin =
            time as f32 * (0.35 + ring as f32 * 0.13) * if ring % 2 == 0 { 1.0 } else { -1.0 };
        for i in 0..SEGMENTS {
            if (i + ring) % 3 != 0 {
                continue;
            }
            let a1 = (i as f32 / SEGMENTS as f32) * std::f32::consts::TAU + spin;
            let a2 = a1 + 0.05 + ring as f32 * 0.012;
            painter.line_segment(
                [
                    pos_from_polar(center, radius, a1),
                    pos_from_polar(center, radius, a2),
                ],
                Stroke::new(
                    1.2,
                    with_alpha(base_color, 78_u8.saturating_sub(ring as u8 * 18)),
                ),
            );
        }
    }
}

fn paint_particles(painter: &egui::Painter, rect: Rect, time: f64) {
    const PARTICLE_COUNT: usize = 360;
    let center = rect.center();
    let max_radius = rect.width().min(rect.height()) * 0.32;
    let boundary = rect.width().max(rect.height()) * 0.42;
    let t = time as f32;

    for i in 0..PARTICLE_COUNT {
        let seed = seeded_unit(i as u32, 17) * 1_000.0;
        let base_angle = seeded_unit(i as u32, 31) * std::f32::consts::TAU;
        let orbit = 0.6 + seeded_unit(i as u32, 47) * 0.6;
        let age = (t * (0.08 + orbit * 0.035) + seeded_unit(i as u32, 61)).fract();
        let radius_seed = 30.0 + seeded_unit(i as u32, 79) * max_radius;
        let radius = (radius_seed * (0.62 + age * 0.38)).min(boundary);
        let angle =
            base_angle + t * (0.42 + orbit * 0.22) + (t + seed + radius * 0.01).sin() * 0.18;
        let pos = pos_from_polar(center, radius, angle);
        let size = 1.0 + seeded_unit(i as u32, 97) * 1.4;
        let alpha = ((0.18_f32.max(1.0 - age)) * 238.0) as u8;
        let wave = (t + seed).sin() * 0.5 + 0.5;
        let color = particle_color(wave, alpha);

        for trail in 1..=7 {
            let trail_t = trail as f32;
            let previous = pos_from_polar(center, radius, angle - 0.04 * orbit * trail_t);
            let trail_alpha = (alpha as f32 * (0.25 / trail_t)) as u8;
            painter.line_segment(
                [pos, previous],
                Stroke::new(
                    size * (2.2 / trail_t.sqrt()),
                    Color32::from_rgba_unmultiplied(30, 0, 0, trail_alpha.saturating_div(2)),
                ),
            );
            painter.line_segment(
                [pos, previous],
                Stroke::new(
                    size * (0.95 / trail_t.sqrt()),
                    with_alpha(color, trail_alpha),
                ),
            );
        }

        if i % 4 == 0 {
            painter.circle_filled(
                pos,
                size * 3.2,
                with_alpha(color, (alpha as f32 * 0.08) as u8),
            );
        }
        painter.circle_filled(pos, size, color);
    }
}

fn paint_far_stars(painter: &egui::Painter, rect: Rect, time: f64) {
    let t = time as f32;
    for i in 0..90 {
        let x = rect.left() + seeded_unit(i, 113) * rect.width();
        let y = rect.top() + seeded_unit(i, 127) * rect.height();
        let flicker = ((t * (0.8 + seeded_unit(i, 131)) + seeded_unit(i, 149) * 10.0).sin() * 0.5
            + 0.5)
            * 55.0;
        let alpha = (18.0 + flicker) as u8;
        painter.circle_filled(
            Pos2::new(x, y),
            0.6 + seeded_unit(i, 151) * 1.2,
            Color32::from_rgba_unmultiplied(255, 104, 74, alpha),
        );
    }
}

fn paint_energy_rays(painter: &egui::Painter, rect: Rect, time: f64) {
    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.36;
    let t = time as f32;

    for i in 0..18 {
        let phase = seeded_unit(i, 173) * std::f32::consts::TAU;
        let angle = phase + t * (0.06 + seeded_unit(i, 181) * 0.12);
        let inner = radius * (0.36 + seeded_unit(i, 191) * 0.22);
        let outer = radius * (0.82 + seeded_unit(i, 193) * 0.36);
        let alpha = (18.0 + seeded_unit(i, 197) * 36.0) as u8;
        painter.line_segment(
            [
                pos_from_polar(center, inner, angle),
                pos_from_polar(center, outer, angle + 0.05),
            ],
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 52, 36, alpha)),
        );
    }
}

fn paint_core_ejections(painter: &egui::Painter, rect: Rect, time: f64) {
    const EJECTION_COUNT: usize = 120;
    let center = rect.center();
    let t = time as f32;
    let max_radius = rect.width().min(rect.height()) * 0.34;

    for i in 0..EJECTION_COUNT {
        let speed = 0.18 + seeded_unit(i as u32, 211) * 0.16;
        let age = (t * speed + seeded_unit(i as u32, 223)).fract();
        let angle = seeded_unit(i as u32, 227) * std::f32::consts::TAU
            + age * 0.42
            + (t * 0.8 + seeded_unit(i as u32, 229) * 8.0).sin() * 0.12;
        let radius = 18.0 + age.powf(1.28) * max_radius;
        let pos = pos_from_polar(center, radius, angle);
        let size = 0.9 + seeded_unit(i as u32, 233) * 1.5;
        let alpha = ((1.0 - age).powf(0.72) * 230.0) as u8;
        let color = Color32::from_rgba_unmultiplied(
            255,
            (58.0 + seeded_unit(i as u32, 239) * 76.0) as u8,
            (34.0 + seeded_unit(i as u32, 241) * 38.0) as u8,
            alpha,
        );

        for trail in 1..=8 {
            let trail_t = trail as f32;
            let trail_age = (age - 0.012 * trail_t).max(0.0);
            let trail_radius = 18.0 + trail_age.powf(1.28) * max_radius;
            let previous = pos_from_polar(center, trail_radius, angle - trail_t * 0.01);
            let trail_alpha = (alpha as f32 * (0.34 / trail_t)) as u8;

            painter.line_segment(
                [pos, previous],
                Stroke::new(
                    size * (2.6 / trail_t.sqrt()),
                    Color32::from_rgba_unmultiplied(35, 0, 0, trail_alpha.saturating_div(2)),
                ),
            );
            painter.line_segment(
                [pos, previous],
                Stroke::new(
                    size * (1.15 / trail_t.sqrt()),
                    with_alpha(color, trail_alpha),
                ),
            );
        }

        painter.circle_filled(pos, size, color);
    }
}

fn paint_satellites(painter: &egui::Painter, rect: Rect, time: f64) {
    let center = rect.center();
    let min_side = rect.width().min(rect.height());
    let t = time as f32;

    for i in 0..5 {
        let orbit = min_side * (0.24 + i as f32 * 0.038);
        let speed = 0.42 + i as f32 * 0.09;
        let angle = t * speed + seeded_unit(i, 257) * std::f32::consts::TAU;
        let wobble = (t * (1.1 + i as f32 * 0.2)).sin() * min_side * 0.006;
        let pos = pos_from_polar(center, orbit + wobble, angle);
        let radius = 4.2 + seeded_unit(i, 263) * 3.4;
        let body = Color32::from_rgba_unmultiplied(90, 8, 10, 245);
        let rim = Color32::from_rgba_unmultiplied(255, 96, 62, 210);

        for trail in 1..=10 {
            let trail_t = trail as f32;
            let previous = pos_from_polar(center, orbit + wobble, angle - trail_t * 0.035 * speed);
            painter.line_segment(
                [pos, previous],
                Stroke::new(
                    radius * (0.42 / trail_t.sqrt()),
                    Color32::from_rgba_unmultiplied(255, 40, 30, (86.0 / trail_t) as u8),
                ),
            );
        }

        painter.circle_filled(
            pos,
            radius * 2.8,
            Color32::from_rgba_unmultiplied(160, 0, 12, 26),
        );
        painter.circle_filled(pos, radius, body);
        painter.circle_stroke(pos, radius + 1.2, Stroke::new(1.0, rim));
        painter.circle_filled(
            Pos2::new(pos.x - radius * 0.25, pos.y - radius * 0.3),
            radius * 0.32,
            Color32::from_rgba_unmultiplied(255, 170, 120, 210),
        );
    }
}

fn pos_from_polar(center: Pos2, radius: f32, angle: f32) -> Pos2 {
    Pos2::new(
        center.x + angle.cos() * radius,
        center.y + angle.sin() * radius,
    )
}

fn particle_color(wave: f32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (170.0 + wave * 85.0) as u8,
        (12.0 + wave * 86.0) as u8,
        (18.0 + wave * 45.0) as u8,
        alpha,
    )
}

fn with_alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

fn seeded_unit(index: u32, salt: u32) -> f32 {
    let mut value = index.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    value ^= salt.wrapping_mul(277_803_737);
    value ^= value >> 16;
    value = value.wrapping_mul(2_246_822_519);
    value ^= value >> 13;
    value = value.wrapping_mul(3_266_489_917);
    value ^= value >> 16;
    value as f32 / u32::MAX as f32
}
