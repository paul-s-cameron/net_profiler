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
        if let Some(ref mut builder) = self.builder {
            egui::Window::new("Profile Builder").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Name:");
                    ui.text_edit_singleline(&mut builder.name);
                });

                ui.separator();

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
                        ips: vec![
                            ("192.168.0.100", "255.255.255.0").into()
                        ],
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
                    egui::Frame::default().show(ui, |ui| {
                        // Profile input fields
                        egui::CollapsingHeader::new(RichText::new(name).color(Color32::WHITE).strong().size(18.))
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
                                    if ui.button(RichText::new("Load").color(Color32::WHITE).size(14.)).clicked() {
                                        profile.load();
                                    }
                                    if ui.button(RichText::new("Remove").color(Color32::WHITE).size(14.))
                                        .on_hover_text("Double Click to delete this profile")
                                        .double_clicked() {
                                        profiles_to_remove.push(profile.clone());
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
                    self.profiles.remove(&profile.name);
                }
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.label(format!("Net Profiler v{} by Paul Cameron", env!("CARGO_PKG_VERSION")));
        });
    }
}

fn display_profile(profile: &mut network::NetworkProfile, ui: &mut egui::Ui, adapters: &Vec<String>) {
    ui.horizontal(|ui| {
        ui.heading("Adapter");
        egui::ComboBox::from_id_source("adapter")
            .selected_text(&profile.adapter)
            .show_ui(ui, |ui| {
                for adapter in adapters.iter() {
                    if ui.selectable_label(profile.adapter == *adapter, adapter).clicked() {
                        profile.adapter = adapter.clone();
                    }
                }
            }
        );
    });

    ui.separator();
    
    ui.heading("IP Addresses");

    let mut remove_index: Vec<usize> = Vec::new();
    for (i, ip) in profile.ips.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.columns(3, |columns| {
                columns[0].with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    let label = ui.label(RichText::new("IP: ").color(Color32::WHITE));
                    ui.text_edit_singleline(&mut ip.address).labelled_by(label.id);
                });
                columns[1].with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    let label = ui.label(RichText::new("Subnet: ").color(Color32::WHITE));
                    ui.text_edit_singleline(&mut ip.subnet).labelled_by(label.id);
                });
                columns[2].with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("remove").clicked() {
                        remove_index.push(i);
                    }
                });
            });
        });
    }
    for i in remove_index.iter() {
        profile.ips.remove(*i);
    }

    if ui.button("+").clicked() {
        profile.ips.push(("","").into());
    }

    ui.add_space(5.0);
    ui.separator();

    ui.heading("Gateways");

    let mut remove_index: Vec<usize> = Vec::new();
    for (i, gateway) in profile.gateways.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.columns(2, |columns| {
                columns[0].with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    let label = ui.label(RichText::new(format!("Gateway {}: ", i+1)).color(Color32::WHITE));
                    ui.text_edit_singleline(gateway).labelled_by(label.id);
                });
                columns[1].with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("remove").clicked() {
                        remove_index.push(i);
                    }
                });
            });
        });
    }
    for i in remove_index.iter() {
        profile.gateways.remove(*i);
    }

    if ui.button("+").clicked() {
        profile.gateways.push("".into());
    }

    ui.add_space(5.0);
    ui.separator();

    let label = ui.heading("DNS Provider");
    ui.horizontal(|ui| {
        ui.radio_value(&mut profile.dns_provider, network::DNSProvider::DHCP, "DHCP");
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
        ui.text_edit_singleline(&mut profile.custom_dns.primary).labelled_by(label.id);
        let label = ui.label(RichText::new("Secondary DNS: ").color(Color32::WHITE));
        ui.text_edit_singleline(&mut profile.custom_dns.secondary).labelled_by(label.id);
    }
    ui.add_space(5.0);
}