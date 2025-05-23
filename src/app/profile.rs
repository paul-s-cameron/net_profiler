use egui::{Color32, RichText};
use net_profiler::DNS;

use crate::app::NetworkProfile;

pub fn show_profile(ui: &mut egui::Ui, profile: &mut NetworkProfile) {
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
                columns[2].with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
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
    egui::ComboBox::from_id_source("dns_selector")
        .selected_text(profile.dns.to_string())
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut profile.dns, DNS::DHCP, DNS::DHCP.to_string());
            ui.selectable_value(&mut profile.dns, DNS::Quad9, DNS::Quad9.to_string())
                .on_hover_text(RichText::new(format!("{}\n{}", DNS::QUAD9.0, DNS::QUAD9.1)));
            ui.selectable_value(&mut profile.dns, DNS::Google, DNS::Google.to_string())
                .on_hover_text(RichText::new(format!("{}\n{}", DNS::GOOGLE.0, DNS::GOOGLE.1)));
            ui.selectable_value(&mut profile.dns, DNS::Cloudflare, DNS::Cloudflare.to_string())
                .on_hover_text(RichText::new(format!("{}\n{}", DNS::CLOUDFLARE.0, DNS::CLOUDFLARE.1)));
            ui.selectable_value(&mut profile.dns, DNS::OpenDNS, DNS::OpenDNS.to_string())
                .on_hover_text(RichText::new(format!("{}\n{}", DNS::OPENDNS.0, DNS::OPENDNS.1)));
            ui.selectable_value(&mut profile.dns, DNS::Custom { primary: "".into(), secondary: "".into() }, "Custom");
        }
    );
    
    match &mut profile.dns {
        DNS::Custom { primary, secondary } => {
            let label = ui.label(RichText::new("Primary DNS: ").color(Color32::WHITE));
            ui.text_edit_singleline(primary).labelled_by(label.id);
            let label = ui.label(RichText::new("Secondary DNS: ").color(Color32::WHITE));
            ui.text_edit_singleline(secondary).labelled_by(label.id);
        }
        _ => {}
    }
    ui.add_space(5.0);
}