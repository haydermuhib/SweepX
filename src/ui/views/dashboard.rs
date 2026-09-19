use crate::models::{format_size, Application, InstallMethod};
use crate::ui::theme::Theme;
use egui::{Color32, RichText, ScrollArea, Stroke, Ui};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiCategoryFilter {
    All,
    Native,
    Flatpak,
    Snap,
    AppImage,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Name,
    Size,
    Origin,
    Version,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

pub struct DashboardView;

impl DashboardView {
    pub fn render(
        ui: &mut Ui,
        apps: &[Application],
        search_query: &mut String,
        selected_filter: &mut UiCategoryFilter,
        sort_field: &mut SortField,
        sort_direction: &mut SortDirection,
        show_system_packages: &mut bool,
        on_inspect: &mut Option<Application>,
        on_clean: &mut Option<Application>,
    ) {
        Self::render_metrics_header(ui, apps, *show_system_packages);
        ui.add_space(14.0_f32);

        // 1. Search Bar & Filter Chips
        ui.horizontal(|ui| {
            ui.label(RichText::new("🔍").size(15.0_f32));
            ui.add(
                egui::TextEdit::singleline(search_query)
                    .hint_text("Search apps by name, ID, or binary...")
                    .desired_width(240.0_f32),
            );

            ui.add_space(10.0_f32);
            ui.label(RichText::new("Filter:").strong().color(Theme::TEXT_MUTED));

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
                if Self::render_pill_button(ui, label, is_selected) {
                    *selected_filter = filter_type;
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.checkbox(show_system_packages, "Show System Libraries");
            });
        });

        ui.add_space(10.0_f32);

        // 2. Sorting Toolbar
        ui.horizontal(|ui| {
            ui.label(RichText::new("Sort by:").strong().color(Theme::TEXT_MUTED));

            let sort_options = [
                (SortField::Size, SortDirection::Descending, "Size (Largest First)"),
                (SortField::Size, SortDirection::Ascending, "Size (Smallest First)"),
                (SortField::Name, SortDirection::Ascending, "Name (A → Z)"),
                (SortField::Name, SortDirection::Descending, "Name (Z → A)"),
                (SortField::Origin, SortDirection::Ascending, "Origin / Packaging Tier"),
            ];

            for (field, dir, label) in sort_options {
                let is_active = *sort_field == field && *sort_direction == dir;
                if Self::render_sort_button(ui, label, is_active) {
                    *sort_field = field;
                    *sort_direction = dir;
                }
            }
        });

        ui.add_space(10.0_f32);
        ui.separator();
        ui.add_space(8.0_f32);

        // 3. Filter & Sort Application Records
        let query_lower = search_query.to_lowercase();
        let mut filtered_apps: Vec<&Application> = apps
            .iter()
            .filter(|app| {
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

        // Sort records
        filtered_apps.sort_by(|a, b| {
            let ord = match sort_field {
                SortField::Name => a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase()),
                SortField::Size => a.total_size_bytes.cmp(&b.total_size_bytes),
                SortField::Origin => a.install_method.badge_label().cmp(b.install_method.badge_label()),
                SortField::Version => a.version.cmp(&b.version),
            };

            match sort_direction {
                SortDirection::Ascending => ord,
                SortDirection::Descending => ord.reverse(),
            }
        });

        // 4. Interactive Table Header with Click-to-Sort
        egui::Frame::none()
            .fill(Theme::BG_CARD)
            .inner_margin(8.0_f32)
            .rounding(6.0_f32)
            .stroke(Stroke::new(1.0_f32, Theme::BORDER))
            .show(ui, |ui| {
                ui.columns(5, |cols| {
                    if Self::table_header_btn(&mut cols[0], "Application", *sort_field == SortField::Name, *sort_direction) {
                        Self::toggle_sort(sort_field, sort_direction, SortField::Name);
                    }
                    if Self::table_header_btn(&mut cols[1], "Origin", *sort_field == SortField::Origin, *sort_direction) {
                        Self::toggle_sort(sort_field, sort_direction, SortField::Origin);
                    }
                    if Self::table_header_btn(&mut cols[2], "Version", *sort_field == SortField::Version, *sort_direction) {
                        Self::toggle_sort(sort_field, sort_direction, SortField::Version);
                    }
                    if Self::table_header_btn(&mut cols[3], "Disk Size", *sort_field == SortField::Size, *sort_direction) {
                        Self::toggle_sort(sort_field, sort_direction, SortField::Size);
                    }
                    cols[4].label(RichText::new("Actions").strong().color(Theme::TEXT_MUTED));
                });
            });

        ui.add_space(4.0_f32);

        // 5. Table Rows
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
                    .stroke(Stroke::new(1.0_f32, Theme::BORDER))
                    .show(ui, |ui| {
                        ui.columns(5, |cols| {
                            // Column 1: Application Name & ID
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

                            // Column 2: Packaging Origin Badge
                            cols[1].vertical(|ui| {
                                let (badge_color, text_color) = match app.install_method {
                                    InstallMethod::Flatpak => {
                                        if app.is_system {
                                            (Color32::from_rgb(59, 130, 246), Color32::WHITE)
                                        } else {
                                            (Color32::from_rgb(37, 99, 235), Color32::WHITE)
                                        }
                                    }
                                    InstallMethod::Snap => {
                                        if app.is_system {
                                            (Color32::from_rgb(146, 64, 14), Color32::WHITE)
                                        } else {
                                            (Color32::from_rgb(217, 119, 6), Color32::WHITE)
                                        }
                                    }
                                    InstallMethod::NativeApt | InstallMethod::NativeDnf | InstallMethod::NativePacman => {
                                        if app.is_system {
                                            (Color32::from_rgb(100, 116, 139), Color32::WHITE)
                                        } else {
                                            (Color32::from_rgb(16, 185, 129), Color32::WHITE)
                                        }
                                    }
                                    InstallMethod::AppImage => (Color32::from_rgb(147, 51, 234), Color32::WHITE),
                                    _ => (Color32::from_rgb(75, 85, 99), Color32::WHITE),
                                };

                                let label_text = if app.is_system {
                                    if app.install_method == InstallMethod::Snap {
                                        "Snap (Runtime)".to_string()
                                    } else {
                                        format!("{} (System)", app.install_method.badge_label())
                                    }
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

                            // Column 3: Version
                            cols[2].vertical(|ui| {
                                ui.label(
                                    RichText::new(app.version.as_deref().unwrap_or("-"))
                                        .size(12.0_f32)
                                        .color(Theme::TEXT_SECONDARY),
                                );
                            });

                            // Column 4: Size
                            cols[3].vertical(|ui| {
                                ui.label(
                                    RichText::new(app.formatted_size())
                                        .size(13.0_f32)
                                        .strong()
                                        .color(Theme::TEXT_PRIMARY),
                                );
                            });

                            // Column 5: Actions
                            cols[4].horizontal(|ui| {
                                if ui
                                    .button(RichText::new("🔍 Inspect").size(12.0_f32))
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    *on_inspect = Some(app.clone());
                                }
                                if ui
                                    .button(
                                        RichText::new("🗑️ Deep Clean")
                                            .size(12.0_f32)
                                            .color(Theme::ACCENT_DANGER),
                                    )
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
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

    /// Renders a modern high-contrast pill button for category filtering.
    fn render_pill_button(ui: &mut Ui, label: &str, is_selected: bool) -> bool {
        let (bg_color, text_color, border_stroke) = if is_selected {
            (Theme::PRIMARY, Color32::WHITE, Stroke::new(1.0_f32, Theme::PRIMARY_HOVER))
        } else {
            (Theme::BG_CARD, Theme::TEXT_SECONDARY, Stroke::new(1.0_f32, Theme::BORDER))
        };

        let text = if is_selected {
            RichText::new(label).color(text_color).strong().size(12.0_f32)
        } else {
            RichText::new(label).color(text_color).size(12.0_f32)
        };

        let btn = egui::Button::new(text)
            .fill(bg_color)
            .stroke(border_stroke)
            .rounding(14.0_f32)
            .min_size(egui::vec2(0.0_f32, 24.0_f32));

        ui.add(btn)
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
    }

    /// Renders a small high-contrast sort button chip.
    fn render_sort_button(ui: &mut Ui, label: &str, is_active: bool) -> bool {
        let (bg_color, text_color, border_stroke) = if is_active {
            (Theme::PRIMARY_MUTED, Color32::WHITE, Stroke::new(1.0_f32, Theme::PRIMARY))
        } else {
            (Theme::BG_DARK, Theme::TEXT_MUTED, Stroke::new(1.0_f32, Theme::BORDER))
        };

        let text = if is_active {
            RichText::new(label).color(text_color).strong().size(11.0_f32)
        } else {
            RichText::new(label).color(text_color).size(11.0_f32)
        };

        let btn = egui::Button::new(text)
            .fill(bg_color)
            .stroke(border_stroke)
            .rounding(6.0_f32)
            .min_size(egui::vec2(0.0_f32, 22.0_f32));

        ui.add(btn)
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
    }

    fn table_header_btn(ui: &mut Ui, label: &str, is_sorted: bool, dir: SortDirection) -> bool {
        let text = if is_sorted {
            let arrow = match dir {
                SortDirection::Ascending => " ▲",
                SortDirection::Descending => " ▼",
            };
            RichText::new(format!("{}{}", label, arrow))
                .strong()
                .color(Theme::PRIMARY_HOVER)
        } else {
            RichText::new(label).strong().color(Theme::TEXT_MUTED)
        };

        ui.add(egui::Button::new(text).fill(Color32::TRANSPARENT).stroke(Stroke::NONE))
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
    }

    fn toggle_sort(field: &mut SortField, dir: &mut SortDirection, target_field: SortField) {
        if *field == target_field {
            *dir = match *dir {
                SortDirection::Ascending => SortDirection::Descending,
                SortDirection::Descending => SortDirection::Ascending,
            };
        } else {
            *field = target_field;
            *dir = match target_field {
                SortField::Size => SortDirection::Descending, // Default largest first
                _ => SortDirection::Ascending,
            };
        }
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
        let snaps = visible_apps.iter().filter(|a| a.install_method == InstallMethod::Snap).count();
        let native_user = visible_apps.iter().filter(|a| a.install_method.is_native_system() && !a.is_system).count();

        ui.horizontal(|ui| {
            Self::metric_card(ui, "Installed User Software", &format!("{}", total_apps), "Applications discovered", Theme::PRIMARY);
            ui.add_space(12.0_f32);

            Self::metric_card(ui, "Storage Footprint", &format_size(total_size), "Estimated disk usage", Theme::ACCENT_SUCCESS);
            ui.add_space(12.0_f32);

            Self::metric_card(
                ui,
                "Package Breakdown",
                &format!("{} Native • {} Flatpak • {} Snap", native_user, flatpaks, snaps),
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
