use super::theme::Theme;
use super::views::{
    CleanModal, DashboardView, HistoryView, InspectorModal, OptimizerView, SortDirection,
    SortField, UiCategoryFilter,
};
use crate::cleaner::{
    discover_residuals_for_app, execute_purge_package, execute_purge_residuals,
    execute_system_sweep, scan_all_sweep_items, DeletionMode, SystemSweepItem, SystemSweepReport,
};
use crate::db::{AuditLogEntry, Database};
use crate::models::{Application, ResidualCandidate};
use crate::scanner::{
    apply_stage_batch, detect_available_package_managers, scan_desktop_entries, scan_flatpaks,
    scan_manual_installations, scan_snaps, ScanStageBatch,
};
use chrono::Utc;
use eframe::egui::{self, Align, Color32, Layout, RichText, Stroke};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use tokio::runtime::Runtime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Dashboard,
    Optimizer,
    History,
}

pub enum AsyncMessage {
    ScanBatchReceived(ScanStageBatch),
    ScanFinished,
    UpdateAvailable(crate::updater::UpdateInfo),
    UpdateComplete(String),
    ResidualsFound(Application, Vec<ResidualCandidate>),
    PurgeComplete(String, u64),
    HistoryLoaded(Vec<AuditLogEntry>),
    SweepItemsScanned(Vec<SystemSweepItem>),
    SweepCleanComplete(SystemSweepReport),
    StatusUpdate(String),
}

pub struct SweepXApp {
    rt: Runtime,
    tx: Sender<AsyncMessage>,
    rx: Receiver<AsyncMessage>,

    app_map: HashMap<String, Application>,
    apps: Vec<Application>,
    history_logs: Vec<AuditLogEntry>,
    sweep_items: Vec<SystemSweepItem>,
    is_cleaning_sweep: bool,
    last_sweep_report: Option<SystemSweepReport>,

    active_tab: ActiveTab,
    search_query: String,
    category_filter: UiCategoryFilter,
    sort_field: SortField,
    sort_direction: SortDirection,
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
    scan_status: Option<String>,

    available_update: Option<crate::updater::UpdateInfo>,
    is_updating_app: bool,
    update_message: Option<String>,
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
            app_map: HashMap::new(),
            apps: Vec::new(),
            history_logs: Vec::new(),
            sweep_items: Vec::new(),
            is_cleaning_sweep: false,
            last_sweep_report: None,
            active_tab: ActiveTab::Dashboard,
            search_query: String::new(),
            category_filter: UiCategoryFilter::All,
            sort_field: SortField::Size,
            sort_direction: SortDirection::Descending,
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
            scan_status: None,
            available_update: None,
            is_updating_app: false,
            update_message: None,
        };

        app.trigger_refresh();
        app.trigger_sweep_scan();
        app.trigger_update_check();
        app
    }

    pub fn trigger_update_check(&mut self) {
        let tx = self.tx.clone();
        self.rt.spawn(async move {
            if let Ok(info) = crate::updater::check_for_updates().await {
                if info.has_update {
                    let _ = tx.send(AsyncMessage::UpdateAvailable(info));
                }
            }
        });
    }

    pub fn trigger_refresh(&mut self) {
        self.is_scanning = true;
        self.scan_status = Some("Starting live scanner...".to_string());
        self.app_map.clear();
        self.apps.clear();

        let tx = self.tx.clone();

        self.rt.spawn(async move {
            let tx1 = tx.clone();
            let t1 = tokio::spawn(async move {
                let apps = scan_desktop_entries().await;
                let _ = tx1.send(AsyncMessage::ScanBatchReceived(ScanStageBatch::Desktop(apps)));
            });

            let tx2 = tx.clone();
            let t2 = tokio::spawn(async move {
                let apps = scan_manual_installations().await;
                let _ = tx2.send(AsyncMessage::ScanBatchReceived(ScanStageBatch::Manual(apps)));
            });

            let tx3 = tx.clone();
            let t3 = tokio::spawn(async move {
                let apps = scan_snaps().await;
                let _ = tx3.send(AsyncMessage::ScanBatchReceived(ScanStageBatch::Snap(apps)));
            });

            let tx4 = tx.clone();
            let t4 = tokio::spawn(async move {
                let apps = scan_flatpaks().await;
                let _ = tx4.send(AsyncMessage::ScanBatchReceived(ScanStageBatch::Flatpak(apps)));
            });

            let tx5 = tx.clone();
            let t5 = tokio::spawn(async move {
                let pms = detect_available_package_managers();
                let mut native_apps = Vec::new();
                for pm in pms {
                    let mut a = pm.list_installed().await;
                    native_apps.append(&mut a);
                }
                let _ = tx5.send(AsyncMessage::ScanBatchReceived(ScanStageBatch::Native(native_apps)));
            });

            let _ = tokio::join!(t1, t2, t3, t4, t5);

            if let Ok(db) = Database::open_default() {
                if let Ok(history) = db.get_audit_history() {
                    let _ = tx.send(AsyncMessage::HistoryLoaded(history));
                }
            }

            let _ = tx.send(AsyncMessage::ScanFinished);
        });
    }

    pub fn trigger_sweep_scan(&mut self) {
        let tx = self.tx.clone();
        self.rt.spawn(async move {
            let items = scan_all_sweep_items().await;
            let _ = tx.send(AsyncMessage::SweepItemsScanned(items));
        });
    }

    pub fn trigger_sweep_clean(&mut self) {
        self.is_cleaning_sweep = true;
        let items_to_clean = self.sweep_items.clone();
        let tx = self.tx.clone();

        self.rt.spawn(async move {
            let report = execute_system_sweep(&items_to_clean).await;
            let _ = tx.send(AsyncMessage::SweepCleanComplete(report));
        });
    }

    fn handle_async_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AsyncMessage::ScanBatchReceived(batch) => {
                    self.scan_status = Some(format!("Discovered {}", batch.stage_name()));
                    apply_stage_batch(&mut self.app_map, batch);
                    self.apps = self.app_map.values().cloned().collect();
                    self.apps.sort_by(|a, b| a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase()));
                }
                AsyncMessage::ScanFinished => {
                    self.is_scanning = false;
                    self.scan_status = None;
                    if let Ok(mut db) = Database::open_default() {
                        let _ = db.save_apps(&self.apps);
                    }
                }
                AsyncMessage::UpdateAvailable(info) => {
                    self.available_update = Some(info);
                }
                AsyncMessage::UpdateComplete(msg) => {
                    self.is_updating_app = false;
                    self.update_message = Some(msg);
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
                    self.trigger_sweep_scan();
                }
                AsyncMessage::SweepItemsScanned(items) => {
                    self.sweep_items = items;
                }
                AsyncMessage::SweepCleanComplete(report) => {
                    self.is_cleaning_sweep = false;
                    self.last_sweep_report = Some(report);
                    self.trigger_sweep_scan();
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

        if self.is_scanning || self.is_updating_app {
            ctx.request_repaint();
        }

        if let Some(ref update) = self.available_update {
            egui::TopBottomPanel::top("update_banner").show(ctx, |ui| {
                ui.add_space(4.0_f32);
                egui::Frame::none()
                    .fill(Theme::PRIMARY_MUTED)
                    .inner_margin(8.0_f32)
                    .rounding(4.0_f32)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!(
                                    "🚀 Update Available: v{} → v{}",
                                    update.current_version, update.latest_version
                                ))
                                .strong()
                                .color(Color32::WHITE),
                            );

                            if let Some(ref url) = update.download_url {
                                let dl_url = url.clone();
                                let tx = self.tx.clone();
                                if ui
                                    .button(
                                        RichText::new(if self.is_updating_app {
                                            "⏳ Updating..."
                                        } else {
                                            "⚡ Install Update"
                                        })
                                        .color(Theme::BG_DARK)
                                        .strong(),
                                    )
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .clicked()
                                    && !self.is_updating_app
                                {
                                    self.is_updating_app = true;
                                    self.rt.spawn(async move {
                                        match crate::updater::perform_self_update(&dl_url).await {
                                            Ok(p) => {
                                                let _ = tx.send(AsyncMessage::UpdateComplete(format!(
                                                    "Updated to latest release at {}! Please restart SweepX.",
                                                    p.display()
                                                )));
                                            }
                                            Err(e) => {
                                                let _ = tx.send(AsyncMessage::UpdateComplete(format!(
                                                    "Update failed: {}",
                                                    e
                                                )));
                                            }
                                        }
                                    });
                                }
                            }

                            if ui
                                .button(RichText::new("🔗 View Release").color(Theme::TEXT_PRIMARY))
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .clicked()
                            {
                                let _ = open::that(&update.html_url);
                            }

                            if let Some(ref msg) = self.update_message {
                                ui.label(RichText::new(msg).color(Theme::ACCENT_SUCCESS).strong());
                            }
                        });
                    });
                ui.add_space(4.0_f32);
            });
        }

        egui::TopBottomPanel::top("top_header").show(ctx, |ui| {
            ui.add_space(8.0_f32);
            ui.horizontal(|ui| {
                ui.label(RichText::new("⚡ SweepX").size(20.0_f32).strong().color(Theme::PRIMARY));
                ui.label(RichText::new("Linux App Tracker & Deep Purge").size(13.0_f32).color(Theme::TEXT_MUTED));

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add(egui::Button::new(
                            if self.is_scanning { "⏳ Scanning..." } else { "🔄 Refresh Scan" },
                        ))
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        self.trigger_refresh();
                        self.trigger_sweep_scan();
                    }

                    ui.add_space(16.0_f32);

                    let tabs = [
                        (ActiveTab::History, "📜 History"),
                        (ActiveTab::Optimizer, "🧹 Optimizer"),
                        (ActiveTab::Dashboard, "📦 Applications"),
                    ];

                    for (tab, label) in tabs {
                        let is_active = self.active_tab == tab;
                        let text = if is_active {
                            RichText::new(label).strong().color(Color32::WHITE)
                        } else {
                            RichText::new(label).color(Theme::TEXT_SECONDARY)
                        };

                        let btn = egui::Button::new(text)
                            .fill(if is_active { Theme::PRIMARY } else { Theme::BG_CARD })
                            .stroke(if is_active {
                                Stroke::new(1.0_f32, Theme::PRIMARY_HOVER)
                            } else {
                                Stroke::new(1.0_f32, Theme::BORDER)
                            })
                            .rounding(6.0_f32);

                        if ui
                            .add(btn)
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
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
                        &mut self.sort_field,
                        &mut self.sort_direction,
                        &mut self.show_system_packages,
                        self.is_scanning,
                        self.scan_status.as_deref(),
                        &mut on_inspect,
                        &mut on_clean,
                    );
                }
                ActiveTab::Optimizer => {
                    let mut trigger_clean = false;
                    let mut trigger_refresh = false;
                    OptimizerView::render(
                        ui,
                        &mut self.sweep_items,
                        self.is_cleaning_sweep,
                        &self.last_sweep_report,
                        &mut trigger_clean,
                        &mut trigger_refresh,
                    );
                    if trigger_clean {
                        self.trigger_sweep_clean();
                    }
                    if trigger_refresh {
                        self.trigger_sweep_scan();
                    }
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
