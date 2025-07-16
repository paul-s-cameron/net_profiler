use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};

use net_profiler::{load_profile, NetworkProfile};

#[derive(Debug, Default)]
pub struct ProfileLoader {
    visible: bool,
    interfaces: Vec<NetworkInterface>,
    selected_interface: Option<NetworkInterface>,
    profile: NetworkProfile,
    last_result: Option<Result<(), String>>,
}

impl ProfileLoader {
    pub fn update(&mut self, ctx: &egui::Context) {
        if !self.visible {
            return;
        }

        egui::Window::new("profile_loader")
            .title_bar(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, (0., 0.))
            .show(ctx, |ui| {
                egui::Frame::default()
                    .inner_margin(6.)
                    .show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                            self.show_interface_selector(ui);
                            self.show_profile_configuration(ui);
                            self.show_action_buttons(ui);
                        });
                    });
            });
    }

    fn show_interface_selector(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_id_salt("interface_selector")
            .width(ui.available_width())
            .selected_text(
                self.selected_interface
                    .as_ref()
                    .map_or("Select an interface".to_string(), |i| i.name.clone()),
            )
            .show_ui(ui, |ui| {
                for interface in &self.interfaces {
                    ui.selectable_value(
                        &mut self.selected_interface,
                        Some(interface.clone()),
                        &interface.name,
                    );
                }
            });
    }

    fn show_profile_configuration(&mut self, ui: &mut egui::Ui) {
        let Some(interface) = &self.selected_interface else {
            return;
        };

        egui::Frame::default()
            .inner_margin(6.)
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    self.show_profile_details(ui);
                    self.show_current_configuration(ui, interface);
                });
            });
    }

    fn show_profile_details(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new(format!("'{}' Configuration", self.profile.name))
            .default_open(true)
            .show(ui, |ui| {
                self.show_profile_ip_addresses(ui);
                self.show_profile_gateways(ui);
                ui.label(format!("DNS: {}", self.profile.dns.to_string()));
            });
    }

    fn show_profile_ip_addresses(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("IP Addresses")
            .default_open(true)
            .show(ui, |ui| {
                for ip in &self.profile.ips {
                    ui.label(format!("IP: {}, Mask: {}", ip.address, ip.subnet));
                }
            });
    }

    fn show_profile_gateways(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Gateways")
            .default_open(true)
            .show(ui, |ui| {
                for gateway in &self.profile.gateways {
                    ui.label(format!("Gateway: {}", gateway));
                }
            });
    }

    fn show_current_configuration(&self, ui: &mut egui::Ui, interface: &NetworkInterface) {
        egui::CollapsingHeader::new("Current Configuration")
            .default_open(true)
            .show(ui, |ui| {
                self.show_current_ip_addresses(ui, interface);
            });
    }

    fn show_current_ip_addresses(&self, ui: &mut egui::Ui, interface: &NetworkInterface) {
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
            });
    }

    fn show_action_buttons(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {
            columns[0].with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let button = ui.add_sized([ui.available_width(), 30.0], egui::Button::new("Apply"));
                if button.clicked() {
                    let result = self.apply_profile();
                    self.last_result = Some(result);
                    if self.last_result.as_ref().unwrap().is_ok() {
                        self.close();
                    }
                }
            });
            columns[1].with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let button = ui.add_sized([ui.available_width(), 30.0], egui::Button::new("Cancel"));
                if button.clicked() {
                    self.close();
                }
            });
        });
    }

    fn apply_profile(&self) -> Result<(), String> {
        if let Some(interface) = &self.selected_interface {
            load_profile(&self.profile, &interface.name)
                .map_err(|e| e.to_string())
        } else {
            Err("No interface selected".to_string())
        }
    }

    fn close(&mut self) {
        self.visible = false;
        self.selected_interface = None;
    }

    pub fn load_profile(&mut self, profile: &NetworkProfile) {
        self.profile = profile.clone();
        self.interfaces = NetworkInterface::show().unwrap_or_default();
        self.visible = true;
        self.last_result = None; // Clear any previous result
    }

    pub fn take_last_result(&mut self) -> Option<Result<(), String>> {
        self.last_result.take()
    }
}