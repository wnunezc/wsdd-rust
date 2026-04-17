/// Renders the standard dimmed backdrop used behind modal windows.
pub(crate) fn render_modal_backdrop(ctx: &egui::Context, id: &'static str) {
    let screen_rect = ctx.screen_rect();

    egui::Area::new(egui::Id::new(id))
        .order(egui::Order::Middle)
        .fixed_pos(screen_rect.min)
        .interactable(true)
        .show(ctx, |ui| {
            ui.set_min_size(screen_rect.size());
            let rect = ui.max_rect();
            let response = ui.allocate_rect(rect, egui::Sense::click());
            ui.painter()
                .rect_filled(rect, 0.0, egui::Color32::from_black_alpha(160));

            if response.clicked() {
                ui.ctx().request_repaint();
            }
        });
}
