use crate::cleaner::{
    discover_residuals_for_app, execute_purge_package, execute_purge_residuals,
    execute_system_sweep, scan_all_sweep_items, DeletionMode, SystemSweepItem,
};
use crate::db::{AuditLogEntry, Database};
use crate::models::{
    format_size_with_unit, Application, InstallMethod, ResidualCandidate, SizeUnitSystem,
};
use crate::scanner::scan_all_applications;
use crate::ui::theme::Theme;

use iced::widget::{
    button, checkbox, column, container, row, scrollable, stack, text, text_input, Space,
};
use iced::{Alignment, Element, Length, Task};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationTab {
    Applications,
    SystemSweep,
    AuditHistory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategoryFilter {
    All,
    Native,
    Flatpak,
    Snap,
    Bottles,
    AppImage,
    Manual,
}

impl CategoryFilter {
    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "All Apps",
            Self::Native => "Native (APT/DNF/Pacman)",
            Self::Flatpak => "Flatpak",
            Self::Snap => "Snap",
            Self::Bottles => "Bottles",
            Self::AppImage => "AppImage",
            Self::Manual => "Manual / /opt",
        }
    }

    pub fn matches(&self, method: &InstallMethod) -> bool {
        match self {
            Self::All => true,
            Self::Native => method.is_native_system() || *method == InstallMethod::CustomDesktop,
            Self::Flatpak => *method == InstallMethod::Flatpak,
            Self::Snap => *method == InstallMethod::Snap,
            Self::Bottles => *method == InstallMethod::Bottles,
            Self::AppImage => *method == InstallMethod::AppImage,
            Self::Manual => *method == InstallMethod::ManualOpt,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Name,
    Size,
    Footprint,
    Origin,
}

impl SortField {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Name => "Name",
            Self::Size => "Payload",
            Self::Footprint => "Total Footprint",
            Self::Origin => "Origin",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

pub struct SweepXApp {
    pub active_tab: NavigationTab,
    pub applications: Vec<Application>,
    pub is_scanning: bool,
    pub search_query: String,
    pub category_filter: CategoryFilter,
    pub sort_field: SortField,
    pub sort_direction: SortDirection,
    pub show_system_libraries: bool,
    pub unit_system: SizeUnitSystem,

    // Inspector Modal
    pub inspector_app: Option<Application>,
    pub inspector_residuals: Vec<ResidualCandidate>,
    pub is_inspector_loading: bool,
    pub app_residuals_cache: HashMap<String, u64>,
    pub in_flight_residuals: HashSet<String>,

    // Clean Modal
    pub clean_target: Option<Application>,
    pub clean_residuals: Vec<(ResidualCandidate, bool)>,
    pub is_clean_loading: bool,
    pub permanent_delete: bool,
    pub is_cleaning: bool,
    pub purge_package: bool,

    // System Sweep
    pub sweep_items: Vec<(SystemSweepItem, bool)>,
    pub is_sweeping: bool,
    pub is_sweep_scanning: bool,

    // Audit History
    pub audit_history: Vec<AuditLogEntry>,
    pub inspector_audit_entry: Option<AuditLogEntry>,

    // Notification / Status
    pub status_message: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SwitchTab(NavigationTab),
    RefreshScan,
    ScanCompleted(Vec<Application>),
    SearchChanged(String),
    FilterCategory(CategoryFilter),
    SelectSort(SortField),
    ToggleShowSystemLibraries(bool),
    ToggleUnitSystem,

    // Inspector & Footprint Cache
    OpenInspector(Application),
    InspectorResidualsLoaded(Vec<ResidualCandidate>),
    ResidualsBatchLoaded(Vec<(String, u64)>),
    CloseInspector,

    // Clean Modal
    OpenCleanModal(Application),
    CleanResidualsLoaded(Vec<ResidualCandidate>),
    ToggleCleanResidual(usize, bool),
    TogglePurgePackage(bool),
    TogglePermanentDelete(bool),
    ConfirmPurge,
    PurgeCompleted(Result<u64, String>),
    CloseCleanModal,

    // System Sweep
    LoadSweepItems,
    SweepItemsLoaded(Vec<SystemSweepItem>),
    ToggleSweepItem(usize, bool),
    RunSystemSweep,
    SystemSweepCompleted(Result<(usize, u64), String>),

    // Audit History
    LoadAuditHistory,
    AuditHistoryLoaded(Vec<AuditLogEntry>),
    OpenAuditInspector(AuditLogEntry),
    CloseAuditInspector,

    // System Actions
    OpenFolder(PathBuf),
    CopyToClipboard(String),
    ClearStatus,
}

impl SweepXApp {
    pub fn new() -> (Self, Task<Message>) {
        let app = Self {
            active_tab: NavigationTab::Applications,
            applications: Vec::new(),
            is_scanning: true,
            search_query: String::new(),
            category_filter: CategoryFilter::All,
            sort_field: SortField::Name,
            sort_direction: SortDirection::Ascending,
            show_system_libraries: false,
            unit_system: SizeUnitSystem::GiB,

            inspector_app: None,
            inspector_residuals: Vec::new(),
            is_inspector_loading: false,
            app_residuals_cache: HashMap::new(),
            in_flight_residuals: HashSet::new(),

            clean_target: None,
            clean_residuals: Vec::new(),
            is_clean_loading: false,
            permanent_delete: false,
            is_cleaning: false,
            purge_package: true,

            sweep_items: Vec::new(),
            is_sweeping: false,
            is_sweep_scanning: false,

            audit_history: Vec::new(),
            inspector_audit_entry: None,
            status_message: Some("Initializing SweepX...".to_string()),
        };

        // Kick off initial asynchronous host scan
        let scan_task = Task::perform(scan_all_applications(), Message::ScanCompleted);

        (app, scan_task)
    }

    pub fn format_bytes(&self, bytes: u64) -> String {
        format_size_with_unit(bytes, self.unit_system)
    }

    pub fn title(&self) -> String {
        "SweepX — Unified Linux Application Tracker & Deep Purge".to_string()
    }

    pub fn select_sort(&mut self, field: SortField) {
        if self.sort_field == field {
            self.sort_direction = match self.sort_direction {
                SortDirection::Ascending => SortDirection::Descending,
                SortDirection::Descending => SortDirection::Ascending,
            };
        } else {
            self.sort_field = field;
            self.sort_direction = match field {
                SortField::Size | SortField::Footprint => SortDirection::Descending,
                _ => SortDirection::Ascending,
            };
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SwitchTab(tab) => {
                self.active_tab = tab;
                match tab {
                    NavigationTab::Applications => {
                        if self.applications.is_empty() && !self.is_scanning {
                            return Task::perform(scan_all_applications(), Message::ScanCompleted);
                        }
                    }
                    NavigationTab::SystemSweep => {
                        if self.sweep_items.is_empty() && !self.is_sweep_scanning {
                            self.is_sweep_scanning = true;
                            return Task::perform(scan_all_sweep_items(), Message::SweepItemsLoaded);
                        }
                    }
                    NavigationTab::AuditHistory => {
                        return Task::perform(
                            async {
                                if let Ok(db) = Database::open_default() {
                                    db.get_audit_history().unwrap_or_default()
                                } else {
                                    Vec::new()
                                }
                            },
                            Message::AuditHistoryLoaded,
                        );
                    }
                }
                Task::none()
            }
            Message::RefreshScan => {
                self.is_scanning = true;
                self.status_message = Some("Scanning applications across all packaging tiers...".to_string());
                Task::perform(scan_all_applications(), Message::ScanCompleted)
            }
            Message::ScanCompleted(apps) => {
                let count = apps.len();
                self.applications = apps;
                self.is_scanning = false;
                self.status_message = Some(format!("Discovered {} installed host applications", count));
                Task::none()
            }
            Message::ResidualsBatchLoaded(batch) => {
                for (id, sum) in batch {
                    self.in_flight_residuals.remove(&id);
                    self.app_residuals_cache.insert(id, sum);
                }
                Task::none()
            }
            Message::SearchChanged(query) => {
                self.search_query = query;
                if !self.search_query.trim().is_empty() {
                    let missing: Vec<Application> = self.applications
                        .iter()
                        .filter(|a| {
                            (!a.is_system || self.show_system_libraries)
                                && self.category_filter.matches(&a.install_method)
                                && (a.name.to_lowercase().contains(&self.search_query.to_lowercase())
                                    || a.display_name.to_lowercase().contains(&self.search_query.to_lowercase())
                                    || a.id.to_lowercase().contains(&self.search_query.to_lowercase())
                                    || a.description.as_deref().map(|d| d.to_lowercase().contains(&self.search_query.to_lowercase())).unwrap_or(false))
                                && !self.app_residuals_cache.contains_key(&a.id)
                                && !self.in_flight_residuals.contains(&a.id)
                        })
                        .take(10)
                        .cloned()
                        .collect();

                    if !missing.is_empty() {
                        for a in &missing {
                            self.in_flight_residuals.insert(a.id.clone());
                        }
                        Task::perform(discover_residuals_batch(missing), Message::ResidualsBatchLoaded)
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }
            Message::FilterCategory(cat) => {
                self.category_filter = cat;
                Task::none()
            }
            Message::SelectSort(field) => {
                self.select_sort(field);
                Task::none()
            }
            Message::ToggleUnitSystem => {
                self.unit_system = self.unit_system.toggle();
                Task::none()
            }
            Message::ToggleShowSystemLibraries(val) => {
                self.show_system_libraries = val;
                if val && !self.search_query.is_empty() {
                    let missing: Vec<Application> = self.applications
                        .iter()
                        .filter(|a| !self.app_residuals_cache.contains_key(&a.id) && !self.in_flight_residuals.contains(&a.id))
                        .take(50)
                        .cloned()
                        .collect();
                    if !missing.is_empty() {
                        for a in &missing {
                            self.in_flight_residuals.insert(a.id.clone());
                        }
                        return Task::perform(discover_residuals_batch(missing), Message::ResidualsBatchLoaded);
                    }
                }
                Task::none()
            }
            Message::OpenInspector(app) => {
                self.inspector_app = Some(app.clone());
                self.inspector_residuals.clear();
                self.is_inspector_loading = true;
                Task::perform(
                    async move { discover_residuals_for_app(&app).await },
                    Message::InspectorResidualsLoaded,
                )
            }
            Message::InspectorResidualsLoaded(res) => {
                let res_sum: u64 = res.iter().map(|r| r.size_bytes).sum();
                if let Some(ref a) = self.inspector_app {
                    self.app_residuals_cache.insert(a.id.clone(), res_sum);
                }
                self.inspector_residuals = res;
                self.is_inspector_loading = false;
                Task::none()
            }
            Message::CloseInspector => {
                self.inspector_app = None;
                self.inspector_residuals.clear();
                self.is_inspector_loading = false;
                Task::none()
            }
            Message::OpenCleanModal(app) => {
                self.clean_target = Some(app.clone());
                self.clean_residuals.clear();
                self.is_clean_loading = true;
                self.purge_package = true;
                Task::perform(
                    async move { discover_residuals_for_app(&app).await },
                    Message::CleanResidualsLoaded,
                )
            }
            Message::CleanResidualsLoaded(res) => {
                let res_sum: u64 = res.iter().map(|r| r.size_bytes).sum();
                if let Some(ref a) = self.clean_target {
                    self.app_residuals_cache.insert(a.id.clone(), res_sum);
                }
                self.clean_residuals = res.into_iter().map(|r| (r, true)).collect();
                self.is_clean_loading = false;
                Task::none()
            }
            Message::ToggleCleanResidual(idx, val) => {
                if let Some(item) = self.clean_residuals.get_mut(idx) {
                    item.1 = val;
                }
                Task::none()
            }
            Message::TogglePurgePackage(val) => {
                self.purge_package = val;
                Task::none()
            }
            Message::TogglePermanentDelete(val) => {
                self.permanent_delete = val;
                Task::none()
            }
            Message::ConfirmPurge => {
                if let Some(app) = self.clean_target.clone() {
                    self.is_cleaning = true;
                    let mut selected_residuals: Vec<ResidualCandidate> = self
                        .clean_residuals
                        .iter()
                        .filter(|(_, checked)| *checked)
                        .map(|(r, _)| r.clone())
                        .collect();
                    for r in &mut selected_residuals {
                        r.selected_for_deletion = true;
                    }

                    let perm = self.permanent_delete;
                    let del_mode = if perm {
                        DeletionMode::Permanent
                    } else {
                        DeletionMode::Trash
                    };
                    let purge_pkg = self.purge_package;

                    Task::perform(
                        async move {
                            // 1. Purge residuals
                            let res_report = execute_purge_residuals(&selected_residuals, del_mode).await;

                            // 2. If app package purge selected, execute and verify
                            let mut freed = res_report.freed_bytes;
                            let mut deleted_paths = res_report.deleted_paths;
                            deleted_paths.extend(res_report.trashed_paths);

                            let pkg_success = if purge_pkg {
                                match execute_purge_package(&app, del_mode).await {
                                    Ok(_) => {
                                        freed += app.total_size_bytes;
                                        // Include the primary package executable, desktop launcher, and artifacts in deleted paths
                                        if let Some(ref exec) = app.exec_path {
                                            if !deleted_paths.contains(exec) {
                                                deleted_paths.insert(0, exec.clone());
                                            }
                                        }
                                        if let Some(ref desk) = app.desktop_file {
                                            if !deleted_paths.contains(desk) {
                                                deleted_paths.push(desk.clone());
                                            }
                                        }
                                        for artifact in &app.artifacts {
                                            if !deleted_paths.contains(&artifact.path) {
                                                deleted_paths.push(artifact.path.clone());
                                            }
                                        }
                                        true
                                    }
                                    Err(e) => {
                                        eprintln!("Package purge failed: {}", e);
                                        false
                                    }
                                }
                            } else {
                                true
                            };

                            let status = if pkg_success && res_report.errors.is_empty() {
                                "success".to_string()
                            } else if pkg_success || !deleted_paths.is_empty() {
                                "partial".to_string()
                            } else {
                                "failed".to_string()
                            };

                            let entry = AuditLogEntry {
                                id: None,
                                app_id: app.id.clone(),
                                app_name: app.display_name.clone(),
                                install_method: app.install_method.badge_label().to_string(),
                                timestamp: chrono::Utc::now(),
                                freed_bytes: freed,
                                deleted_paths,
                                status,
                                error_details: if !res_report.errors.is_empty() {
                                    Some(format!("Encountered {} errors during file cleanup", res_report.errors.len()))
                                } else {
                                    None
                                },
                            };

                            if let Ok(db) = Database::open_default() {
                                let _ = db.record_audit(&entry);
                            }

                            Ok(freed)
                        },
                        Message::PurgeCompleted,
                    )
                } else {
                    Task::none()
                }
            }
            Message::PurgeCompleted(res) => {
                self.is_cleaning = false;
                self.clean_target = None;
                self.clean_residuals.clear();
                match res {
                    Ok(freed) => {
                        self.status_message = Some(format!(
                            "Successfully purged application files! Freed {}",
                            self.format_bytes(freed)
                        ));
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Purge error: {}", e));
                    }
                }
                // Refresh list and reload audit history
                Task::batch(vec![
                    Task::perform(scan_all_applications(), Message::ScanCompleted),
                    Task::perform(
                        async {
                            if let Ok(db) = Database::open_default() {
                                db.get_audit_history().unwrap_or_default()
                            } else {
                                Vec::new()
                            }
                        },
                        Message::AuditHistoryLoaded,
                    ),
                ])
            }
            Message::CloseCleanModal => {
                self.clean_target = None;
                self.clean_residuals.clear();
                self.is_clean_loading = false;
                self.is_cleaning = false;
                self.purge_package = true;
                Task::none()
            }
            Message::LoadSweepItems => {
                self.is_sweep_scanning = true;
                Task::perform(scan_all_sweep_items(), Message::SweepItemsLoaded)
            }
            Message::SweepItemsLoaded(items) => {
                self.sweep_items = items.into_iter().map(|i| (i, true)).collect();
                self.is_sweep_scanning = false;
                Task::none()
            }
            Message::ToggleSweepItem(idx, val) => {
                if let Some(item) = self.sweep_items.get_mut(idx) {
                    item.1 = val;
                }
                Task::none()
            }
            Message::RunSystemSweep => {
                let selected_items: Vec<SystemSweepItem> = self
                    .sweep_items
                    .iter()
                    .filter(|(_, checked)| *checked)
                    .map(|(i, _)| i.clone())
                    .collect();

                if selected_items.is_empty() {
                    self.status_message = Some("No system sweep items selected.".to_string());
                    return Task::none();
                }

                self.is_sweeping = true;
                Task::perform(
                    async move {
                        let report = execute_system_sweep(&selected_items).await;
                        let mut deleted_paths = Vec::new();
                        for item in &selected_items {
                            deleted_paths.extend(item.paths.clone());
                        }

                        let entry = AuditLogEntry {
                            id: None,
                            app_id: "system-sweep".to_string(),
                            app_name: "System Sweep & Maintenance".to_string(),
                            install_method: "System".to_string(),
                            timestamp: chrono::Utc::now(),
                            freed_bytes: report.freed_bytes,
                            deleted_paths,
                            status: if report.errors.is_empty() {
                                "SUCCESS".to_string()
                            } else if report.items_cleaned > 0 {
                                "PARTIAL".to_string()
                            } else {
                                "FAILED".to_string()
                            },
                            error_details: if !report.errors.is_empty() {
                                Some(report.errors.join("; "))
                            } else {
                                None
                            },
                        };

                        if let Ok(db) = Database::open_default() {
                            let _ = db.record_audit(&entry);
                        }

                        Ok((report.items_cleaned, report.freed_bytes))
                    },
                    Message::SystemSweepCompleted,
                )
            }
            Message::SystemSweepCompleted(res) => {
                self.is_sweeping = false;
                match res {
                    Ok((count, freed)) => {
                        self.status_message = Some(format!(
                            "Cleaned {} system items! Reclaimed {}",
                            count,
                            self.format_bytes(freed)
                        ));
                    }
                    Err(e) => {
                        self.status_message = Some(format!("System sweep failed: {}", e));
                    }
                }
                // Refresh sweep items and reload audit history
                self.is_sweep_scanning = true;
                Task::batch(vec![
                    Task::perform(scan_all_sweep_items(), Message::SweepItemsLoaded),
                    Task::perform(
                        async {
                            if let Ok(db) = Database::open_default() {
                                db.get_audit_history().unwrap_or_default()
                            } else {
                                Vec::new()
                            }
                        },
                        Message::AuditHistoryLoaded,
                    ),
                ])
            }
            Message::LoadAuditHistory => Task::perform(
                async {
                    if let Ok(db) = Database::open_default() {
                        db.get_audit_history().unwrap_or_default()
                    } else {
                        Vec::new()
                    }
                },
                Message::AuditHistoryLoaded,
            ),
            Message::AuditHistoryLoaded(entries) => {
                self.audit_history = entries;
                Task::none()
            }
            Message::OpenAuditInspector(entry) => {
                self.inspector_audit_entry = Some(entry);
                Task::none()
            }
            Message::CloseAuditInspector => {
                self.inspector_audit_entry = None;
                Task::none()
            }
            Message::OpenFolder(path) => {
                let target = if path.is_file() {
                    path.parent().unwrap_or(&path).to_path_buf()
                } else {
                    path
                };
                tokio::task::spawn_blocking(move || {
                    let _ = open::that(&target);
                });
                Task::none()
            }
            Message::CopyToClipboard(text) => {
                self.status_message = Some(format!("Copied to clipboard: {}", text));
                iced::clipboard::write(text)
            }
            Message::ClearStatus => {
                self.status_message = None;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let header = self.view_header();
        let content: Element<'_, Message> = match self.active_tab {
            NavigationTab::Applications => self.view_applications_dashboard(),
            NavigationTab::SystemSweep => self.view_system_sweep(),
            NavigationTab::AuditHistory => self.view_audit_history(),
        };

        let mut main_col = column![header, content].spacing(16).padding(20);

        if let Some(ref msg) = self.status_message {
            let status_bar = container(
                row![
                    text(format!("● {}", msg)).color(Theme::WARM_CREAM).size(13),
                    Space::new().width(Length::Fill),
                    button(text("Dismiss").size(11))
                        .style(|_t, s| Theme::ghost_button(false, s))
                        .on_press(Message::ClearStatus)
                ]
                .align_y(Alignment::Center)
                .padding([6, 12]),
            )
            .style(|_t| container::Style {
                background: Some(iced::Background::Color(Theme::MAROON_WINE)),
                border: iced::border::Border {
                    color: Theme::BORDER_ACCENT,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .width(Length::Fill);

            main_col = main_col.push(status_bar);
        }

        let base_view: Element<'_, Message> = container(main_col)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_t| container::Style {
                background: Some(iced::Background::Color(Theme::BG_CANVAS)),
                text_color: Some(Theme::TEXT_PRIMARY),
                ..Default::default()
            })
            .into();

        if self.inspector_app.is_some() {
            return self.view_inspector_overlay(base_view);
        }

        if self.clean_target.is_some() {
            return self.view_clean_overlay(base_view);
        }

        if self.inspector_audit_entry.is_some() {
            return self.view_audit_inspector_overlay(base_view);
        }

        base_view
    }

    fn view_header(&self) -> Element<'_, Message> {
        let logo_and_title = row![
            column![
                text("SweepX")
                    .size(22)
                    .color(Theme::WARM_CREAM),
                text("Linux Application Tracker & Deep Purge")
                    .size(12)
                    .color(Theme::OLIVE_SAGE),
            ]
            .spacing(2),
            Space::new().width(Length::Fill),
            // Navigation tabs
            row![
                button(
                    text("Applications")
                        .size(13)
                )
                .style(move |_t, s| Theme::ghost_button(self.active_tab == NavigationTab::Applications, s))
                .on_press(Message::SwitchTab(NavigationTab::Applications)),
                button(
                    text("System Sweep")
                        .size(13)
                )
                .style(move |_t, s| Theme::ghost_button(self.active_tab == NavigationTab::SystemSweep, s))
                .on_press(Message::SwitchTab(NavigationTab::SystemSweep)),
                button(
                    text("Audit History")
                        .size(13)
                )
                .style(move |_t, s| Theme::ghost_button(self.active_tab == NavigationTab::AuditHistory, s))
                .on_press(Message::SwitchTab(NavigationTab::AuditHistory)),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
            Space::new().width(Length::Fill),
            row![
                button(
                    text(if self.is_scanning { "Scanning..." } else { "Refresh Scan" })
                        .size(13)
                )
                .style(|_t, s| Theme::primary_button(s))
                .on_press(Message::RefreshScan),
                button(
                    row![
                        text("Unit: ").size(12).color(Theme::TEXT_MUTED),
                        text(self.unit_system.label()).size(12).color(Theme::PISTACHIO),
                    ]
                    .align_y(Alignment::Center)
                )
                .style(|_t, s| Theme::header_toggle_button(s))
                .on_press(Message::ToggleUnitSystem),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
        ]
        .align_y(Alignment::Center)
        .padding(12);

        container(logo_and_title)
            .style(|_t| Theme::header_style())
            .width(Length::Fill)
            .into()
    }
}

/// Pure filtering and sorting function for applications, exposed for UI and deterministic tests.
pub fn sort_and_filter_applications<'a>(
    applications: &'a [Application],
    query: &str,
    category_filter: CategoryFilter,
    show_system_libraries: bool,
    sort_field: SortField,
    sort_direction: SortDirection,
    app_residuals_cache: &HashMap<String, u64>,
) -> Vec<&'a Application> {
    let mut filtered: Vec<&Application> = applications
        .iter()
        .filter(|app| {
            if !show_system_libraries && app.is_system {
                return false;
            }
            if !category_filter.matches(&app.install_method) {
                return false;
            }
            if !query.is_empty() {
                let q = query.to_lowercase();
                let name_match = app.name.to_lowercase().contains(&q);
                let display_match = app.display_name.to_lowercase().contains(&q);
                let id_match = app.id.to_lowercase().contains(&q);
                let desc_match = app.description.as_deref().map(|d| d.to_lowercase().contains(&q)).unwrap_or(false);
                return name_match || display_match || id_match || desc_match;
            }
            true
        })
        .collect();

    filtered.sort_by(|a, b| {
        let cmp = match sort_field {
            SortField::Name => a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase()),
            SortField::Size => a.total_size_bytes.cmp(&b.total_size_bytes),
            SortField::Footprint => {
                let a_footprint = app_residuals_cache.get(&a.id).copied().map(|r| a.total_size_bytes + r).unwrap_or_else(|| a.known_footprint_bytes());
                let b_footprint = app_residuals_cache.get(&b.id).copied().map(|r| b.total_size_bytes + r).unwrap_or_else(|| b.known_footprint_bytes());
                a_footprint.cmp(&b_footprint)
            }
            SortField::Origin => a.install_method.badge_label().cmp(b.install_method.badge_label()),
        };
        match sort_direction {
            SortDirection::Ascending => cmp,
            SortDirection::Descending => cmp.reverse(),
        }
    });

    filtered
}

impl SweepXApp {
    fn view_applications_dashboard(&self) -> Element<'_, Message> {
        let visible_for_stats: Vec<&Application> = if self.show_system_libraries {
            self.applications.iter().collect()
        } else {
            self.applications.iter().filter(|a| !a.is_system).collect()
        };

        let total_apps = visible_for_stats.len();
        let total_footprint_bytes: u64 = visible_for_stats
            .iter()
            .map(|a| a.total_size_bytes)
            .sum();
        let flatpak_count = visible_for_stats
            .iter()
            .filter(|a| a.install_method == InstallMethod::Flatpak)
            .count();
        let snap_count = visible_for_stats
            .iter()
            .filter(|a| a.install_method == InstallMethod::Snap)
            .count();
        let bottles_count = visible_for_stats
            .iter()
            .filter(|a| a.install_method == InstallMethod::Bottles)
            .count();
        let native_count = visible_for_stats
            .iter()
            .filter(|a| a.install_method.is_native_system() || a.install_method == InstallMethod::CustomDesktop)
            .count();

        // Stat cards
        let stats_row = row![
            self.metric_card(
                "Discovered Apps",
                total_apps.to_string(),
                if self.show_system_libraries {
                    "All software & system packages"
                } else {
                    "User applications discovered"
                },
                Theme::CARD_TEXT_PRIMARY,
            ),
            self.metric_card("Storage Footprint", self.format_bytes(total_footprint_bytes), "Discovered package footprint", Theme::CARD_TEXT_PRIMARY),
            self.metric_card("Native Packages", native_count.to_string(), "APT / DNF / Pacman & Desktop", Theme::CARD_TEXT_PRIMARY),
            self.metric_card("Sandboxed Containers", format!("{} / {}", flatpak_count, snap_count), "Flatpak apps / Snap packages", Theme::CARD_TEXT_PRIMARY),
            self.metric_card("Bottles Wine Apps", bottles_count.to_string(), "Isolated Windows prefixes", Theme::CARD_TEXT_PRIMARY),
        ]
        .spacing(12)
        .width(Length::Fill);

        // Search & Filter controls
        let search_box = text_input("Search apps by name, binary, or package id...", &self.search_query)
            .on_input(Message::SearchChanged)
            .padding(10)
            .size(14)
            .style(|_t, s| Theme::search_input(s))
            .width(Length::FillPortion(3));

        let category_pills = row![
            self.filter_pill(CategoryFilter::All),
            self.filter_pill(CategoryFilter::Native),
            self.filter_pill(CategoryFilter::Flatpak),
            self.filter_pill(CategoryFilter::Snap),
            self.filter_pill(CategoryFilter::Bottles),
            self.filter_pill(CategoryFilter::AppImage),
            self.filter_pill(CategoryFilter::Manual),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        let search_and_filter_row = row![search_box, Space::new().width(Length::Fill), category_pills]
            .spacing(12)
            .align_y(Alignment::Center);

        // Filter & sort applications
        let filtered = sort_and_filter_applications(
            &self.applications,
            &self.search_query,
            self.category_filter,
            self.show_system_libraries,
            self.sort_field,
            self.sort_direction,
            &self.app_residuals_cache,
        );

        let count_summary = text(format!(
            "Showing {} of {} applications",
            filtered.len(),
            self.applications.len()
        ))
        .size(12)
        .color(Theme::TEXT_MUTED);

        let sort_controls = row![
            text("Sort:").size(12).color(Theme::TEXT_MUTED),
            self.sort_pill(SortField::Name),
            self.sort_pill(SortField::Size),
            self.sort_pill(SortField::Footprint),
            self.sort_pill(SortField::Origin),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        let show_sys_toggle = row![
            checkbox(self.show_system_libraries)
                .on_toggle(Message::ToggleShowSystemLibraries)
                .size(14),
            text("Show System Libraries")
                .size(12)
                .color(Theme::WARM_CREAM),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        let sub_filter_row = row![
            count_summary,
            Space::new().width(16.0),
            sort_controls,
            Space::new().width(Length::Fill),
            show_sys_toggle,
        ]
        .align_y(Alignment::Center);

        // Application cards list
        let mut apps_list_col = column![].spacing(8);

        if filtered.is_empty() {
            let empty_text = if self.is_scanning {
                "Scanning system applications across all packaging tiers..."
            } else {
                "No applications found matching your search and filter criteria."
            };
            apps_list_col = apps_list_col.push(
                container(
                    text(empty_text)
                        .size(15)
                        .color(Theme::TEXT_MUTED)
                )
                .padding(40)
                .width(Length::Fill)
                .center_x(Length::Fill),
            );
        } else {
            for app in filtered {
                apps_list_col = apps_list_col.push(self.view_application_card(app));
            }
        }

        let scrollable_apps = scrollable(apps_list_col)
            .direction(Theme::scrollbar_direction())
            .style(Theme::scrollable_style)
            .width(Length::Fill)
            .height(Length::Fill);

        column![
            stats_row,
            Space::new().height(4.0),
            search_and_filter_row,
            sub_filter_row,
            scrollable_apps,
        ]
        .spacing(12)
        .height(Length::Fill)
        .into()
    }

    fn view_application_card<'a>(&'a self, app: &'a Application) -> Element<'a, Message> {
        let method_badge = Theme::method_badge(app.install_method);

        let desc_text = if let Some(ref d) = app.description {
            d.as_str()
        } else if let Some(ref p) = app.exec_path {
            p.to_str().unwrap_or("")
        } else {
            ""
        };

        let mut title_row = row![
            text(&app.display_name).size(15).color(Theme::CARD_TEXT_PRIMARY),
            method_badge,
            button(text("Copy ID").size(10))
                .style(|_t, s| Theme::card_ghost_button(s))
                .on_press(Message::CopyToClipboard(app.id.clone())),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        if let Some(ref ver) = app.version {
            title_row = title_row.push(text(format!("v{}", ver)).size(12).color(Theme::CARD_TEXT_MUTED));
        }

        let desc_widget: Element<'_, Message> = if !desc_text.is_empty() {
            text_input("", desc_text)
                .size(12)
                .style(|_t, s| Theme::selectable_text(s))
                .into()
        } else {
            text("").size(12).color(Theme::CARD_TEXT_MUTED).into()
        };

        let app_details = column![
            title_row,
            desc_widget,
        ]
        .spacing(4);

        // Size pills: Payload and Total Footprint
        let cached_residuals = self.app_residuals_cache.get(&app.id).copied();
        let total_footprint = match cached_residuals {
            Some(res_bytes) => app.total_size_bytes + res_bytes,
            None => app.known_footprint_bytes(),
        };

        let size_pills = row![
            column![
                text("Payload").size(10).color(Theme::CARD_TEXT_MUTED),
                text(self.format_bytes(app.total_size_bytes))
                    .size(13)
                    .color(Theme::CARD_TEXT_PRIMARY),
            ]
            .align_x(Alignment::End),
            column![
                text("Total Footprint").size(10).color(Theme::CARD_TEXT_MUTED),
                text(self.format_bytes(total_footprint))
                    .size(13)
                    .color(Theme::CARD_TEXT_PRIMARY),
            ]
            .align_x(Alignment::End),
        ]
        .spacing(16)
        .align_y(Alignment::Center);

        // Action buttons
        let actions = row![
            button(text("Inspect").size(12))
                .style(|_t, s| Theme::secondary_button(s))
                .on_press(Message::OpenInspector(app.clone())),
            button(text("Purge").size(12))
                .style(|_t, s| Theme::danger_button(s))
                .on_press(Message::OpenCleanModal(app.clone())),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let card_content = row![
            app_details.width(Length::Fill),
            size_pills.width(Length::Shrink),
            actions.width(Length::Shrink),
        ]
        .spacing(16)
        .align_y(Alignment::Center)
        .padding(12);

        container(card_content)
            .style(|_t| Theme::card_style())
            .width(Length::Fill)
            .into()
    }

    fn metric_card<'a>(&'a self, title: &'static str, value: String, subtitle: &'static str, val_color: iced::Color) -> Element<'a, Message> {
        let content = column![
            text(title).size(11).color(Theme::CARD_TEXT_PRIMARY),
            text(value).size(20).color(val_color),
            text(subtitle).size(10).color(Theme::CARD_TEXT_MUTED),
        ]
        .spacing(4);

        container(content)
            .style(|_t| Theme::card_style())
            .padding(12)
            .width(Length::Fill)
            .into()
    }

    fn filter_pill(&self, cat: CategoryFilter) -> Element<'_, Message> {
        let is_active = self.category_filter == cat;
        button(text(cat.label()).size(12))
            .style(move |_t, s| Theme::ghost_button(is_active, s))
            .on_press(Message::FilterCategory(cat))
            .into()
    }

    fn sort_pill(&self, field: SortField) -> Element<'_, Message> {
        let is_active = self.sort_field == field;
        let arrow = if is_active {
            match self.sort_direction {
                SortDirection::Ascending => " ↑",
                SortDirection::Descending => " ↓",
            }
        } else {
            ""
        };
        let label = format!("{}{}", field.label(), arrow);

        button(text(label).size(12))
            .style(move |_t, s| Theme::ghost_button(is_active, s))
            .on_press(Message::SelectSort(field))
            .into()
    }

    // ==========================================
    // INSPECTOR MODAL VIEW
    // ==========================================
    fn view_inspector_overlay<'a>(&'a self, underlay: Element<'a, Message>) -> Element<'a, Message> {
        let app = match self.inspector_app {
            Some(ref a) => a,
            None => return underlay,
        };

        let residuals_size: u64 = self.inspector_residuals.iter().map(|r| r.size_bytes).sum();
        let total_combined_footprint = app.total_footprint_bytes(&self.inspector_residuals);

        let header = row![
            column![
                row![
                    text("Application Inspector")
                        .size(13)
                        .color(Theme::PISTACHIO),
                    Theme::method_badge(app.install_method),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                text(&app.display_name)
                    .size(20)
                    .color(Theme::WARM_CREAM),
                row![
                    text_input("", &app.id)
                        .size(12)
                        .style(|_t, s| Theme::selectable_text(s)),
                    button(text("Copy ID").size(10))
                        .style(|_t, s| Theme::ghost_button(false, s))
                        .on_press(Message::CopyToClipboard(app.id.clone())),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            ]
            .spacing(4),
            Space::new().width(Length::Fill),
            button(text("Close").size(13))
                .style(|_t, s| Theme::secondary_button(s))
                .on_press(Message::CloseInspector)
        ]
        .align_y(Alignment::Start);

        // Footprint accounting summary box
        let accounting_box = container(
            row![
                column![
                    text("Base App Payload").size(11).color(Theme::TEXT_MUTED),
                    text(self.format_bytes(app.total_size_bytes)).size(16).color(Theme::WARM_CREAM),
                ].width(Length::Fill),
                text("+").size(16).color(Theme::OLIVE_SAGE),
                column![
                    text("Discovered User Caches/Data").size(11).color(Theme::TEXT_MUTED),
                    text(self.format_bytes(residuals_size)).size(16).color(Theme::OLIVE_SAGE),
                ].width(Length::Fill),
                text("=").size(16).color(Theme::OLIVE_SAGE),
                column![
                    text("Total Combined Footprint").size(11).color(Theme::TEXT_MUTED),
                    text(self.format_bytes(total_combined_footprint)).size(18).color(Theme::WARM_CREAM),
                ].width(Length::Fill),
            ]
            .spacing(12)
            .align_y(Alignment::Center)
            .padding(12),
        )
        .style(|_t| Theme::modal_inset_card_style())
        .width(Length::Fill);

        // Application Payload Section
        let payload_info = column![
            text("Application Binary & Desktop Launcher")
                .size(15)
                .color(Theme::WARM_CREAM),
            if let Some(ref bin) = app.exec_path {
                let bin_str = bin.to_string_lossy().to_string();
                let bin_label = match app.install_method {
                    InstallMethod::Snap => "Snap Package Bundle",
                    InstallMethod::Flatpak => "Flatpak Application Executable",
                    InstallMethod::Bottles => "Bottle Wine Binary",
                    InstallMethod::AppImage => "AppImage Binary",
                    _ => "Executable Binary",
                };
                row![
                    column![
                        text(bin_label).size(11).color(Theme::OLIVE_SAGE),
                        text_input("", &bin_str)
                            .size(13)
                            .style(|_t, s| Theme::selectable_text(s)),
                    ].width(Length::Fill),
                    text(self.format_bytes(app.total_size_bytes)).size(13).color(Theme::WARM_CREAM),
                    button(text("Copy").size(11))
                        .style(|_t, s| Theme::secondary_button(s))
                        .on_press(Message::CopyToClipboard(bin_str)),
                    button(text("Open Folder").size(11))
                        .style(|_t, s| Theme::secondary_button(s))
                        .on_press(Message::OpenFolder(bin.clone())),
                ].spacing(8).align_y(Alignment::Center)
            } else {
                row![text("No executable binary path registered").size(12).color(Theme::TEXT_MUTED)]
            },
            if let Some(ref desk) = app.desktop_file {
                let desk_str = desk.to_string_lossy().to_string();
                row![
                    column![
                        text("Desktop Entry Launcher").size(11).color(Theme::OLIVE_SAGE),
                        text_input("", &desk_str)
                            .size(13)
                            .style(|_t, s| Theme::selectable_text(s)),
                    ].width(Length::Fill),
                    button(text("Copy").size(11))
                        .style(|_t, s| Theme::secondary_button(s))
                        .on_press(Message::CopyToClipboard(desk_str)),
                    button(text("Open Folder").size(11))
                        .style(|_t, s| Theme::secondary_button(s))
                        .on_press(Message::OpenFolder(desk.clone())),
                ].spacing(8).align_y(Alignment::Center)
            } else {
                row![text("No .desktop launcher file found").size(12).color(Theme::TEXT_MUTED)]
            },
        ]
        .spacing(8);

        // Discovered User Residuals Section
        let mut residuals_list = column![
            text(format!(
                "Discovered User Residuals & Sandboxes ({})",
                self.inspector_residuals.len()
            ))
            .size(15)
            .color(Theme::WARM_CREAM),
        ]
        .spacing(8);

        if self.is_inspector_loading {
            residuals_list = residuals_list.push(
                text("Scanning deep host heuristics for configuration, cache, and state files...")
                    .size(13)
                    .color(Theme::TEXT_MUTED),
            );
        } else if self.inspector_residuals.is_empty() {
            residuals_list = residuals_list.push(
                text("No additional user cache or config paths discovered outside base payload.")
                    .size(13)
                    .color(Theme::TEXT_MUTED),
            );
        } else {
            for res in &self.inspector_residuals {
                let path_str = res.path.to_string_lossy().to_string();
                let res_row = row![
                    column![
                        text(res.kind.display_name()).size(11).color(Theme::OLIVE_SAGE),
                        text_input("", &path_str)
                            .size(12)
                            .style(|_t, s| Theme::selectable_text(s)),
                    ]
                    .width(Length::Fill),
                    text(self.format_bytes(res.size_bytes)).size(13).color(Theme::WARM_CREAM),
                    button(text("Copy").size(11))
                        .style(|_t, s| Theme::secondary_button(s))
                        .on_press(Message::CopyToClipboard(path_str)),
                    button(text("Open").size(11))
                        .style(|_t, s| Theme::secondary_button(s))
                        .on_press(Message::OpenFolder(res.path.clone())),
                ]
                .spacing(8)
                .align_y(Alignment::Center)
                .padding(6);

                residuals_list = residuals_list.push(
                    container(res_row)
                        .style(|_t| Theme::modal_inset_card_style())
                        .width(Length::Fill),
                );
            }
        }

        let modal_body = column![
            header,
            accounting_box,
            payload_info,
            residuals_list,
        ]
        .spacing(14)
        .padding(20);

        let modal_container = container(
            scrollable(modal_body)
                .direction(Theme::scrollbar_direction())
                .style(Theme::modal_scrollable_style),
        )
        .style(|_t| Theme::modal_style())
        .width(Length::Fixed(850.0))
        .max_height(600);

        let backdrop = container(modal_container)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_t| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.70))),
                ..Default::default()
            });

        stack![underlay, backdrop].into()
    }

    // ==========================================
    // CLEAN / PURGE MODAL VIEW
    // ==========================================
    fn view_clean_overlay<'a>(&'a self, underlay: Element<'a, Message>) -> Element<'a, Message> {
        let app = match self.clean_target {
            Some(ref a) => a,
            None => return underlay,
        };

        let selected_residuals_bytes: u64 = self
            .clean_residuals
            .iter()
            .filter(|(_, checked)| *checked)
            .map(|(r, _)| r.size_bytes)
            .sum();

        let package_bytes = if self.purge_package {
            app.total_size_bytes
        } else {
            0
        };

        let total_reclaimable = selected_residuals_bytes + package_bytes;

        let header = row![
            column![
                row![
                    text("Purge Application")
                        .size(13)
                        .color(Theme::PISTACHIO),
                    Theme::method_badge(app.install_method),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                text(&app.display_name)
                    .size(20)
                    .color(Theme::WARM_CREAM),
                text("Select discovered application package payloads and user caches to safely remove.")
                    .size(12)
                    .color(Theme::TEXT_MUTED),
            ]
            .spacing(4),
            Space::new().width(Length::Fill),
            button(text("Cancel").size(13))
                .style(|_t, s| Theme::secondary_button(s))
                .on_press(Message::CloseCleanModal)
        ]
        .align_y(Alignment::Start);

        // Summary banner
        let summary_banner = container(
            row![
                column![
                    text("Estimated Reclaimable Disk Space")
                        .size(11)
                        .color(Theme::TEXT_MUTED),
                    text(self.format_bytes(total_reclaimable))
                        .size(20)
                        .color(Theme::WARM_CREAM),
                ]
                .width(Length::Fill),
                column![
                    row![
                        checkbox(self.permanent_delete)
                            .on_toggle(Message::TogglePermanentDelete)
                            .size(14),
                        text("Permanently Delete (bypass trash)")
                            .size(13)
                            .color(Theme::WARM_CREAM),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                    text("If unchecked, files are safely moved to host Trash.")
                        .size(11)
                        .color(Theme::TEXT_MUTED),
                ],
            ]
            .spacing(16)
            .align_y(Alignment::Center)
            .padding(12),
        )
        .style(|_t| Theme::modal_inset_card_style())
        .width(Length::Fill);

        // Section 1: Application Binary & Primary Package Payload
        let package_label = match app.install_method {
            InstallMethod::Flatpak => "Flatpak Application Package (system / user uninstall)",
            InstallMethod::Snap => "Snap Application Package (snap remove --purge)",
            InstallMethod::NativeDnf => "DNF System Package (dnf remove)",
            InstallMethod::NativeApt => "APT System Package (apt purge)",
            InstallMethod::NativePacman => "Pacman System Package (pacman -Rns)",
            InstallMethod::AppImage => "AppImage Application Executable Binary",
            InstallMethod::Bottles => "Wine Bottle Prefix & Application Directory",
            InstallMethod::ManualOpt => "Manual Installation Payload Directory (/opt or user bin)",
            InstallMethod::CustomDesktop => "Application Binary & Desktop Launcher",
        };

        let package_target_str = if let Some(ref exec) = app.exec_path {
            exec.to_string_lossy().to_string()
        } else {
            app.id.clone()
        };
        let package_target_copy = package_target_str.clone();

        let package_section = column![
            row![
                text("Application Binary & Primary Package Payload")
                    .size(14)
                    .color(Theme::WARM_CREAM),
                Theme::method_badge(app.install_method),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            container(
                row![
                    checkbox(self.purge_package)
                        .on_toggle(Message::TogglePurgePackage)
                        .size(13),
                    column![
                        text(package_label).size(12).color(Theme::OLIVE_SAGE),
                        text_input("", &package_target_str)
                            .size(12)
                            .style(|_t, s| Theme::selectable_text(s)),
                    ]
                    .spacing(2)
                    .width(Length::Fill),
                    text(format!("({})", self.format_bytes(app.total_size_bytes)))
                        .size(12)
                        .color(Theme::WARM_CREAM),
                    button(text("Copy").size(10))
                        .style(|_t, s| Theme::ghost_button(false, s))
                        .on_press(Message::CopyToClipboard(package_target_copy)),
                ]
                .spacing(10)
                .align_y(Alignment::Center)
                .padding(8)
            )
            .style(|_t| Theme::modal_inset_card_style())
            .width(Length::Fill),
        ]
        .spacing(8);

        // Section 2: Discovered Residuals & Artifacts
        let mut items_col = column![
            text(format!("Discovered User Residuals & Sandboxes ({})", self.clean_residuals.len()))
                .size(14)
                .color(Theme::WARM_CREAM),
        ]
        .spacing(8);

        if self.is_clean_loading {
            items_col = items_col.push(
                text("Scanning application paths...").size(13).color(Theme::TEXT_MUTED)
            );
        } else if self.clean_residuals.is_empty() {
            items_col = items_col.push(
                container(
                    text("No additional user caches discovered outside base package payload.")
                        .size(13)
                        .color(Theme::TEXT_MUTED)
                )
                .padding(8)
            );
        } else {
            for (idx, (res, checked)) in self.clean_residuals.iter().enumerate() {
                let path_str = res.path.to_string_lossy().to_string();
                let path_str_copy = path_str.clone();
                let check_row = row![
                    checkbox(*checked)
                        .on_toggle(move |val| Message::ToggleCleanResidual(idx, val))
                        .size(13),
                    column![
                        text(format!("{}:", res.kind.display_name())).size(11).color(Theme::OLIVE_SAGE),
                        text_input("", &path_str)
                            .size(12)
                            .style(|_t, s| Theme::selectable_text(s)),
                    ]
                    .spacing(2)
                    .width(Length::Fill),
                    text(format!("({})", self.format_bytes(res.size_bytes)))
                        .size(11)
                        .color(Theme::TEXT_MUTED),
                    button(text("Copy").size(10))
                        .style(|_t, s| Theme::ghost_button(false, s))
                        .on_press(Message::CopyToClipboard(path_str_copy)),
                ]
                .spacing(8)
                .align_y(Alignment::Center);

                items_col = items_col.push(
                    container(check_row)
                        .style(|_t| Theme::modal_inset_card_style())
                        .padding(8)
                        .width(Length::Fill),
                );
            }
        }

        let action_buttons = row![
            Space::new().width(Length::Fill),
            button(text("Cancel").size(13))
                .style(|_t, s| Theme::secondary_button(s))
                .on_press(Message::CloseCleanModal),
            button(
                text(if self.is_cleaning { "Purging..." } else { "Confirm Purge" })
                    .size(13)
            )
            .style(|_t, s| Theme::danger_button(s))
            .on_press(Message::ConfirmPurge),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        let checklist_content = column![
            package_section,
            items_col,
        ]
        .spacing(12);

        let modal_body = column![
            header,
            summary_banner,
            scrollable(checklist_content)
                .direction(Theme::scrollbar_direction())
                .style(Theme::modal_scrollable_style)
                .height(Length::Fixed(280.0)),
            action_buttons,
        ]
        .spacing(14)
        .padding(20);

        let modal_container = container(modal_body)
            .style(|_t| Theme::modal_style())
            .width(Length::Fixed(820.0));

        let backdrop = container(modal_container)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_t| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.70))),
                ..Default::default()
            });

        stack![underlay, backdrop].into()
    }

    // ==========================================
    // SYSTEM SWEEP VIEW
    // ==========================================
    fn view_system_sweep(&self) -> Element<'_, Message> {
        let total_reclaimable: u64 = self
            .sweep_items
            .iter()
            .filter(|(_, checked)| *checked)
            .map(|(item, _)| item.reclaimable_bytes)
            .sum();

        let header_row = row![
            column![
                text("System Optimizer & Space Reclaimer")
                    .size(20)
                    .color(Theme::WARM_CREAM),
                text("Clean residual package caches, disabled snap revisions, and system thumbnails safely.")
                    .size(13)
                    .color(Theme::OLIVE_SAGE),
            ]
            .spacing(2),
            Space::new().width(Length::Fill),
            button(text("Rescan System").size(13))
                .style(|_t, s| Theme::secondary_button(s))
                .on_press(Message::LoadSweepItems),
            button(
                text(if self.is_sweeping { "Reclaiming..." } else { "Reclaim Space" })
                    .size(13)
            )
            .style(|_t, s| Theme::primary_button(s))
            .on_press(Message::RunSystemSweep),
        ]
        .align_y(Alignment::Center);

        let banner = container(
            row![
                column![
                    text("Total Selected Reclaimable Disk Space")
                        .size(11)
                        .color(Theme::CARD_TEXT_MUTED),
                    text(self.format_bytes(total_reclaimable))
                        .size(24)
                        .color(Theme::CARD_TEXT_PRIMARY),
                ]
                .width(Length::Fill),
                text("Zero risk of removing active dependencies or critical user documents.")
                    .size(12)
                    .color(Theme::CARD_TEXT_MUTED),
            ]
            .padding(14)
            .align_y(Alignment::Center),
        )
        .style(|_t| Theme::active_card_style())
        .width(Length::Fill);

        let mut items_col = column![].spacing(8);

        if self.is_sweep_scanning {
            items_col = items_col.push(
                container(
                    text("Scanning host package managers, snapd, and cache repositories...")
                        .size(14)
                        .color(Theme::TEXT_MUTED),
                )
                .padding(30),
            );
        } else if self.sweep_items.is_empty() {
            items_col = items_col.push(
                container(
                    text("No cleanable system caches or disabled snap revisions found! Your system is clean.")
                        .size(14)
                        .color(Theme::OLIVE_SAGE),
                )
                .padding(30),
            );
        } else {
            for (idx, (item, checked)) in self.sweep_items.iter().enumerate() {
                let item_card = row![
                    checkbox(*checked)
                        .on_toggle(move |val| Message::ToggleSweepItem(idx, val))
                        .size(14),
                    column![
                        row![
                            text(&item.title).size(14).color(Theme::CARD_TEXT_PRIMARY),
                            container(text(item.category.display_name()).size(10).color(Theme::WARM_CREAM))
                                .style(|_t| container::Style {
                                    background: Some(iced::Background::Color(Theme::BURGUNDY_CANVAS)),
                                    border: iced::border::Border { radius: 4.0.into(), ..Default::default() },
                                    ..Default::default()
                                })
                                .padding([2, 6]),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                        text(&item.description).size(12).color(Theme::CARD_TEXT_MUTED),
                    ]
                    .width(Length::Fill),
                    text(self.format_bytes(item.reclaimable_bytes))
                        .size(14)
                        .color(Theme::CARD_TEXT_PRIMARY),
                ]
                .spacing(12)
                .align_y(Alignment::Center)
                .padding(12);

                items_col = items_col.push(
                    container(item_card)
                        .style(|_t| Theme::card_style())
                        .width(Length::Fill),
                );
            }
        }

        column![
            header_row,
            banner,
            scrollable(items_col)
                .direction(Theme::scrollbar_direction())
                .style(Theme::scrollable_style)
                .height(Length::Fill),
        ]
        .spacing(14)
        .height(Length::Fill)
        .into()
    }

    // ==========================================
    // AUDIT HISTORY VIEW
    // ==========================================
    fn view_audit_history(&self) -> Element<'_, Message> {
        let header_row = row![
            column![
                text("Purge & Sweep Audit History")
                    .size(20)
                    .color(Theme::WARM_CREAM),
                text("Tamper-evident record of all uninstalled applications, deleted paths, and freed storage.")
                    .size(13)
                    .color(Theme::OLIVE_SAGE),
            ]
            .spacing(2),
            Space::new().width(Length::Fill),
            button(text("Refresh History").size(13))
                .style(|_t, s| Theme::secondary_button(s))
                .on_press(Message::LoadAuditHistory),
        ]
        .align_y(Alignment::Center);

        let mut list_col = column![].spacing(8);

        if self.audit_history.is_empty() {
            list_col = list_col.push(
                container(
                    text("No purge operations recorded yet. Clean an application or run a system sweep to see logs.")
                        .size(14)
                        .color(Theme::TEXT_MUTED),
                )
                .padding(30),
            );
        } else {
            for entry in &self.audit_history {
                let date_str = entry.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string();
                let status_upper = entry.status.to_uppercase();
                let status_color = if status_upper == "SUCCESS" {
                    Theme::OLIVE_SAGE
                } else {
                    Theme::DANGER
                };

                let entry_card = row![
                    column![
                        row![
                            text(&entry.app_name).size(15).color(Theme::CARD_TEXT_PRIMARY),
                            Theme::badge_by_label(&entry.install_method),
                            text(date_str).size(11).color(Theme::CARD_TEXT_MUTED),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                        text(format!("Deleted {} paths", entry.deleted_paths.len()))
                            .size(12)
                            .color(Theme::CARD_TEXT_MUTED),
                    ]
                    .width(Length::Fill),
                    column![
                        text(status_upper).size(12).color(status_color),
                        text(format!("+{}", self.format_bytes(entry.freed_bytes)))
                            .size(15)
                            .color(Theme::CARD_TEXT_PRIMARY),
                    ]
                    .align_x(Alignment::End),
                    button(text("Inspect").size(12))
                        .style(|_t, s| Theme::primary_button(s))
                        .on_press(Message::OpenAuditInspector(entry.clone())),
                ]
                .spacing(16)
                .align_y(Alignment::Center)
                .padding(12);

                list_col = list_col.push(
                    container(entry_card)
                        .style(|_t| Theme::card_style())
                        .width(Length::Fill),
                );
            }
        }

        column![
            header_row,
            scrollable(list_col)
                .direction(Theme::scrollbar_direction())
                .style(Theme::scrollable_style)
                .height(Length::Fill),
        ]
        .spacing(14)
        .height(Length::Fill)
        .into()
    }

    // ==========================================
    // AUDIT LOG INSPECTOR MODAL VIEW
    // ==========================================
    fn view_audit_inspector_overlay<'a>(&'a self, underlay: Element<'a, Message>) -> Element<'a, Message> {
        let entry = match self.inspector_audit_entry {
            Some(ref e) => e,
            None => return underlay,
        };

        let status_upper = entry.status.to_uppercase();
        let status_color = if status_upper == "SUCCESS" {
            Theme::OLIVE_SAGE
        } else {
            Theme::DANGER
        };
        let date_str = entry.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string();

        let header = row![
            column![
                row![
                    text("Audit Record Inspector")
                        .size(13)
                        .color(Theme::PISTACHIO),
                    Theme::badge_by_label(&entry.install_method),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                text(&entry.app_name)
                    .size(20)
                    .color(Theme::WARM_CREAM),
                row![
                    text_input("", &entry.app_id)
                        .size(12)
                        .style(|_t, s| Theme::selectable_text(s)),
                    button(text("Copy ID").size(10))
                        .style(|_t, s| Theme::ghost_button(false, s))
                        .on_press(Message::CopyToClipboard(entry.app_id.clone())),
                    text(format!("Executed: {}", date_str))
                        .size(11)
                        .color(Theme::TEXT_MUTED),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            ]
            .spacing(4),
            Space::new().width(Length::Fill),
            button(text("Close").size(13))
                .style(|_t, s| Theme::secondary_button(s))
                .on_press(Message::CloseAuditInspector)
        ]
        .align_y(Alignment::Start);

        // Accounting summary box
        let accounting_box = container(
            row![
                column![
                    text("Reclaimed Space").size(11).color(Theme::TEXT_MUTED),
                    text(format!("+{}", self.format_bytes(entry.freed_bytes))).size(16).color(Theme::WARM_CREAM),
                ].width(Length::Fill),
                text("•").size(16).color(Theme::OLIVE_SAGE),
                column![
                    text("Operation Status").size(11).color(Theme::TEXT_MUTED),
                    text(status_upper).size(16).color(status_color),
                ].width(Length::Fill),
                text("•").size(16).color(Theme::OLIVE_SAGE),
                column![
                    text("Deleted Targets").size(11).color(Theme::TEXT_MUTED),
                    text(format!("{} paths", entry.deleted_paths.len())).size(16).color(Theme::OLIVE_SAGE),
                ].width(Length::Fill),
            ]
            .spacing(12)
            .align_y(Alignment::Center)
            .padding(12),
        )
        .style(|_t| Theme::modal_inset_card_style())
        .width(Length::Fill);

        // Deleted paths list section
        let mut paths_col = column![
            text(format!("Purged File & Directory Paths ({})", entry.deleted_paths.len()))
                .size(15)
                .color(Theme::WARM_CREAM),
        ]
        .spacing(8);

        if let Some(ref err) = entry.error_details {
            paths_col = paths_col.push(
                container(
                    column![
                        text("Recorded Error / Warning Details")
                            .size(11)
                            .color(Theme::DANGER),
                        text(err)
                            .size(12)
                            .color(Theme::WARM_CREAM),
                    ]
                    .spacing(4)
                )
                .padding(10)
                .style(|_t| Theme::modal_inset_card_style())
                .width(Length::Fill),
            );
        }

        if entry.deleted_paths.is_empty() {
            paths_col = paths_col.push(
                container(
                    text("No individual file paths were recorded for this operation (e.g. uninstalled via system package manager).")
                        .size(13)
                        .color(Theme::TEXT_MUTED),
                )
                .padding(12)
            );
        } else {
            let max_display = 100;
            for path in entry.deleted_paths.iter().take(max_display) {
                let path_str = path.to_string_lossy().to_string();
                let path_str_copy = path_str.clone();

                let path_row = row![
                    text_input("", &path_str)
                        .size(12)
                        .style(|_t, s| Theme::selectable_text(s)),
                    button(text("Copy").size(11))
                        .style(|_t, s| Theme::secondary_button(s))
                        .on_press(Message::CopyToClipboard(path_str_copy)),
                ]
                .spacing(8)
                .align_y(Alignment::Center)
                .padding(6);

                paths_col = paths_col.push(
                    container(path_row)
                        .style(|_t| Theme::modal_inset_card_style())
                        .width(Length::Fill),
                );
            }

            if entry.deleted_paths.len() > max_display {
                let remaining = entry.deleted_paths.len() - max_display;
                paths_col = paths_col.push(
                    container(
                        text(format!("... and {} more recorded paths (total {})", remaining, entry.deleted_paths.len()))
                            .size(12)
                            .color(Theme::TEXT_MUTED)
                    )
                    .padding(8)
                );
            }
        }

        let modal_body = column![
            header,
            accounting_box,
            paths_col,
        ]
        .spacing(14)
        .padding(20);

        let modal_container = container(
            scrollable(modal_body)
                .direction(Theme::scrollbar_direction())
                .style(Theme::modal_scrollable_style),
        )
        .style(|_t| Theme::modal_style())
        .width(Length::Fixed(850.0))
        .max_height(600);

        let backdrop = container(modal_container)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_t| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.70))),
                ..Default::default()
            });

        stack![underlay, backdrop].into()
    }
}

pub fn get_window_icon() -> Option<iced::window::Icon> {
    const ICON_BYTES: &[u8] = include_bytes!("../../assets/icons/sweepx.png");
    let img = image::load_from_memory(ICON_BYTES).ok()?.to_rgba8();
    let (width, height) = img.dimensions();
    iced::window::icon::from_rgba(img.into_raw(), width, height).ok()
}

pub fn run_gui() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure application icons and desktop entry reflect the current official branding
    let _ = crate::updater::update_installed_assets();

    iced::application(SweepXApp::new, SweepXApp::update, SweepXApp::view)
        .title("SweepX — Unified Linux App Tracker & Deep Purge")
        .window(iced::window::Settings {
            size: iced::Size::new(1150.0, 750.0),
            min_size: Some(iced::Size::new(900.0, 600.0)),
            icon: get_window_icon(),
            #[cfg(target_os = "linux")]
            platform_specific: iced::window::settings::PlatformSpecific {
                application_id: "sweepx".to_string(),
                ..Default::default()
            },
            ..Default::default()
        })
        .run()?;
    Ok(())
}

/// Asynchronously calculates total residual footprints in parallel across a batch of applications.
async fn discover_residuals_batch(apps: Vec<Application>) -> Vec<(String, u64)> {
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    let semaphore = Arc::new(Semaphore::new(16));
    let mut handles = Vec::with_capacity(apps.len());
    for app in apps {
        let sem = semaphore.clone();
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await;
            let res = discover_residuals_for_app(&app).await;
            let sum: u64 = res.iter().map(|r| r.size_bytes).sum();
            (app.id, sum)
        }));
    }
    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        if let Ok(res) = handle.await {
            results.push(res);
        }
    }
    results
}
