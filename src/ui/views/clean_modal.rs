use crate::models::{format_size, Application, ResidualCandidate};
use crate::ui::theme::Theme;
use egui::{Align, Color32, Context, Layout, RichText, ScrollArea, Window};

pub struct CleanModal;

impl CleanModal {
    pub fn render(
        ctx: &Context,
        app: &Application,
        residuals: &mut [ResidualCandidate],
        is_permanent: &mut bool,
        is_open: &mut bool,
        is_purging: bool,
        purge_status: &Option<String>,
        on_confirm_purge: &mut bool,
    ) {
        let mut close_requested = false;

        Window::new(format!("🗑️ Deep Clean: {}", app.display_name))
            .open(is_open)
            .resizable(true)
            .default_width(600.0_f32)
            .default_height(480.0_f32)
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Deep Uninstallation & Residual Purge");
                    ui.label(
                        RichText::new("Select the leftover files and directories to remove. All selections are strictly validated against system safety policies.")
                            .color(Theme::TEXT_SECONDARY),
                    );
                    ui.add_space(8.0_f32);

                    // Package Purge Box
                    egui::Frame::none()
                        .fill(Theme::BG_DARK)
                        .inner_margin(10.0_f32)
                        .rounding(6.0_f32)
                        .stroke(egui::Stroke::new(1.0_f32, Theme::BORDER))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("📦 Package:").strong());
                                ui.label(format!("{} ({})", app.name, app.install_method.badge_label()));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(RichText::new(app.formatted_size()).strong());
                                });
                            });
                        });

                    ui.add_space(10.0_f32);
                    ui.heading("Residual Directories & Files to Clean:");
                    ui.add_space(4.0_f32);

                    if residuals.is_empty() {
                        ui.label(RichText::new("No associated residual directories found.").color(Theme::TEXT_MUTED));
                    } else {
                        for r in residuals.iter_mut() {
                            egui::Frame::none()
                                .fill(Theme::BG_DARK)
                                .inner_margin(8.0_f32)
                                .rounding(4.0_f32)
                                .stroke(egui::Stroke::new(1.0_f32, Theme::BORDER))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut r.selected_for_deletion, "");
                                        ui.vertical(|ui| {
                                            ui.label(
                                                RichText::new(r.kind.display_name())
                                                    .strong()
                                                    .color(Theme::TEXT_PRIMARY),
                                            );
                                            ui.label(
                                                RichText::new(r.path.to_string_lossy())
                                                    .color(Theme::TEXT_SECONDARY)
                                                    .size(11.0_f32),
                                            );
                                        });

                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            ui.label(
                                                RichText::new(format_size(r.size_bytes))
                                                    .strong()
                                                    .color(Theme::ACCENT_SUCCESS),
                                            );
                                        });
                                    });
                                });
                            ui.add_space(4.0_f32);
                        }
                    }

                    ui.add_space(12.0_f32);
                    ui.separator();
                    ui.add_space(8.0_f32);

                    // Deletion Mode / Safety Setting
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Safety Mode:").strong());
                        ui.radio_value(is_permanent, false, "Move to Desktop Trash (Recommended)");
                        ui.radio_value(is_permanent, true, "Permanent Hard Delete");
                    });

                    if *is_permanent {
                        ui.label(
                            RichText::new("⚠️ Warning: Hard delete bypasses FreeDesktop trash and permanently removes files.")
                                .color(Theme::ACCENT_WARNING)
                                .size(11.0_f32),
                        );
                    }

                    // Total Selected Size calculation
                    let selected_residual_bytes: u64 = residuals
                        .iter()
                        .filter(|r| r.selected_for_deletion)
                        .map(|r| r.size_bytes)
                        .sum();
                    let total_reclaimable = selected_residual_bytes + app.total_size_bytes;

                    ui.add_space(12.0_f32);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Estimated Space Reclaimed:").strong().size(14.0_f32));
                        ui.label(
                            RichText::new(format_size(total_reclaimable))
                                .strong()
                                .size(16.0_f32)
                                .color(Theme::ACCENT_SUCCESS),
                        );
                    });

                    if let Some(status) = purge_status {
                        ui.add_space(8.0_f32);
                        ui.label(RichText::new(status).color(Theme::ACCENT_CYAN).strong());
                    }

                    ui.add_space(16.0_f32);

                    ui.horizontal(|ui| {
                        let btn_text = if is_purging {
                            "⏳ Cleaning in Progress..."
                        } else {
                            "🚀 Confirm & Purge Application"
                        };

                        let btn = ui.add_enabled(
                            !is_purging,
                            egui::Button::new(
                                RichText::new(btn_text)
                                    .size(14.0_f32)
                                    .color(Color32::WHITE)
                                    .strong(),
                            )
                            .fill(Theme::ACCENT_DANGER)
                            .min_size(egui::Vec2::new(220.0_f32, 32.0_f32)),
                        );

                        if btn.clicked() {
                            *on_confirm_purge = true;
                        }

                        if ui.button("Cancel").clicked() {
                            close_requested = true;
                        }
                    });
                });
            });

        if close_requested {
            *is_open = false;
        }
    }
}
