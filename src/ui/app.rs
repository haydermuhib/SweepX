use super::theme::Theme;
use super::views::{CleanModal, DashboardView, HistoryView, InspectorModal, UiCategoryFilter};
use crate::cleaner::{
    discover_residuals_for_app, execute_purge_package, execute_purge_residuals, DeletionMode,
};
use crate::db::{AuditLogEntry, Database};
use crate::models::{Application, ResidualCandidate};
use crate::scanner::scan_all_applications;
use chrono::Utc;
use eframe::egui::{self, Align, Layout, RichText};
use std::sync::mpsc::{channel, Receiver, Sender};
use tokio::runtime::Runtime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Dashboard,
    History,
}

pub enum AsyncMessage {
    AppsScanned(Vec<Application>),
    ResidualsFound(Application, Vec<ResidualCandidate>),
    PurgeComplete(String, u64),
    HistoryLoaded(Vec<AuditLogEntry>),
    StatusUpdate(String),
}

pub struct SweepXApp {
    rt: Runtime,
    tx: Sender<AsyncMessage>,
    rx: Receiver<AsyncMessage>,

    apps: Vec<Application>,
    history_logs: Vec<AuditLogEntry>,

    active_tab: ActiveTab,
    search_query: String,
    category_filter: UiCategoryFilter,
    show_system_packages: bool,

    // Modal states
    inspecting_app: Option<Application>,
    inspecting_residuals: Vec<ResidualCandidate>,
    show_inspector: bool,

    cleaning_app: Option<Application>,
    cleaning_residuals: Vec<ResidualCandidate>,
    show_clean_modal: bool,
    is_permanent_delete: bool,
    is_purging: bool,
    purge_status: Option<String>,

    is_scanning: bool,
}

impl SweepXApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Theme::apply(&cc.egui_ctx);
        let rt = Runtime::new().expect("Failed to create Tokio runtime");
        let (tx, rx) = channel();

        let mut app = Self {
            rt,
            tx,
            rx,
            apps: Vec::new(),
            history_logs: Vec::new(),
            active_tab: ActiveTab::Dashboard,
            search_query: String::new(),
            category_filter: UiCategoryFilter::All,
            show_system_packages: false,
            inspecting_app: None,
            inspecting_residuals: Vec::new(),
            show_inspector: false,
            cleaning_app: None,
            cleaning_residuals: Vec::new(),
            show_clean_modal: false,
            is_permanent_delete: false,
            is_purging: false,
            purge_status: None,
            is_scanning: false,
        };

        app.trigger_refresh();
        app
    }

    pub fn trigger_refresh(&mut self) {
        self.is_scanning = true;
        let tx = self.tx.clone();

        self.rt.spawn(async move {
            let apps = scan_all_applications().await;

            if let Ok(mut db) = Database::open_default() {
                let _ = db.save_apps(&apps);
                if let Ok(history) = db.get_audit_history() {
                    let _ = tx.send(AsyncMessage::HistoryLoaded(history));
                }
            }

            let _ = tx.send(AsyncMessage::AppsScanned(apps));
        });
    }

    fn handle_async_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AsyncMessage::AppsScanned(apps) => {
                    self.apps = apps;
                    self.is_scanning = false;
                }
                AsyncMessage::ResidualsFound(app, residuals) => {
                    if self.show_clean_modal && self.cleaning_app.as_ref().map(|a| &a.id) == Some(&app.id) {
                        self.cleaning_residuals = residuals.clone();
                    }
                    if self.show_inspector && self.inspecting_app.as_ref().map(|a| &a.id) == Some(&app.id) {
                        self.inspecting_residuals = residuals;
                    }
                }
                AsyncMessage::PurgeComplete(msg, _freed) => {
                    self.is_purging = false;
                    self.purge_status = Some(msg);
                    self.show_clean_modal = false;
                    self.trigger_refresh();
                }
                AsyncMessage::HistoryLoaded(logs) => {
                    self.history_logs = logs;
                }
                AsyncMessage::StatusUpdate(s) => {
                    self.purge_status = Some(s);
                }
            }
        }
    }
}

impl eframe::App for SweepXApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_async_messages();

        egui::TopBottomPanel::top("top_header").show(ctx, |ui| {
            ui.add_space(8.0_f32);
            ui.horizontal(|ui| {
                ui.label(RichText::new("⚡ SweepX").size(20.0_f32).strong().color(Theme::PRIMARY));
                ui.label(RichText::new("Linux App Tracker & Deep Purge").size(13.0_f32).color(Theme::TEXT_MUTED));

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button(if self.is_scanning { "⏳ Scanning..." } else { "🔄 Refresh Scan" }).clicked() {
                        self.trigger_refresh();
                    }

                    ui.add_space(16.0_f32);

                    let tabs = [
                        (ActiveTab::History, "📜 History"),
                        (ActiveTab::Dashboard, "📦 Applications"),
                    ];

                    for (tab, label) in tabs {
                        let is_active = self.active_tab == tab;
                        let text = if is_active {
                            RichText::new(label).strong().color(Theme::PRIMARY)
                        } else {
                            RichText::new(label).color(Theme::TEXT_SECONDARY)
                        };

                        if ui.selectable_label(is_active, text).clicked() {
                            self.active_tab = tab;
                        }
                    }
                });
            });
            ui.add_space(8.0_f32);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut on_inspect = None;
            let mut on_clean = None;

            match self.active_tab {
                ActiveTab::Dashboard => {
                    DashboardView::render(
                        ui,
                        &self.apps,
                        &mut self.search_query,
                        &mut self.category_filter,
                        &mut self.show_system_packages,
                        &mut on_inspect,
                        &mut on_clean,
                    );
                }
                ActiveTab::History => {
                    HistoryView::render(ui, &self.history_logs);
                }
            }

            if let Some(app) = on_inspect {
                self.inspecting_app = Some(app.clone());
                self.show_inspector = true;
                self.inspecting_residuals = Vec::new();

                let tx = self.tx.clone();
                self.rt.spawn(async move {
                    let residuals = discover_residuals_for_app(&app).await;
                    let _ = tx.send(AsyncMessage::ResidualsFound(app, residuals));
                });
            }

            if let Some(app) = on_clean {
                self.cleaning_app = Some(app.clone());
                self.show_clean_modal = true;
                self.cleaning_residuals = Vec::new();
                self.purge_status = None;

                let tx = self.tx.clone();
                self.rt.spawn(async move {
                    let residuals = discover_residuals_for_app(&app).await;
                    let _ = tx.send(AsyncMessage::ResidualsFound(app, residuals));
                });
            }

            if self.show_inspector {
                if let Some(ref app) = self.inspecting_app {
                    InspectorModal::render(
                        ctx,
                        app,
                        &self.inspecting_residuals,
                        &mut self.show_inspector,
                    );
                }
            }

            if self.show_clean_modal {
                if let Some(ref app) = self.cleaning_app {
                    let mut confirm_purge = false;
                    CleanModal::render(
                        ctx,
                        app,
                        &mut self.cleaning_residuals,
                        &mut self.is_permanent_delete,
                        &mut self.show_clean_modal,
                        self.is_purging,
                        &self.purge_status,
                        &mut confirm_purge,
                    );

                    if confirm_purge {
                        self.is_purging = true;
                        self.purge_status = Some("Purging package & artifacts...".to_string());
                        let app_clone = app.clone();
                        let residuals = self.cleaning_residuals.clone();
                        let mode = if self.is_permanent_delete {
                            DeletionMode::Permanent
                        } else {
                            DeletionMode::Trash
                        };
                        let tx = self.tx.clone();

                        self.rt.spawn(async move {
                            let _pkg_res = execute_purge_package(&app_clone).await;
                            let report = execute_purge_residuals(&residuals, mode).await;

                            if let Ok(db) = Database::open_default() {
                                let audit = AuditLogEntry {
                                    id: None,
                                    app_id: app_clone.id.clone(),
                                    app_name: app_clone.display_name.clone(),
                                    install_method: app_clone.install_method.badge_label().to_string(),
                                    timestamp: Utc::now(),
                                    freed_bytes: report.freed_bytes + app_clone.total_size_bytes,
                                    deleted_paths: report
                                        .deleted_paths
                                        .iter()
                                        .chain(report.trashed_paths.iter())
                                        .cloned()
                                        .collect(),
                                    status: if report.success { "SUCCESS".to_string() } else { "PARTIAL".to_string() },
                                    error_details: None,
                                };
                                let _ = db.record_audit(&audit);
                            }

                            let _ = tx.send(AsyncMessage::PurgeComplete(
                                "Purge completed successfully".to_string(),
                                report.freed_bytes,
                            ));
                        });
                    }
                }
            }
        });
    }
}
