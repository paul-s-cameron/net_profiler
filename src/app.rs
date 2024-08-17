use std::{default, path::PathBuf};

use eframe::egui;
use egui_file_dialog::FileDialog;
use egui::{ahash::HashMap, Widget};
use network_interface::{NetworkInterface, NetworkInterfaceConfig};

use crate::network::{self, NetworkProfile};

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Default)]
#[serde(default)]
pub struct NetProfiler {
    pub profiles: HashMap<String, network::NetworkProfile>,

    // Private fields:
    #[serde(skip)]
    file_dialog: FileDialog,
    #[serde(skip)]
    import_export: bool, // 0 = import, 1 = export
    #[serde(skip)]
    profile_builder: bool,
    #[serde(skip)]
    builder: network::NetworkProfile,
    #[serde(skip)]
    pub adapters: Vec<String>,
}

impl NetProfiler {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
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
        // Profile Builder
        if self.profile_builder {
            egui::Window::new("Profile Builder").show(ctx, |ui| {
                display_profile(&mut self.builder, ui, &self.adapters);

                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        self.profiles.insert(self.builder.name.clone(), self.builder.clone());
                        self.profile_builder = false;
                    }
                    if ui.button("Cancel").clicked() {
                        self.profile_builder = false;
                    }
                });
            });
        }


        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Import").clicked() {
                        // self.import_export = true;
                        self.file_dialog.select_file();
                    }
                    if ui.button("Export").clicked() {
                        // self.import_export = false;
                        self.file_dialog.save_file();
                    }
                });
                if ui.button("Add Profile").clicked() {
                    self.builder = network::NetworkProfile {
                        name: "New Profile".to_string(),
                        subnet: "255.255.255.0".to_string(),
                        ..Default::default()
                    };
                    self.profile_builder = true;
                }
            });

            // Check for file dialog events
            self.file_dialog.update(ctx);
            if let Some(file_path) = self.file_dialog.take_selected() {
                // Set the file extension to .nprf
                let file_path = PathBuf::from(file_path).with_extension("nprf");
                if self.import_export {
                    // Import the file
                    if let Ok(profiles) = serde_json::from_str::<HashMap<String, network::NetworkProfile>>(&std::fs::read_to_string(&file_path).unwrap()) {
                        for (name, profile) in profiles {
                            self.profiles.insert(name, profile);
                        }
                    }
                } else {
                    // Export the file
                    let profiles = serde_json::to_string(&self.profiles).unwrap();
                    std::fs::write(&file_path, profiles).unwrap();
                }
            }
        });

        egui::CentralPanel::default().show(ctx, move |ui| {
            // The central panel contains all the profiles created by the user
            let mut profiles_to_remove: Vec<NetworkProfile> = Vec::new();
            for (name, profile) in self.profiles.iter_mut() {
                egui::CollapsingHeader::new(name)
                    .default_open(false)
                    .show(ui, |ui| {
                        display_profile(profile, ui, &self.adapters);

                        ui.horizontal(|ui| {
                            if ui.button("Load Profile").clicked() {
                                profile.load();
                            }
                            if ui.button("Remove Profile").double_clicked() {
                                profiles_to_remove.push(profile.clone());
                            }
                        });
                    });
            }
            for profile in profiles_to_remove {
                self.profiles.remove(&profile.name);
            }
        });
    }
}

fn display_profile(profile: &mut network::NetworkProfile, ui: &mut egui::Ui, adapters: &Vec<String>) {
    ui.horizontal(|ui| {
        ui.label("Profile Name:");
        ui.text_edit_singleline(&mut profile.name);
    });
    egui::ComboBox::from_label("Adapter")
        .selected_text(&profile.adapter)
        .show_ui(ui, |ui| {
            for adapter in adapters.iter() {
                if ui.selectable_label(profile.adapter == *adapter, adapter).clicked() {
                    profile.adapter = adapter.clone();
                }
            }
        });
    ui.label("IP: ");
    ui.text_edit_singleline(&mut profile.ip);
    ui.label("Subnet: ");
    ui.text_edit_singleline(&mut profile.subnet);
    ui.label("Gateway: ");
    ui.text_edit_singleline(&mut profile.gateway);
    ui.label("DNS Provider: ");
    ui.horizontal(|ui| {
        ui.radio_value(&mut profile.dns_provider, network::DNSProvider::Quad9, "Quad9");
        ui.radio_value(&mut profile.dns_provider, network::DNSProvider::Google, "Google");
        ui.radio_value(&mut profile.dns_provider, network::DNSProvider::Cloudflare, "Cloudflare");
        ui.radio_value(&mut profile.dns_provider, network::DNSProvider::OpenDNS, "OpenDNS");
        ui.radio_value(&mut profile.dns_provider, network::DNSProvider::Custom, "Custom");
    });
    if profile.dns_provider == network::DNSProvider::Custom {
        ui.label("Primary DNS: ");
        ui.text_edit_singleline(&mut profile.primary_dns);
        ui.label("Secondary DNS: ");
        ui.text_edit_singleline(&mut profile.secondary_dns);
    }
}