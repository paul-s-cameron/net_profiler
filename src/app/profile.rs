use egui::{Color32, RichText};
use net_profiler::{DNS, check_valid_ipv4, check_valid_subnet};

use crate::app::NetworkProfile;

pub fn show_profile(ui: &mut egui::Ui, profile: &mut NetworkProfile) {
    show_ip_addresses_section(ui, profile);
    ui.add_space(5.0);
    ui.separator();
    
    show_gateways_section(ui, profile);
    ui.add_space(5.0);
    ui.separator();
    
    show_dns_section(ui, profile);
    ui.add_space(5.0);
}

fn show_ip_addresses_section(ui: &mut egui::Ui, profile: &mut NetworkProfile) {
    ui.heading("IP Addresses");

    let mut remove_indices = Vec::new();
    for (i, ip) in profile.ips.iter_mut().enumerate() {
        if show_ip_address_row(ui, ip) {
            remove_indices.push(i);
        }
    }

    remove_items_by_indices(&mut profile.ips, remove_indices);

    if ui.button("+").clicked() {
        profile.ips.push(("192.168.", "255.255.255.0").into());
    }
}

fn show_ip_address_row(ui: &mut egui::Ui, ip: &mut net_profiler::IP) -> bool {
    let mut should_remove = false;
    
    ui.horizontal(|ui| {
        ui.columns(3, |columns| {
            columns[0].with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let label = ui.label(RichText::new("IP: ").color(Color32::WHITE));
                
                ui.horizontal(|ui| {
                    // IP input field
                    ui.add_sized(
                        [ui.available_width() - 25.0, 20.0], // Reserve space for validation icon
                        egui::TextEdit::singleline(&mut ip.address)
                    ).labelled_by(label.id);
                    
                    // Check if IP is valid and show validation icon
                    let is_valid = check_valid_ipv4(&ip.address);
                    if !is_valid {
                        ui.label(RichText::new("❌").color(Color32::RED).size(16.0))
                            .on_hover_text("Invalid IP address format");
                    }
                });
            });
            columns[1].with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let label = ui.label(RichText::new("Subnet: ").color(Color32::WHITE));
                
                ui.horizontal(|ui| {
                    // Subnet input field
                    ui.add_sized(
                        [ui.available_width() - 25.0, 20.0], // Reserve space for validation icon
                        egui::TextEdit::singleline(&mut ip.subnet)
                    ).labelled_by(label.id);
                    
                    // Check if subnet is valid and show validation icon
                    let is_valid = check_valid_subnet(&ip.subnet);
                    if !is_valid {
                        ui.label(RichText::new("❌").color(Color32::RED).size(16.0))
                            .on_hover_text("Invalid subnet mask format (use dotted decimal like 255.255.255.0 or CIDR like /24)");
                    }
                });
            });
            columns[2].with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                if ui.button("remove").clicked() {
                    should_remove = true;
                }
            });
        });
    });
    
    should_remove
}

fn show_gateways_section(ui: &mut egui::Ui, profile: &mut NetworkProfile) {
    ui.heading("Gateways");

    let mut remove_indices = Vec::new();
    for (i, gateway) in profile.gateways.iter_mut().enumerate() {
        if show_gateway_row(ui, gateway, i) {
            remove_indices.push(i);
        }
    }

    remove_items_by_indices(&mut profile.gateways, remove_indices);

    if ui.button("+").clicked() {
        profile.gateways.push("".into());
    }
}

fn show_gateway_row(ui: &mut egui::Ui, gateway: &mut String, index: usize) -> bool {
    let mut should_remove = false;
    
    ui.horizontal(|ui| {
        ui.columns(2, |columns| {
            columns[0].with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let label = ui.label(RichText::new(format!("Gateway {}: ", index + 1)).color(Color32::WHITE));
                ui.text_edit_singleline(gateway).labelled_by(label.id);
            });
            columns[1].with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("remove").clicked() {
                    should_remove = true;
                }
            });
        });
    });
    
    should_remove
}

fn show_dns_section(ui: &mut egui::Ui, profile: &mut NetworkProfile) {
    ui.heading("DNS Provider");
    
    show_dns_selector(ui, profile);
    show_custom_dns_fields(ui, profile);
}

fn show_dns_selector(ui: &mut egui::Ui, profile: &mut NetworkProfile) {
    egui::ComboBox::from_id_salt("dns_selector")
        .selected_text(profile.dns.to_string())
        .show_ui(ui, |ui| {
            add_dns_option(ui, &mut profile.dns, DNS::None);
            add_dns_option_with_tooltip(ui, &mut profile.dns, DNS::Quad9, &DNS::QUAD9);
            add_dns_option_with_tooltip(ui, &mut profile.dns, DNS::Google, &DNS::GOOGLE);
            add_dns_option_with_tooltip(ui, &mut profile.dns, DNS::Cloudflare, &DNS::CLOUDFLARE);
            add_dns_option_with_tooltip(ui, &mut profile.dns, DNS::OpenDNS, &DNS::OPENDNS);
            
            ui.selectable_value(
                &mut profile.dns, 
                DNS::Custom { primary: "".into(), secondary: "".into() }, 
                "Custom"
            );
        });
}

fn add_dns_option(ui: &mut egui::Ui, current_dns: &mut DNS, option: DNS) {
    ui.selectable_value(current_dns, option.clone(), option.to_string());
}

fn add_dns_option_with_tooltip(ui: &mut egui::Ui, current_dns: &mut DNS, option: DNS, servers: &(&str, &str)) {
    ui.selectable_value(current_dns, option.clone(), option.to_string())
        .on_hover_text(RichText::new(format!("{}\n{}", servers.0, servers.1)));
}

fn show_custom_dns_fields(ui: &mut egui::Ui, profile: &mut NetworkProfile) {
    if let DNS::Custom { primary, secondary } = &mut profile.dns {
        let label = ui.label(RichText::new("Primary DNS: ").color(Color32::WHITE));
        ui.text_edit_singleline(primary).labelled_by(label.id);
        
        let label = ui.label(RichText::new("Secondary DNS: ").color(Color32::WHITE));
        ui.text_edit_singleline(secondary).labelled_by(label.id);
    }
}

fn remove_items_by_indices<T>(vec: &mut Vec<T>, indices: Vec<usize>) {
    for &i in indices.iter().rev() {
        vec.remove(i);
    }
}