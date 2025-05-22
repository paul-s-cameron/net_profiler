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
            .fixed_rect(
                egui::Rect::from_center_size(
                    ctx.screen_rect().center(),
                    egui::Vec2::new(ctx.screen_rect().size().x / 2., 80.)
                )
            )
            .show(ctx, |ui| {
                egui::Frame::default()
                    .inner_margin(6.)
                    .show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                            egui::ComboBox::from_id_source("interface_selector")
                                .width(ui.available_width())
                                .selected_text(self.selected_interface.as_ref().map_or("Select an interface".to_string(), |i| i.name.clone()))
                                .show_ui(ui, |ui| {
                                    for interface in &self.interfaces {
                                        ui.selectable_value(&mut self.selected_interface, Some(interface.clone()), &interface.name);
                                    }
                                }
                            );
                        
                            ui.columns(2, |columns| {
                                columns[0].with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                                    if ui.button("Apply").clicked() {
                                        
                                    }
                                });
                                columns[1].with_layout(egui::Layout::centered_and_justified(egui::Direction::RightToLeft), |ui| {
                                    if ui.button("Cancel").clicked() {
                                        self.hide();
                                    }
                                });
                            });
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