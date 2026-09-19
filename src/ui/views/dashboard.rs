use crate::models::{format_size, Application, InstallMethod};
use crate::ui::theme::Theme;
use egui::{Color32, RichText, ScrollArea, Ui};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiCategoryFilter {
    All,
    Native,
    Flatpak,
    Snap,
    AppImage,
    Manual,
}

pub struct DashboardView;

impl DashboardView {
    pub fn render(
        ui: &mut Ui,
        apps: &[Application],
        search_query: &mut String,
        selected_filter: &mut UiCategoryFilter,
        show_system_packages: &mut bool,
        on_inspect: &mut Option<Application>,
        on_clean: &mut Option<Application>,
    ) {
        Self::render_metrics_header(ui, apps, *show_system_packages);
        ui.add_space(16.0_f32);

        ui.horizontal(|ui| {
            ui.label(RichText::new("🔍").size(16.0_f32));
            ui.add(
                egui::TextEdit::singleline(search_query)
                    .hint_text("Search by name, ID, or executable...")
                    .desired_width(260.0_f32),
            );

            ui.add_space(12.0_f32);
            ui.label(RichText::new("Filter:").color(Theme::TEXT_MUTED));

            let filters = [
                (UiCategoryFilter::All, "All"),
                (UiCategoryFilter::Native, "Native (APT/DNF/Pacman)"),
                (UiCategoryFilter::Flatpak, "Flatpak"),
                (UiCategoryFilter::Snap, "Snap"),
                (UiCategoryFilter::AppImage, "AppImage"),
                (UiCategoryFilter::Manual, "Manual /opt"),
            ];

            for (filter_type, label) in filters {
                let is_selected = *selected_filter == filter_type;
                let text = if is_selected {
                    RichText::new(label).color(Theme::PRIMARY).strong()
                } else {
                    RichText::new(label).color(Theme::TEXT_SECONDARY)
                };

                if ui.selectable_label(is_selected, text).clicked() {
                    *selected_filter = filter_type;
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.checkbox(show_system_packages, "Show System Libraries");
            });
        });

        ui.add_space(12.0_f32);
        ui.separator();
        ui.add_space(8.0_f32);

        let query_lower = search_query.to_lowercase();
        let filtered_apps: Vec<&Application> = apps
            .iter()
            .filter(|app| {
                // System filter check
                if !*show_system_packages && app.is_system {
                    return false;
                }

                let matches_cat = match selected_filter {
                    UiCategoryFilter::All => true,
                    UiCategoryFilter::Native => app.install_method.is_native_system(),
                    UiCategoryFilter::Flatpak => app.install_method == InstallMethod::Flatpak,
                    UiCategoryFilter::Snap => app.install_method == InstallMethod::Snap,
                    UiCategoryFilter::AppImage => app.install_method == InstallMethod::AppImage,
                    UiCategoryFilter::Manual => {
                        app.install_method == InstallMethod::ManualOpt
                            || app.install_method == InstallMethod::CustomDesktop
                    }
                };

                let matches_search = query_lower.is_empty()
                    || app.name.to_lowercase().contains(&query_lower)
                    || app.display_name.to_lowercase().contains(&query_lower)
                    || app.id.to_lowercase().contains(&query_lower);

                matches_cat && matches_search
            })
            .collect();

        // Table Header
        egui::Frame::none()
            .fill(Theme::BG_CARD)
            .inner_margin(8.0_f32)
            .rounding(6.0_f32)
            .show(ui, |ui| {
                ui.columns(5, |cols| {
                    cols[0].label(RichText::new("Application").strong().color(Theme::TEXT_MUTED));
                    cols[1].label(RichText::new("Origin").strong().color(Theme::TEXT_MUTED));
                    cols[2].label(RichText::new("Version").strong().color(Theme::TEXT_MUTED));
                    cols[3].label(RichText::new("Disk Size").strong().color(Theme::TEXT_MUTED));
                    cols[4].label(RichText::new("Actions").strong().color(Theme::TEXT_MUTED));
                });
            });

        ui.add_space(4.0_f32);

        ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            if filtered_apps.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0_f32);
                    ui.label(
                        RichText::new("No applications found matching the selected criteria.")
                            .color(Theme::TEXT_MUTED)
                            .size(15.0_f32),
                    );
                });
                return;
            }

            for app in filtered_apps {
                egui::Frame::none()
                    .fill(Theme::BG_CARD)
                    .inner_margin(10.0_f32)
                    .rounding(6.0_f32)
                    .stroke(egui::Stroke::new(1.0_f32, Theme::BORDER))
                    .show(ui, |ui| {
                        ui.columns(5, |cols| {
                            cols[0].vertical(|ui| {
                                ui.label(
                                    RichText::new(&app.display_name)
                                        .strong()
                                        .size(14.0_f32)
                                        .color(Theme::TEXT_PRIMARY),
                                );
                                ui.label(
                                    RichText::new(&app.id)
                                        .size(11.0_f32)
                                        .color(Theme::TEXT_MUTED),
                                );
                            });

                            cols[1].vertical(|ui| {
                                let (badge_color, text_color) = match app.install_method {
                                    InstallMethod::Flatpak => (Color32::from_rgb(37, 99, 235), Color32::WHITE),
                                    InstallMethod::Snap => (Color32::from_rgb(217, 119, 6), Color32::WHITE),
                                    InstallMethod::NativeApt | InstallMethod::NativeDnf | InstallMethod::NativePacman => {
                                        if app.is_system {
                                            (Color32::from_rgb(107, 114, 128), Color32::WHITE)
                                        } else {
                                            (Color32::from_rgb(16, 185, 129), Color32::WHITE)
                                        }
                                    }
                                    InstallMethod::AppImage => (Color32::from_rgb(147, 51, 234), Color32::WHITE),
                                    _ => (Color32::from_rgb(75, 85, 99), Color32::WHITE),
                                };

                                let label_text = if app.is_system {
                                    format!("{} (System)", app.install_method.badge_label())
                                } else {
                                    app.install_method.badge_label().to_string()
                                };

                                egui::Frame::none()
                                    .fill(badge_color)
                                    .inner_margin(egui::Margin::symmetric(6.0_f32, 2.0_f32))
                                    .rounding(4.0_f32)
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new(label_text)
                                                .size(11.0_f32)
                                                .color(text_color)
                                                .strong(),
                                        );
                                    });
                            });

                            cols[2].vertical(|ui| {
                                ui.label(
                                    RichText::new(app.version.as_deref().unwrap_or("-"))
                                        .size(12.0_f32)
                                        .color(Theme::TEXT_SECONDARY),
                                );
                            });

                            cols[3].vertical(|ui| {
                                ui.label(
                                    RichText::new(app.formatted_size())
                                        .size(13.0_f32)
                                        .strong()
                                        .color(Theme::TEXT_PRIMARY),
                                );
                            });

                            cols[4].horizontal(|ui| {
                                if ui.button(RichText::new("🔍 Inspect").size(12.0_f32)).clicked() {
                                    *on_inspect = Some(app.clone());
                                }
                                if ui
                                    .button(
                                        RichText::new("🗑️ Deep Clean")
                                            .size(12.0_f32)
                                            .color(Theme::ACCENT_DANGER),
                                    )
                                    .clicked()
                                {
                                    *on_clean = Some(app.clone());
                                }
                            });
                        });
                    });
                ui.add_space(4.0_f32);
            }
        });
    }

    fn render_metrics_header(ui: &mut Ui, apps: &[Application], show_system: bool) {
        let visible_apps: Vec<&Application> = if show_system {
            apps.iter().collect()
        } else {
            apps.iter().filter(|a| !a.is_system).collect()
        };

        let total_apps = visible_apps.len();
        let total_size: u64 = visible_apps.iter().map(|a| a.total_size_bytes).sum();
        let flatpaks = visible_apps.iter().filter(|a| a.install_method == InstallMethod::Flatpak).count();
        let native_user = visible_apps.iter().filter(|a| a.install_method.is_native_system() && !a.is_system).count();

        ui.horizontal(|ui| {
            Self::metric_card(ui, "Installed User Software", &format!("{}", total_apps), "Applications discovered", Theme::PRIMARY);
            ui.add_space(12.0_f32);

            Self::metric_card(ui, "Storage Footprint", &format_size(total_size), "Estimated disk usage", Theme::ACCENT_SUCCESS);
            ui.add_space(12.0_f32);

            Self::metric_card(
                ui,
                "Package Breakdown",
                &format!("{} Native • {} Flatpak", native_user, flatpaks),
                "User-installed applications",
                Theme::ACCENT_CYAN,
            );
        });
    }

    fn metric_card(ui: &mut Ui, title: &str, value: &str, subtitle: &str, accent: Color32) {
        egui::Frame::none()
            .fill(Theme::BG_CARD)
            .inner_margin(12.0_f32)
            .rounding(8.0_f32)
            .stroke(egui::Stroke::new(1.0_f32, Theme::BORDER))
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
