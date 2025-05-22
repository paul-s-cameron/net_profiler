use network_interface::{NetworkInterface, NetworkInterfaceConfig};

use net_profiler::{
    NetworkProfile,
    check_valid_ipv4
};

#[derive(Debug, Default)]
pub struct ProfileLoader {
    visible: bool,
    interfaces: Vec<NetworkInterface>,
    selected_interface: Option<NetworkInterface>,
    profile: NetworkProfile,
}

impl ProfileLoader {
    pub fn update(&mut self, ctx: &egui::Context) {
        if !self.visible { return; }

        egui::Window::new("profile_loader")
            .title_bar(false)
            .show(ctx, |ui| {
                egui::TopBottomPanel::top("interface_selector")
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Select Interface:");

                            egui::ComboBox::from_id_source("interface_selector")
                                .selected_text(self.selected_interface.as_ref().map_or("None".to_string(), |i| i.name.clone()))
                                .show_ui(ui, |ui| {
                                    for interface in &self.interfaces {
                                        ui.selectable_value(&mut self.selected_interface, Some(interface.clone()), &interface.name);
                                    }
                                }
                            );
                        });
                        ui.add_space(10.);
                    }
                );
                egui::CentralPanel::default()
                    .show_inside(ui, |ui| {
                        ui.horizontal(|ui| {
                            //TODO: Show information about the selected interface
                        });
                    }
                );
                egui::TopBottomPanel::bottom("menu")
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        ui.add_space(10.);
                        ui.horizontal(|ui| {
                            if ui.button("Load").clicked() {
                                self.hide();
                            }
                            if ui.button("Cancel").clicked() {
                                self.hide();
                            }
                        });
                    }
                );
            }
        );
    }

    fn hide(&mut self) {
        self.visible = false;
        self.selected_interface = None;
    }

    pub fn load_profile(&mut self, profile: &NetworkProfile) {
        self.profile = profile.clone();

        self.interfaces = match NetworkInterface::show() {
            Ok(interfaces) => interfaces,
            Err(_) => vec![]
        };

        self.visible = true;
    }
}