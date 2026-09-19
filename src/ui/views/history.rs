use crate::db::AuditLogEntry;
use crate::models::format_size;
use crate::ui::theme::Theme;
use egui::{RichText, ScrollArea, Ui};

pub struct HistoryView;

impl HistoryView {
    pub fn render(ui: &mut Ui, logs: &[AuditLogEntry]) {
        ui.heading("📜 Uninstallation & Purge History");
        ui.label(
            RichText::new("Audit trail of all previous package removals and deep cleaned artifacts.")
                .color(Theme::TEXT_SECONDARY),
        );
        ui.add_space(12.0_f32);

        let total_freed: u64 = logs.iter().map(|l| l.freed_bytes).sum();
        egui::Frame::none()
            .fill(Theme::BG_CARD)
            .inner_margin(12.0_f32)
            .rounding(8.0_f32)
            .stroke(egui::Stroke::new(1.0_f32, Theme::BORDER))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Lifetime Disk Space Reclaimed:").strong().size(14.0_f32));
                    ui.label(
                        RichText::new(format_size(total_freed))
                            .strong()
                            .size(16.0_f32)
                            .color(Theme::ACCENT_SUCCESS),
                    );
                });
            });

        ui.add_space(16.0_f32);

        if logs.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0_f32);
                ui.label(
                    RichText::new("No uninstallation history recorded yet.")
                        .color(Theme::TEXT_MUTED)
                        .size(15.0_f32),
                );
            });
            return;
        }

        ScrollArea::vertical().show(ui, |ui| {
            for log in logs {
                egui::Frame::none()
                    .fill(Theme::BG_CARD)
                    .inner_margin(10.0_f32)
                    .rounding(6.0_f32)
                    .stroke(egui::Stroke::new(1.0_f32, Theme::BORDER))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(&log.app_name)
                                        .strong()
                                        .size(14.0_f32)
                                        .color(Theme::TEXT_PRIMARY),
                                );
                                ui.label(
                                    RichText::new(log.timestamp.format("%Y-%m-%d %H:%M:%S").to_string())
                                        .size(11.0_f32)
                                        .color(Theme::TEXT_MUTED),
                                );
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(
                                    RichText::new(format!("+{}", format_size(log.freed_bytes)))
                                        .strong()
                                        .color(Theme::ACCENT_SUCCESS),
                                );
                                ui.label(
                                    RichText::new(&log.install_method)
                                        .size(11.0_f32)
                                        .color(Theme::TEXT_SECONDARY),
                                );
                            });
                        });
                    });
                ui.add_space(4.0_f32);
            }
        });
    }
}
