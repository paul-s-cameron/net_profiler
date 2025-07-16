use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};

use net_profiler::{
    check_valid_ipv4, load_profile, NetworkProfile
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
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, (0., 0.))
            .show(ctx, |ui| {
                egui::Frame::default()
                    .inner_margin(6.)
                    .show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                            egui::ComboBox::from_id_salt("interface_selector")
                                .width(ui.available_width())
                                .selected_text(self.selected_interface.as_ref().map_or("Select an interface".to_string(), |i| i.name.clone()))
                                .show_ui(ui, |ui| {
                                    for interface in &self.interfaces {
                                        ui.selectable_value(&mut self.selected_interface, Some(interface.clone()), &interface.name);
                                    }
                                }
                            );
                            if let Some(interface) = &self.selected_interface {
                                egui::Frame::default()
                                    .inner_margin(6.)
                                    .show(ui, |ui| {
                                        ui.vertical_centered(|ui| {
                                            egui::CollapsingHeader::new(format!("'{}' Configuration", self.profile.name))
                                                .default_open(true)
                                                .show(ui, |ui| {
                                                    ui.label(format!("Profile: {}", self.profile.name));

                                                    egui::CollapsingHeader::new("IP Addresses")
                                                        .default_open(true)
                                                        .show(ui, |ui| {
                                                            for ip in &self.profile.ips {
                                                                ui.label(format!("IP: {}, Mask: {}", ip.address, ip.subnet));
                                                            }
                                                        }
                                                    );
                                                    
                                                    egui::CollapsingHeader::new("Gateways")
                                                        .default_open(true)
                                                        .show(ui, |ui| {
                                                            for gateway in &self.profile.gateways {
                                                                ui.label(format!("Gateway: {}", gateway));
                                                            }
                                                        }
                                                    );

                                                    ui.label(format!("DNS: {}", self.profile.dns.to_string()));
                                                }
                                            );
                                            egui::CollapsingHeader::new("Current Configuration")
                                                .default_open(true)
                                                .show(ui, |ui| {
                                                    egui::CollapsingHeader::new("IP Addresses")
                                                        .default_open(true)
                                                        .show(ui, |ui| {
                                                            for addr in &interface.addr {
                                                                match addr {
                                                                    Addr::V4(if_addr) => {
                                                                        ui.label(format!(
                                                                            "Ipv4: {}, Mask: {}",
                                                                            if_addr.ip,
                                                                            if_addr.netmask.map_or("None".to_string(), |v| v.to_string())
                                                                        ));
                                                                    }
                                                                    Addr::V6(if_addr) => {
                                                                        ui.label(format!(
                                                                            "Ipv6: {}, Mask: {}",
                                                                            if_addr.ip,
                                                                            if_addr.netmask.map_or("None".to_string(), |v| v.to_string())
                                                                        ));
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    );
                                                }
                                            );
                                        });
                                    }
                                );
                            }
                            ui.columns(2, |columns| {
                                columns[0].with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                    let button = ui.add_sized(
                                        [ui.available_width(), 30.0],
                                        egui::Button::new("Apply")
                                    );
                                    if button.clicked() {
                                        if let Some(interface) = &self.selected_interface {
                                            load_profile(&self.profile, &interface.name);
                                        }
                                    }
                                });
                                columns[1].with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let button = ui.add_sized(
                                        [ui.available_width(), 30.0],
                                        egui::Button::new("Cancel")
                                    );
                                    if button.clicked() {
                                        self.close();
                                    }
                                });
                            });
                        });
                    }
                );
            }
        );
    }

    fn close(&mut self) {
        self.visible = false;
        self.selected_interface = None;
    }

    pub fn load_profile(&mut self, profile: &NetworkProfile) {
        //TODO: Check for valid ips and configuration

        self.profile = profile.clone();

        self.interfaces = match NetworkInterface::show() {
            Ok(interfaces) => interfaces,
            Err(_) => vec![]
        };

        self.visible = true;
    }
}