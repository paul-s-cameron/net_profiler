use std::path::PathBuf;

use eframe::egui;
use egui_file_dialog::{DialogMode, FileDialog};
use egui::{Color32, RichText};
use loader::ProfileLoader;

mod profile;
use profile::show_profile;
mod loader;

use net_profiler::NetworkProfile;

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Default)]
#[serde(default)]
pub struct NetProfiler {
    pub profiles: Vec<NetworkProfile>,

    #[serde(skip)]
    file_dialog: FileDialog,
    #[serde(skip)]
    builder: Option<NetworkProfile>,
    #[serde(skip)]
    loader: ProfileLoader,
}

impl NetProfiler {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for NetProfiler {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.file_dialog.update(ctx);
        self.loader.update(ctx);

        // Profile import and export
        if let Some(file_path) = self.file_dialog.take_selected() {
            match self.file_dialog.mode() {
                DialogMode::SelectFile => {
                    if let Ok(mut profiles) = serde_json::from_str(&std::fs::read_to_string(&file_path).unwrap()) {
                        self.profiles.append(&mut profiles);
                    }
                }
                DialogMode::SaveFile => {
                    let file_path = PathBuf::from(file_path).with_extension("nprf");
                    let profiles = serde_json::to_string(&self.profiles).unwrap();
                    match std::fs::write(&file_path, profiles) {
                        Ok(_) => println!("File saved successfully"),
                        Err(e) => println!("Error saving file: {}", e),
                    }
                }
                _ => {}
            }
        }

        // Profile Builder
        let mut finished = false;
        if let Some(ref mut builder) = self.builder {
            egui::Window::new("Profile Builder")
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Name:");
                        ui.text_edit_singleline(&mut builder.name);
                    });

                    ui.separator();

                    show_profile(ui, builder);

                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked() {
                            self.profiles.push(builder.clone());
                            finished = true;
                        }
                        if ui.button("Cancel").clicked() {
                            finished = true;
                        }
                    });
                }
            );
        }
        if finished {
            self.builder = None;
        }


        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Import").clicked() {
                        self.file_dialog.select_file();
                    }
                    if ui.button("Export").clicked() {
                        self.file_dialog.save_file();
                    }
                });

                if ui.button("Add Profile").clicked() {
                    self.builder = Some(NetworkProfile {
                        name: "New Profile".to_string(),
                        ips: vec![
                            ("192.168.", "255.255.255.0").into()
                        ],
                        ..Default::default()
                    });
                }
            });
        });

        egui::CentralPanel::default().show(ctx, move |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut profiles_to_remove: Vec<usize> = Vec::new();

                for (i, profile) in self.profiles.iter_mut().enumerate() {
                    // Background Frame for padding and stylization
                    egui::Frame::default().show(ui, |ui| {
                        // Profile input fields
                        egui::CollapsingHeader::new(RichText::new(&profile.name).color(Color32::WHITE).strong().size(18.))
                            .default_open(false)
                            .show(ui, |ui| {
                                egui::Frame::default()
                                    .inner_margin(egui::Margin::same(10.0))
                                    .show(ui, |ui| {
                                        show_profile(ui, profile);
                                    });
                            })
                            .fully_open();

                        // Profile actions
                        egui::Frame::default()
                            .inner_margin(egui::Margin::same(4.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    if ui.button(RichText::new("Load").color(Color32::WHITE).size(14.)).clicked() {
                                        self.loader.load_profile(profile);
                                    }
                                    if ui.button(RichText::new("Remove").color(Color32::WHITE).size(14.))
                                        .on_hover_text("Double Click to delete this profile")
                                        .double_clicked() {
                                        profiles_to_remove.push(i);
                                    }
                                    if ui.button(RichText::new("Clone").color(Color32::WHITE).size(14.)).clicked() {
                                        self.builder = Some(profile.clone())
                                    }
                                    ui.add_space(ui.available_width());
                                });
                            });
                    });

                    ui.separator();
                }

                for profile in profiles_to_remove {
                    self.profiles.remove(profile);
                }
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.label(format!("Net Profiler v{} by Paul Cameron", env!("CARGO_PKG_VERSION")));
        });
    }
}
