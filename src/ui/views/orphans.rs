use crate::models::{format_size, ResidualCandidate};
use crate::ui::theme::Theme;
use egui::{Align, Layout, RichText, ScrollArea, Ui};

pub struct OrphansView;

impl OrphansView {
    pub fn render(
        ui: &mut Ui,
        orphans: &mut [ResidualCandidate],
        is_cleaning: bool,
        on_clean_orphans: &mut bool,
    ) {
        ui.heading("🧹 Orphaned Residual Cleaner");
        ui.label(
            RichText::new("Scan and clean leftover configuration and cache folders from previously removed software.")
                .color(Theme::TEXT_SECONDARY),
        );
        ui.add_space(12.0_f32);

        let total_waste: u64 = orphans
            .iter()
            .filter(|o| o.selected_for_deletion)
            .map(|o| o.size_bytes)
            .sum();

        ui.horizontal(|ui| {
            ui.label(RichText::new("Selected Reclaimable Waste:").strong().size(14.0_f32));
            ui.label(
                RichText::new(format_size(total_waste))
                    .strong()
                    .size(16.0_f32)
                    .color(Theme::ACCENT_SUCCESS),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let btn = ui.add_enabled(
                    !is_cleaning && !orphans.is_empty(),
                    egui::Button::new(
                        RichText::new("🗑️ Clean Selected Residuals")
                            .color(egui::Color32::WHITE)
                            .strong(),
                    )
                    .fill(Theme::ACCENT_DANGER),
                );

                if btn.clicked() {
                    *on_clean_orphans = true;
                }
            });
        });

        ui.add_space(12.0_f32);
        ui.separator();
        ui.add_space(8.0_f32);

        if orphans.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0_f32);
                ui.label(
                    RichText::new("✨ No orphaned residuals found! Your system is clean.")
                        .color(Theme::ACCENT_SUCCESS)
                        .size(15.0_f32),
                );
            });
            return;
        }

        ScrollArea::vertical().show(ui, |ui| {
            for o in orphans.iter_mut() {
                egui::Frame::none()
                    .fill(Theme::BG_CARD)
                    .inner_margin(10.0_f32)
                    .rounding(6.0_f32)
                    .stroke(egui::Stroke::new(1.0_f32, Theme::BORDER))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut o.selected_for_deletion, "");
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(&o.app_name)
                                        .strong()
                                        .size(14.0_f32)
                                        .color(Theme::TEXT_PRIMARY),
                                );
                                ui.label(
                                    RichText::new(o.path.to_string_lossy())
                                        .size(11.0_f32)
                                        .color(Theme::TEXT_SECONDARY),
                                );
                            });

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.button("📂 Open").clicked() {
                                    let _ = open::that(&o.path);
                                }
                                ui.label(
                                    RichText::new(format_size(o.size_bytes))
                                        .strong()
                                        .color(Theme::ACCENT_SUCCESS),
                                );
                                ui.label(
                                    RichText::new(o.kind.display_name())
                                        .size(11.0_f32)
                                        .color(Theme::TEXT_MUTED),
                                );
                            });
                        });
                    });
                ui.add_space(4.0_f32);
            }
        });
    }
}
