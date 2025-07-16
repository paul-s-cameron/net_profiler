use std::vec;

use eframe::egui;
use egui::{Color32, RichText};
use egui_file_dialog::{DialogMode, FileDialog};
use egui_toast::{Toast, Toasts, ToastKind};

mod profile;
use profile::show_profile;
mod loader;
use loader::ProfileLoader;
mod file_operations;
use file_operations::{import_profiles_from_file, export_profiles_to_file};

use net_profiler::NetworkProfile;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct NetProfiler {
    pub profiles: Vec<NetworkProfile>,

    #[serde(skip)]
    file_dialog: FileDialog,
    #[serde(skip)]
    builder: Option<NetworkProfile>,
    #[serde(skip)]
    loader: ProfileLoader,
    #[serde(skip)]
    toasts: Toasts,
}

impl Default for NetProfiler {
    fn default() -> Self {
        let file_dialog = FileDialog::default();
        let loader = ProfileLoader::default();
        let toasts = Toasts::new()
            .anchor(egui::Align2::RIGHT_TOP, (-10., 10.))
            .direction(egui::Direction::TopDown);
        
        Self {
            profiles: vec![],
            file_dialog,
            builder: None,
            loader,
            toasts,
        }
    }
}

impl NetProfiler {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            log::info!("Storage found, attempting to load app state");
            
            match eframe::get_value::<NetProfiler>(storage, eframe::APP_KEY) {
                Some(loaded_app) => {
                    return loaded_app;
                }
                None => {
                    log::info!("No saved app state found, using default");
                }
            }
        } else {
            log::info!("No storage available, using default");
        }

        Default::default()
    }
}

impl NetProfiler {
    fn handle_file_dialog(&mut self) {
        let Some(file_path) = self.file_dialog.take_picked() else {
            return;
        };

        match self.file_dialog.mode() {
            DialogMode::PickFile => self.import_profiles(file_path),
            DialogMode::SaveFile => self.export_profiles(file_path),
            _ => {}
        }
    }

    fn import_profiles(&mut self, file_path: std::path::PathBuf) {
        match import_profiles_from_file(file_path) {
            Ok(mut profiles) => {
                self.profiles.append(&mut profiles);
                self.show_success_toast("Successfully imported profiles");
            }
            Err(error_message) => {
                log::error!("{}", error_message);
                self.show_error_toast(&error_message);
            }
        }
    }

    fn export_profiles(&mut self, file_path: std::path::PathBuf) {
        match export_profiles_to_file(&self.profiles, file_path) {
            Ok(_) => self.show_success_toast("Successfully saved profiles"),
            Err(error_message) => {
                log::error!("{}", error_message);
                self.show_error_toast(&error_message);
            }
        }
    }

    fn show_success_toast(&mut self, message: &str) {
        self.toasts.add(Toast {
            kind: ToastKind::Success,
            text: message.into(),
            ..Default::default()
        });
    }

    fn show_error_toast(&mut self, message: &str) {
        self.toasts.add(Toast {
            kind: ToastKind::Error,
            text: message.into(),
            ..Default::default()
        });
    }

    fn handle_profile_builder(&mut self, ctx: &egui::Context) -> bool {
        let Some(ref mut builder) = self.builder else {
            return false;
        };

        let mut finished = false;
        let mut should_create = false;
        
        egui::Window::new("Profile Builder")
            .collapsible(false)
            .default_width(ctx.available_rect().width() * 0.8)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Name:");
                    ui.text_edit_singleline(&mut builder.name);
                });

                ui.separator();
                show_profile(ui, builder);

                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        should_create = true;
                        finished = true;
                    }
                    if ui.button("Cancel").clicked() {
                        finished = true;
                    }
                });
            });

        if should_create {
            self.profiles.push(builder.clone());
            self.show_success_toast("Successfully created profile");
        }

        finished
    }

    fn show_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Import").clicked() {
                        self.file_dialog.pick_file();
                    }
                    if ui.button("Export").clicked() {
                        self.file_dialog.save_file();
                    }
                });

                if ui.button("Add Profile").clicked() {
                    self.builder = Some(NetworkProfile {
                        name: "New Profile".to_string(),
                        ips: vec![("192.168.", "255.255.255.0").into()],
                        ..Default::default()
                    });
                }
            });
        });
    }

    fn show_profiles_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut profiles_to_remove: Vec<usize> = Vec::new();
                let mut profile_to_load: Option<NetworkProfile> = None;
                let mut profile_to_clone: Option<NetworkProfile> = None;

                for (i, profile) in self.profiles.iter_mut().enumerate() {
                    let (should_remove, should_load, should_clone) = Self::show_profile_item_inner(ui, profile);
                    
                    if should_remove {
                        profiles_to_remove.push(i);
                    }
                    if should_load {
                        profile_to_load = Some(profile.clone());
                    }
                    if should_clone {
                        profile_to_clone = Some(profile.clone());
                    }
                    
                    ui.separator();
                }

                // Handle actions that need mutable access to self
                if let Some(profile) = profile_to_load {
                    self.loader.load_profile(&profile);
                }
                if let Some(profile) = profile_to_clone {
                    self.builder = Some(profile);
                }

                // Remove profiles in reverse order to avoid index shifting
                for &index in profiles_to_remove.iter().rev() {
                    self.profiles.remove(index);
                }
            });
        });
    }

    fn show_profile_item_inner(ui: &mut egui::Ui, profile: &mut NetworkProfile) -> (bool, bool, bool) {
        let mut should_remove = false;
        let mut should_load = false;
        let mut should_clone = false;

        egui::Frame::default().show(ui, |ui| {
            egui::CollapsingHeader::new(
                RichText::new(&profile.name)
                    .color(Color32::WHITE)
                    .strong()
                    .size(18.),
            )
            .default_open(false)
            .show(ui, |ui| {
                egui::Frame::default()
                    .inner_margin(egui::Margin::same(10))
                    .show(ui, |ui| {
                        show_profile(ui, profile);
                    });
            });

            egui::Frame::default()
                .inner_margin(egui::Margin::same(4))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Load").color(Color32::WHITE).size(14.)).clicked() {
                            should_load = true;
                        }
                        if ui.button(RichText::new("Remove").color(Color32::WHITE).size(14.))
                            .on_hover_text("Double Click to delete this profile")
                            .double_clicked()
                        {
                            should_remove = true;
                        }
                        if ui.button(RichText::new("Clone").color(Color32::WHITE).size(14.)).clicked() {
                            should_clone = true;
                        }
                        ui.add_space(ui.available_width());
                    });
                });
        });

        (should_remove, should_load, should_clone)
    }

    fn show_bottom_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.add_space(1.);
            ui.horizontal(|ui| {
                ui.label(format!("Net Profiler v{} by Paul Cameron", env!("CARGO_PKG_VERSION")));
                if ui.link("source").clicked() {
                    open::that(env!("CARGO_PKG_REPOSITORY")).unwrap_or_else(|_| {
                        log::error!("Failed to open repository link");
                    });
                }
            });
        });
    }
}

impl eframe::App for NetProfiler {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.file_dialog.update(ctx);
        self.loader.update(ctx);

        // Handle loader results
        if let Some(result) = self.loader.take_last_result() {
            match result {
                Ok(()) => {
                    self.show_success_toast("Profile applied successfully");
                }
                Err(e) => {
                    self.show_error_toast(&format!("Failed to apply profile: {}", e));
                }
            }
        }

        self.handle_file_dialog();

        if self.handle_profile_builder(ctx) {
            self.builder = None;
        }

        self.show_top_panel(ctx);
        self.show_profiles_panel(ctx);
        self.show_bottom_panel(ctx);
        self.toasts.show(ctx);
    }
}
