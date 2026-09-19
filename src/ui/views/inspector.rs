use crate::models::{format_size, Application, ResidualCandidate};
use crate::ui::theme::Theme;
use egui::{Context, RichText, ScrollArea, Window};

pub struct InspectorModal;

impl InspectorModal {
    pub fn render(
        ctx: &Context,
        app: &Application,
        residuals: &[ResidualCandidate],
        is_open: &mut bool,
    ) {
        Window::new(format!("🔍 Application Inspector: {}", app.display_name))
            .open(is_open)
            .resizable(true)
            .default_width(650.0_f32)
            .default_height(500.0_f32)
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.heading(&app.display_name);
                    ui.label(RichText::new(&app.id).color(Theme::TEXT_MUTED));
                    ui.add_space(8.0_f32);

                    if let Some(ref desc) = app.description {
                        ui.label(RichText::new(desc).color(Theme::TEXT_SECONDARY));
                        ui.add_space(8.0_f32);
                    }

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Packaging Tier:").strong());
                        ui.label(app.install_method.badge_label());

                        ui.add_space(16.0_f32);
                        ui.label(RichText::new("Version:").strong());
                        ui.label(app.version.as_deref().unwrap_or("N/A"));

                        ui.add_space(16.0_f32);
                        ui.label(RichText::new("Total Footprint:").strong());
                        ui.label(app.formatted_size());
                    });

                    ui.add_space(12.0_f32);
                    ui.separator();
                    ui.add_space(8.0_f32);

                    ui.heading("📦 Primary Executable & Launchers");
                    ui.add_space(4.0_f32);

                    if let Some(ref bin) = app.exec_path {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Binary:").strong().color(Theme::PRIMARY));
                            ui.label(bin.to_string_lossy());
                            if ui
                                .button("📂 Open")
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .clicked()
                            {
                                if let Some(parent) = bin.parent() {
                                    let _ = open::that(parent);
                                }
                            }
                        });
                    }

                    if let Some(ref desk) = app.desktop_file {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Desktop Launcher:").strong().color(Theme::ACCENT_CYAN));
                            ui.label(desk.to_string_lossy());
                            if ui
                                .button("📂 Open")
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .clicked()
                            {
                                if let Some(parent) = desk.parent() {
                                    let _ = open::that(parent);
                                }
                            }
                        });
                    }

                    ui.add_space(12.0_f32);
                    ui.separator();
                    ui.add_space(8.0_f32);

                    ui.heading(format!("📂 Discovered Artifacts & Caches ({})", residuals.len()));
                    ui.add_space(4.0_f32);

                    if residuals.is_empty() {
                        ui.label(RichText::new("No leftover caches or user config paths found.").color(Theme::TEXT_MUTED));
                    } else {
                        for r in residuals {
                            egui::Frame::none()
                                .fill(Theme::BG_DARK)
                                .inner_margin(8.0_f32)
                                .rounding(4.0_f32)
                                .stroke(egui::Stroke::new(1.0_f32, Theme::BORDER))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.vertical(|ui| {
                                            ui.label(
                                                RichText::new(r.kind.display_name())
                                                    .strong()
                                                    .color(Theme::ACCENT_WARNING),
                                            );
                                            ui.label(
                                                RichText::new(r.path.to_string_lossy())
                                                    .color(Theme::TEXT_SECONDARY),
                                            );
                                        });

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui
                                                .button("📂 Open Folder")
                                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                                .clicked()
                                            {
                                                let _ = open::that(&r.path);
                                            }
                                            ui.label(RichText::new(format_size(r.size_bytes)).strong());
                                        });
                                    });
                                });
                            ui.add_space(4.0_f32);
                        }
                    }

                    if !app.dependencies.is_empty() {
                        ui.add_space(12.0_f32);
                        ui.separator();
                        ui.add_space(8.0_f32);
                        ui.heading("🔗 Package Dependencies");
                        ui.add_space(4.0_f32);
                        for dep in &app.dependencies {
                            ui.label(format!("• {}", dep));
                        }
                    }
                });
            });
    }
}
