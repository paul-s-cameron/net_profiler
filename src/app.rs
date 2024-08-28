use std::{collections::HashMap, default, path::PathBuf};

use eframe::egui;
use egui_file_dialog::FileDialog;
use egui::{Color32, RichText, Widget};
use network_interface::{NetworkInterface, NetworkInterfaceConfig};

use crate::network::{self, NetworkProfile};

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Default)]
#[serde(default)]
pub struct NetProfiler {
    pub profiles: HashMap<String, network::NetworkProfile>,
    #[serde(skip)]
    pub adapters: Vec<String>,

    // Private fields:
    #[serde(skip)]
    file_dialog: FileDialog,
    #[serde(skip)]
    import_export: bool, // 0 = import, 1 = export
    #[serde(skip)]
    builder: Option<network::NetworkProfile>,
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
        // Check for file dialog events
        self.file_dialog.update(ctx);
        if let Some(file_path) = self.file_dialog.take_selected() {
            if self.import_export {
                // Import the file
                if let Ok(profiles) = serde_json::from_str::<HashMap<String, network::NetworkProfile>>(&std::fs::read_to_string(&file_path).unwrap()) {
                    for (name, profile) in profiles {
                        self.profiles.insert(name, NetworkProfile {
                            adapter: String::new(),
                            ..profile
                        });
                    }
                }
            } else {
                // Remove adapter field from profiles
                let mut export_profiles: HashMap<String, NetworkProfile> = HashMap::new();
                for (name, profile) in self.profiles.iter() {
                    export_profiles.insert(name.clone(), NetworkProfile {
                        adapter: String::new(),
                        ..profile.clone()
                    });
                }

                // Export the file
                let file_path = PathBuf::from(file_path).with_extension("nprf");
                let profiles = serde_json::to_string(&export_profiles).unwrap();
                match std::fs::write(&file_path, profiles) {
                    Ok(_) => println!("File saved successfully"),
                    Err(e) => println!("Error saving file: {}", e),
                }
            }
        }

        // Profile Builder
        let mut finished = false;
        if let Some(ref mut builder) = self.builder.as_mut() {
            egui::Window::new("Profile Builder").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Profile Name:");
                    ui.text_edit_singleline(&mut builder.name);
                });

                display_profile(builder, ui, &self.adapters);

                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        self.profiles.insert(builder.name.clone(), builder.clone());
                        finished = true;
                    }
                    if ui.button("Cancel").clicked() {
                        finished = true;
                    }
                });
            });
        }
        if finished {
            self.builder = None;
        }


        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Import").clicked() {
                        self.import_export = true;
                        self.file_dialog.select_file();
                    }
                    if ui.button("Export").clicked() {
                        self.import_export = false;
                        self.file_dialog.save_file();
                    }
                });

                if ui.button("Add Profile").clicked() {
                    self.builder = Some(network::NetworkProfile {
                        name: "New Profile".to_string(),
                        subnet: "255.255.255.0".to_string(),
                        ..Default::default()
                    });
                }
            });
        });

        egui::CentralPanel::default().show(ctx, move |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut profiles_to_remove: Vec<NetworkProfile> = Vec::new();

                for (name, profile) in self.profiles.iter_mut() {
                    // Background Frame for padding and stylization
                    egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                        // Profile input fields
                        let open = egui::CollapsingHeader::new(RichText::new(name).color(Color32::WHITE))
                            .default_open(false)
                            .show(ui, |ui| {
                                egui::Frame::default()
                                    .inner_margin(egui::Margin::same(10.0))
                                    .show(ui, |ui| {
                                        display_profile(profile, ui, &self.adapters);
                                    });
                            })
                            .fully_open();

                        // Profile actions
                        egui::Frame::default()
                            .inner_margin(egui::Margin::same(4.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    if ui.button(RichText::new("Load Profile").color(Color32::WHITE)).clicked() {
                                        profile.load();
                                    }
                                    if ui.button(RichText::new("Remove Profile").color(Color32::WHITE)).double_clicked() {
                                        profiles_to_remove.push(profile.clone());
                                    }
                                });
                            });
                    });

                    ui.separator();
                }

                for profile in profiles_to_remove {
                    self.profiles.remove(&profile.name);
                }
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.label(format!("Net Profiler v{}", env!("CARGO_PKG_VERSION")));
        });
    }
}

fn display_profile(profile: &mut network::NetworkProfile, ui: &mut egui::Ui, adapters: &Vec<String>) {
    egui::ComboBox::from_label(RichText::new("Adapter").color(Color32::WHITE))
        .selected_text(&profile.adapter)
        .show_ui(ui, |ui| {
            for adapter in adapters.iter() {
                if ui.selectable_label(profile.adapter == *adapter, adapter).clicked() {
                    profile.adapter = adapter.clone();
                }
            }
        });
    
    ui.horizontal(|ui| {
        let label = ui.label(RichText::new("IP: ").color(Color32::WHITE));
        ui.text_edit_singleline(&mut profile.ip).labelled_by(label.id);
    });

    ui.separator();

    ui.horizontal(|ui| {
        let label = ui.label(RichText::new("Subnet: ").color(Color32::WHITE));
        ui.text_edit_singleline(&mut profile.subnet).labelled_by(label.id);
    });

    ui.separator();

    ui.horizontal(|ui| {
        let label = ui.label(RichText::new("Gateway: ").color(Color32::WHITE));
        ui.text_edit_singleline(&mut profile.gateway).labelled_by(label.id);
    });

    ui.separator();

    egui::Frame::default()
        .fill(Color32::from_rgb(30, 30, 30))
        .inner_margin(egui::Margin::same(2.0))
        .rounding(5.0)
        .show(ui, |ui| {
            let label = ui.label(RichText::new("DNS Provider: ").color(Color32::WHITE));
            ui.horizontal(|ui| {
                ui.radio_value(&mut profile.dns_provider, network::DNSProvider::None, "None");
                ui.radio_value(&mut profile.dns_provider, network::DNSProvider::Quad9, "Quad9").on_hover_ui(|ui| {
                    ui.style_mut().interaction.selectable_labels = true;
                    ui.label(RichText::new("9.9.9.9\n149.112.112.112\n(Recommended)").color(Color32::WHITE));
                }).labelled_by(label.id);
                ui.radio_value(&mut profile.dns_provider, network::DNSProvider::Google, "Google").on_hover_ui(|ui| {
                    ui.style_mut().interaction.selectable_labels = true;
                    ui.label(RichText::new("8.8.8.8\n8.8.4.4").color(Color32::WHITE));
                }).labelled_by(label.id);
                ui.radio_value(&mut profile.dns_provider, network::DNSProvider::Cloudflare, "Cloudflare").on_hover_ui(|ui| {
                    ui.style_mut().interaction.selectable_labels = true;
                    ui.label(RichText::new("1.1.1.2\n1.0.0.2").color(Color32::WHITE));
                }).labelled_by(label.id);
                ui.radio_value(&mut profile.dns_provider, network::DNSProvider::OpenDNS, "OpenDNS").on_hover_ui(|ui| {
                    ui.style_mut().interaction.selectable_labels = true;
                    ui.label(RichText::new("208.67.222.222\n208.67.220.220").color(Color32::WHITE));
                }).labelled_by(label.id);
                ui.radio_value(&mut profile.dns_provider, network::DNSProvider::Custom, "Custom");
            });
            if profile.dns_provider == network::DNSProvider::Custom {
                let label = ui.label(RichText::new("Primary DNS: ").color(Color32::WHITE));
                ui.text_edit_singleline(&mut profile.primary_dns).labelled_by(label.id);
                let label = ui.label(RichText::new("Secondary DNS: ").color(Color32::WHITE));
                ui.text_edit_singleline(&mut profile.secondary_dns).labelled_by(label.id);
            }
        });
}