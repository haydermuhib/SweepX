use crate::cleaner::{SystemSweepItem, SystemSweepReport};
use crate::models::format_size;
use crate::ui::theme::Theme;
use egui::{Align, Color32, Layout, RichText, ScrollArea, Stroke, Ui};

pub struct OptimizerView;

impl OptimizerView {
    pub fn render(
        ui: &mut Ui,
        items: &mut [SystemSweepItem],
        is_cleaning: bool,
        last_report: &Option<SystemSweepReport>,
        on_trigger_clean: &mut bool,
        on_refresh: &mut bool,
    ) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("🧹 System Optimizer & Container Sweep");
                ui.label(
                    RichText::new("Clean obsolete Snap revisions, unused Flatpak runtimes, and native package manager download caches.")
                        .color(Theme::TEXT_SECONDARY),
                );
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(egui::Button::new("🔄 Re-scan System"))
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    *on_refresh = true;
                }
            });
        });

        ui.add_space(12.0_f32);

        // Summary Metric Cards
        let selected_count = items.iter().filter(|i| i.selected).count();
        let total_reclaimable: u64 = items
            .iter()
            .filter(|i| i.selected)
            .map(|i| i.reclaimable_bytes)
            .sum();
        let all_reclaimable: u64 = items.iter().map(|i| i.reclaimable_bytes).sum();

        ui.horizontal(|ui| {
            Self::metric_card(
                ui,
                "Reclaimable Space",
                &format_size(total_reclaimable),
                &format!("Selected from {} available", format_size(all_reclaimable)),
                Theme::ACCENT_SUCCESS,
            );
            ui.add_space(12.0_f32);

            Self::metric_card(
                ui,
                "Selected Items",
                &format!("{}/{}", selected_count, items.len()),
                "Items queued for optimization",
                Theme::PRIMARY,
            );
            ui.add_space(12.0_f32);

            if let Some(report) = last_report {
                Self::metric_card(
                    ui,
                    "Last Sweep Result",
                    &format!("Freed {}", format_size(report.freed_bytes)),
                    &format!("{} items cleaned", report.items_cleaned),
                    Theme::ACCENT_CYAN,
                );
            }
        });

        ui.add_space(14.0_f32);

        // Action Toolbar
        ui.horizontal(|ui| {
            let select_all_btn = ui
                .add(egui::Button::new("Select All"))
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            if select_all_btn.clicked() {
                for item in items.iter_mut() {
                    item.selected = true;
                }
            }

            let deselect_btn = ui
                .add(egui::Button::new("Deselect All"))
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            if deselect_btn.clicked() {
                for item in items.iter_mut() {
                    item.selected = false;
                }
            }

            ui.add_space(16.0_f32);

            let clean_btn_text = if is_cleaning {
                "⏳ Optimizing System..."
            } else {
                "⚡ Clean Selected System Bloat"
            };

            let clean_btn = ui.add_enabled(
                !is_cleaning && selected_count > 0,
                egui::Button::new(
                    RichText::new(clean_btn_text)
                        .size(13.0_f32)
                        .color(Color32::WHITE)
                        .strong(),
                )
                .fill(Theme::ACCENT_DANGER)
                .min_size(egui::vec2(220.0_f32, 28.0_f32)),
            );

            if clean_btn.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                *on_trigger_clean = true;
            }
        });

        ui.add_space(10.0_f32);
        ui.separator();
        ui.add_space(8.0_f32);

        if items.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0_f32);
                ui.label(RichText::new("✨ System is fully optimized!").size(16.0_f32).strong().color(Theme::ACCENT_SUCCESS));
                ui.add_space(4.0_f32);
                ui.label(RichText::new("No obsolete Snap revisions, unused Flatpak runtimes, or package caches found.").color(Theme::TEXT_MUTED));
            });
            return;
        }

        // List of Sweepable items
        ScrollArea::vertical().show(ui, |ui| {
            for item in items.iter_mut() {
                let card_bg = if item.selected { Theme::BG_CARD } else { Theme::BG_DARK };
                let stroke = if item.selected {
                    Stroke::new(1.0_f32, Theme::PRIMARY_MUTED)
                } else {
                    Stroke::new(1.0_f32, Theme::BORDER)
                };

                egui::Frame::none()
                    .fill(card_bg)
                    .stroke(stroke)
                    .rounding(8.0_f32)
                    .inner_margin(12.0_f32)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut item.selected, "");

                            ui.add_space(4.0_f32);
                            ui.label(RichText::new(item.category.icon()).size(16.0_f32));

                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(&item.title).strong().size(13.0_f32).color(Theme::TEXT_PRIMARY));
                                    ui.add_space(6.0_f32);
                                    ui.label(
                                        RichText::new(format!("[{}]", item.category.display_name()))
                                            .size(11.0_f32)
                                            .color(Theme::PRIMARY_HOVER),
                                    );
                                });

                                ui.add_space(2.0_f32);
                                ui.label(RichText::new(&item.description).size(11.0_f32).color(Theme::TEXT_SECONDARY));

                                if let Some(ref cmd) = item.command {
                                    ui.add_space(2.0_f32);
                                    ui.label(
                                        RichText::new(format!("Command: {}", cmd))
                                            .size(10.0_f32)
                                            .monospace()
                                            .color(Theme::TEXT_MUTED),
                                    );
                                }
                            });

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if item.reclaimable_bytes > 0 {
                                    ui.label(
                                        RichText::new(format!("+{}", format_size(item.reclaimable_bytes)))
                                            .strong()
                                            .size(13.0_f32)
                                            .color(Theme::ACCENT_SUCCESS),
                                    );
                                } else {
                                    ui.label(
                                        RichText::new("Dynamic")
                                            .size(12.0_f32)
                                            .color(Theme::TEXT_MUTED),
                                    );
                                }
                            });
                        });
                    });
                ui.add_space(6.0_f32);
            }
        });
    }

    fn metric_card(ui: &mut Ui, title: &str, value: &str, subtitle: &str, accent: Color32) {
        egui::Frame::none()
            .fill(Theme::BG_CARD)
            .inner_margin(12.0_f32)
            .rounding(8.0_f32)
            .stroke(Stroke::new(1.0_f32, Theme::BORDER))
            .show(ui, |ui| {
                ui.set_width(240.0_f32);
                ui.vertical(|ui| {
                    ui.label(RichText::new(title).size(12.0_f32).color(Theme::TEXT_MUTED));
                    ui.add_space(4.0_f32);
                    ui.label(RichText::new(value).size(18.0_f32).strong().color(accent));
                    ui.add_space(2.0_f32);
                    ui.label(RichText::new(subtitle).size(11.0_f32).color(Theme::TEXT_SECONDARY));
                });
            });
    }
}
