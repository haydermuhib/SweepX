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
        let package_size = app.total_size_bytes;
        let residuals_size: u64 = residuals.iter().map(|r| r.size_bytes).sum();
        let total_combined_footprint = package_size + residuals_size;

        Window::new(format!("🔍 Application Inspector: {}", app.display_name))
            .open(is_open)
            .resizable(true)
            .default_width(650.0_f32)
            .default_height(540.0_f32)
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
                        ui.label(
                            RichText::new(format_size(total_combined_footprint))
                                .color(Theme::ACCENT_CYAN)
                                .strong(),
                        );
                    });

                    ui.add_space(12.0_f32);
                    ui.separator();
                    ui.add_space(8.0_f32);

                    // ==========================================
                    // 1. PRIMARY EXECUTABLE & APPLICATION PAYLOAD
                    // ==========================================
                    ui.heading("📦 Application Payload & Executables");
                    ui.add_space(4.0_f32);

                    egui::Frame::none()
                        .fill(Theme::BG_DARK)
                        .inner_margin(8.0_f32)
                        .rounding(4.0_f32)
                        .stroke(egui::Stroke::new(1.0_f32, Theme::BORDER))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    let payload_label = match app.install_method {
                                        crate::models::InstallMethod::NativeApt => "System Package Payload (APT)",
                                        crate::models::InstallMethod::NativeDnf => "System Package Payload (DNF)",
                                        crate::models::InstallMethod::NativePacman => "System Package Payload (Pacman)",
                                        crate::models::InstallMethod::Flatpak => "Container App Bundle (Flatpak)",
                                        crate::models::InstallMethod::Snap => "Snap Package Bundle",
                                        crate::models::InstallMethod::AppImage => "AppImage Binary",
                                        crate::models::InstallMethod::ManualOpt => "Installed Application Files (/opt)",
                                        crate::models::InstallMethod::CustomDesktop => "Desktop Application",
                                    };
                                    ui.label(RichText::new(payload_label).strong().color(Theme::ACCENT_CYAN));
                                    if let Some(ref bin) = app.exec_path {
                                        ui.label(RichText::new(bin.to_string_lossy()).color(Theme::TEXT_SECONDARY));
                                    } else {
                                        ui.label(RichText::new("Installed package binaries & assets").color(Theme::TEXT_MUTED));
                                    }
                                });

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if let Some(ref bin) = app.exec_path {
                                        if ui.button("📂 Open").on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                            if let Some(parent) = bin.parent() {
                                                let _ = open::that(parent);
                                            }
                                        }
                                    }
                                    ui.label(RichText::new(format_size(package_size)).strong());
                                });
                            });
                        });

                    if let Some(ref desk) = app.desktop_file {
                        ui.add_space(4.0_f32);
                        egui::Frame::none()
                            .fill(Theme::BG_DARK)
                            .inner_margin(8.0_f32)
                            .rounding(4.0_f32)
                            .stroke(egui::Stroke::new(1.0_f32, Theme::BORDER))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label(RichText::new("Desktop Launcher (.desktop)").strong().color(Theme::PRIMARY));
                                        ui.label(RichText::new(desk.to_string_lossy()).color(Theme::TEXT_SECONDARY));
                                    });

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button("📂 Open").on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                            if let Some(parent) = desk.parent() {
                                                let _ = open::that(parent);
                                            }
                                        }
                                    });
                                });
                            });
                    }

                    ui.add_space(12.0_f32);
                    ui.separator();
                    ui.add_space(8.0_f32);

                    // ==========================================
                    // 2. DISCOVERED USER ARTIFACTS, CACHES & SANDBOXES
                    // ==========================================
                    ui.heading(format!("📂 Discovered User Artifacts & Caches ({})", residuals.len()));
                    ui.add_space(4.0_f32);

                    if residuals.is_empty() {
                        ui.label(RichText::new("No additional user caches, configs, or sandbox paths discovered.").color(Theme::TEXT_MUTED));
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
