use crate::models::{format_size, Application, InstallMethod};
use crate::ui::theme::Theme;
use egui::{Color32, RichText, ScrollArea, Stroke, Ui};
use std::collections::HashSet;

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
        selected_ids: &mut HashSet<String>,
        focus_search: &mut bool,
        is_scanning: bool,
        scan_status: Option<&str>,
        on_inspect: &mut Option<Application>,
        on_clean: &mut Option<Application>,
    ) {
        Self::render_metrics_header(ui, apps, *show_system_packages);
        ui.add_space(10.0_f32);

        // Live Scanning Progress Banner with animated glowing spinner
        if is_scanning {
            egui::Frame::none()
                .fill(Theme::BG_DARK)
                .inner_margin(10.0_f32)
                .rounding(6.0_f32)
                .stroke(Stroke::new(1.0_f32, Theme::PRIMARY))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.add_space(8.0_f32);
                        let status = scan_status.unwrap_or("Scanning packaging tiers...");
                        ui.label(
                            RichText::new(format!("⚡ Live Scan: {} — {} applications discovered", status, apps.len()))
                                .color(Theme::ACCENT_CYAN)
                                .strong()
                                .size(13.0_f32),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new("Populating in real-time").color(Theme::TEXT_MUTED).size(11.0_f32));
                        });
                    });
                });
            ui.add_space(8.0_f32);
        }

        // 1. Search Bar & Filter Chips
        ui.horizontal(|ui| {
            ui.label(RichText::new("🔍").size(15.0_f32));
            let search_edit = egui::TextEdit::singleline(search_query)
                .hint_text("Search apps by name, ID, or binary... (Ctrl+F)")
                .desired_width(260.0_f32);
            let response = ui.add(search_edit);
            if *focus_search {
                response.request_focus();
                *focus_search = false;
            }

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
                ui.checkbox(show_system_packages, RichText::new("Show System Libraries").size(12.0_f32));
            });
        });

        ui.add_space(6.0_f32);

        // 2. Sort Controls Row
        ui.horizontal(|ui| {
            ui.label(RichText::new("Sort by:").size(12.0_f32).color(Theme::TEXT_MUTED));
            ui.add_space(4.0_f32);

            let sort_options = [
                ("Size (Largest First)", SortField::Size, SortDirection::Descending),
                ("Size (Smallest First)", SortField::Size, SortDirection::Ascending),
                ("Name (A → Z)", SortField::Name, SortDirection::Ascending),
                ("Name (Z → A)", SortField::Name, SortDirection::Descending),
                ("Origin / Packaging Tier", SortField::Origin, SortDirection::Ascending),
            ];

            for (label, field, dir) in sort_options {
                let is_active = *sort_field == field && *sort_direction == dir;
                if Self::render_sort_button(ui, label, is_active) {
                    *sort_field = field;
                    *sort_direction = dir;
                }
            }
        });

        ui.add_space(8.0_f32);

        // 3. Selection Calculation Banner (if 1 or more apps are selected)
        if !selected_ids.is_empty() {
            let selected_apps: Vec<&Application> = apps.iter().filter(|a| selected_ids.contains(&a.id)).collect();
            let selected_count = selected_apps.len();
            let selected_size: u64 = selected_apps.iter().map(|a| a.total_size_bytes).sum();

            egui::Frame::none()
                .fill(Theme::PRIMARY_MUTED)
                .inner_margin(8.0_f32)
                .rounding(6.0_f32)
                .stroke(Stroke::new(1.0_f32, Theme::PRIMARY))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("📦").size(14.0_f32));
                        ui.label(
                            RichText::new(format!(
                                "Selected: {} application{}  |  Combined Storage Footprint: {}",
                                selected_count,
                                if selected_count == 1 { "" } else { "s" },
                                format_size(selected_size)
                            ))
                            .color(Color32::WHITE)
                            .strong(),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .button(RichText::new("✕ Clear Selection").color(Theme::TEXT_PRIMARY).size(11.0_f32))
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .clicked()
                            {
                                selected_ids.clear();
                            }
                        });
                    });
                });
            ui.add_space(8.0_f32);
        }

        // 4. Filter and Sort Applications
        let mut visible_apps: Vec<Application> = apps
            .iter()
            .filter(|app| {
                if !*show_system_packages && app.is_system {
                    return false;
                }

                match selected_filter {
                    UiCategoryFilter::All => true,
                    UiCategoryFilter::Native => app.install_method.is_native_system(),
                    UiCategoryFilter::Flatpak => app.install_method == InstallMethod::Flatpak,
                    UiCategoryFilter::Snap => app.install_method == InstallMethod::Snap,
                    UiCategoryFilter::AppImage => app.install_method == InstallMethod::AppImage,
                    UiCategoryFilter::Manual => app.install_method == InstallMethod::ManualOpt,
                }
            })
            .filter(|app| {
                if search_query.trim().is_empty() {
                    return true;
                }
                let q = search_query.to_lowercase();
                app.name.to_lowercase().contains(&q)
                    || app.display_name.to_lowercase().contains(&q)
                    || app.id.to_lowercase().contains(&q)
                    || app
                        .exec_path
                        .as_ref()
                        .map_or(false, |p| p.to_string_lossy().to_lowercase().contains(&q))
            })
            .cloned()
            .collect();

        match sort_field {
            SortField::Name => {
                visible_apps.sort_by(|a, b| match sort_direction {
                    SortDirection::Ascending => a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase()),
                    SortDirection::Descending => b.display_name.to_lowercase().cmp(&a.display_name.to_lowercase()),
                });
            }
            SortField::Size => {
                visible_apps.sort_by(|a, b| match sort_direction {
                    SortDirection::Ascending => a.total_size_bytes.cmp(&b.total_size_bytes),
                    SortDirection::Descending => b.total_size_bytes.cmp(&a.total_size_bytes),
                });
            }
            SortField::Origin => {
                visible_apps.sort_by(|a, b| match sort_direction {
                    SortDirection::Ascending => a.install_method.badge_label().cmp(b.install_method.badge_label()),
                    SortDirection::Descending => b.install_method.badge_label().cmp(a.install_method.badge_label()),
                });
            }
            SortField::Version => {
                visible_apps.sort_by(|a, b| match sort_direction {
                    SortDirection::Ascending => a.version.cmp(&b.version),
                    SortDirection::Descending => b.version.cmp(&a.version),
                });
            }
        }

        // 5. Table Header
        egui::Frame::none()
            .fill(Theme::BG_DARK)
            .inner_margin(8.0_f32)
            .rounding(4.0_f32)
            .show(ui, |ui| {
                ui.columns(6, |cols| {
                    cols[0].set_width(32.0_f32);
                    let all_visible_selected = !visible_apps.is_empty() && visible_apps.iter().all(|a| selected_ids.contains(&a.id));
                    let mut select_all_state = all_visible_selected;
                    if cols[0].checkbox(&mut select_all_state, "").changed() {
                        if select_all_state {
                            for a in &visible_apps {
                                selected_ids.insert(a.id.clone());
                            }
                        } else {
                            for a in &visible_apps {
                                selected_ids.remove(&a.id);
                            }
                        }
                    }

                    cols[1].set_width(220.0_f32);
                    if Self::table_header_btn(&mut cols[1], "Application", *sort_field == SortField::Name, *sort_direction) {
                        Self::toggle_sort(sort_field, sort_direction, SortField::Name);
                    }

                    cols[2].set_width(90.0_f32);
                    if Self::table_header_btn(&mut cols[2], "Origin", *sort_field == SortField::Origin, *sort_direction) {
                        Self::toggle_sort(sort_field, sort_direction, SortField::Origin);
                    }

                    cols[3].set_width(120.0_f32);
                    if Self::table_header_btn(&mut cols[3], "Version", *sort_field == SortField::Version, *sort_direction) {
                        Self::toggle_sort(sort_field, sort_direction, SortField::Version);
                    }

                    cols[4].set_width(100.0_f32);
                    if Self::table_header_btn(&mut cols[4], "Disk Size", *sort_field == SortField::Size, *sort_direction) {
                        Self::toggle_sort(sort_field, sort_direction, SortField::Size);
                    }

                    cols[5].label(RichText::new("Actions").strong().color(Theme::TEXT_MUTED));
                });
            });

        ui.add_space(4.0_f32);

        // 6. Application Rows
        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            if visible_apps.is_empty() {
                ui.add_space(32.0_f32);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("No matching applications found.").size(14.0_f32).color(Theme::TEXT_MUTED));
                });
                return;
            }

            for app in &visible_apps {
                let is_checked = selected_ids.contains(&app.id);
                let row_bg = if is_checked {
                    Theme::PRIMARY_MUTED
                } else {
                    Theme::BG_CARD
                };

                egui::Frame::none()
                    .fill(row_bg)
                    .inner_margin(8.0_f32)
                    .rounding(6.0_f32)
                    .stroke(Stroke::new(1.0_f32, if is_checked { Theme::PRIMARY } else { Theme::BORDER }))
                    .show(ui, |ui| {
                        ui.columns(6, |cols| {
                            // Column 0: Selection Checkbox
                            cols[0].set_width(32.0_f32);
                            cols[0].vertical(|ui| {
                                let mut checked = is_checked;
                                if ui.checkbox(&mut checked, "").changed() {
                                    if checked {
                                        selected_ids.insert(app.id.clone());
                                    } else {
                                        selected_ids.remove(&app.id);
                                    }
                                }
                            });

                            // Column 1: Name & ID
                            cols[1].set_width(220.0_f32);
                            cols[1].vertical(|ui| {
                                ui.label(RichText::new(&app.display_name).strong().size(13.0_f32));
                                ui.label(RichText::new(&app.id).size(11.0_f32).color(Theme::TEXT_MUTED));
                            });

                            // Column 2: Origin Badge
                            cols[2].set_width(90.0_f32);
                            cols[2].vertical(|ui| {
                                let (badge_bg, text_color, label_text) = match app.install_method {
                                    InstallMethod::NativeApt => (Color32::from_rgb(30, 58, 43), Theme::ACCENT_SUCCESS, "APT"),
                                    InstallMethod::NativeDnf => (Color32::from_rgb(18, 53, 60), Theme::ACCENT_CYAN, "DNF"),
                                    InstallMethod::NativePacman => (Color32::from_rgb(45, 30, 60), Color32::from_rgb(216, 180, 254), "Pacman"),
                                    InstallMethod::Flatpak => (Color32::from_rgb(15, 45, 82), Color32::from_rgb(147, 197, 253), "Flatpak"),
                                    InstallMethod::Snap => (Color32::from_rgb(60, 35, 20), Color32::from_rgb(253, 186, 116), "Snap"),
                                    InstallMethod::AppImage => (Color32::from_rgb(50, 45, 20), Color32::from_rgb(254, 240, 138), "AppImage"),
                                    InstallMethod::ManualOpt => (Color32::from_rgb(40, 40, 45), Theme::TEXT_SECONDARY, "Manual (/opt)"),
                                    InstallMethod::CustomDesktop => (Color32::from_rgb(35, 35, 40), Theme::TEXT_MUTED, "Desktop Entry"),
                                };

                                egui::Frame::none()
                                    .fill(badge_bg)
                                    .inner_margin(egui::vec2(6.0_f32, 2.0_f32))
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
                            cols[3].set_width(120.0_f32);
                            cols[3].vertical(|ui| {
                                ui.label(
                                    RichText::new(app.version.as_deref().unwrap_or("-"))
                                        .size(12.0_f32)
                                        .color(Theme::TEXT_SECONDARY),
                                );
                            });

                            // Column 4: Size
                            cols[4].set_width(100.0_f32);
                            cols[4].vertical(|ui| {
                                ui.label(
                                    RichText::new(app.formatted_size())
                                        .size(13.0_f32)
                                        .strong()
                                        .color(Theme::TEXT_PRIMARY),
                                );
                            });

                            // Column 5: Actions
                            cols[5].horizontal(|ui| {
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
                SortField::Size => SortDirection::Descending,
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
