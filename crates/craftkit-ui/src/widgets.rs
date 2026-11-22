//! Small shared widgets used by CraftKit applications.

use craftkit_jobs::JobInfo;
use eframe::egui::{self, RichText, Vec2};

use crate::theme;

/// Uppercase section label used at the top of panels.
pub fn section_label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text.to_uppercase())
            .size(10.5)
            .color(theme::FAINT),
    );
}

/// Compact toolbar button.
pub fn toolbar_button(ui: &mut egui::Ui, label: &str, hover: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).size(11.0))
            .fill(theme::PANEL)
            .corner_radius(egui::CornerRadius::same(5))
            .min_size(Vec2::new(0.0, 27.0)),
    )
    .on_hover_text(hover)
}

/// Centered empty-state message.
pub fn empty_state(ui: &mut egui::Ui, title: &str, body: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(64.0);
        ui.label(RichText::new(title).size(16.0).strong());
        ui.add_space(4.0);
        ui.label(RichText::new(body).size(11.5).color(theme::MUTED));
    });
}

/// Progress row for a background job with a cancel button.
pub fn job_row(ui: &mut egui::Ui, job: &JobInfo) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(&job.label).size(11.5).color(theme::TEXT));
        ui.add(
            egui::ProgressBar::new(job.fraction())
                .desired_width(150.0)
                .show_percentage(),
        );
        let message = job.message();
        if !message.is_empty() {
            ui.label(RichText::new(message).size(10.5).color(theme::MUTED));
        }
        if !job.is_cancelled() && ui.small_button("Cancel").clicked() {
            job.cancel();
        }
    });
}
