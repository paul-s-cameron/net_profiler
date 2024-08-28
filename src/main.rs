#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused)]

use std::process::Command;

use network_interface::{NetworkInterface, NetworkInterfaceConfig};


mod app;
mod network;

fn main()  -> eframe::Result {
    let adapters: Vec<String> = NetworkInterface::show().unwrap().iter().map(|adapter| adapter.name.clone()).collect();
    println!("{:?}", adapters);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([480.0, 690.0])
            .with_min_inner_size([400.0, 400.0]),
            // .with_icon(
            //     // NOTE: Adding an icon is optional
            //     eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
            //         .expect("Failed to load icon"),
            // ),
        ..Default::default()
    };
    eframe::run_native(
        "Net Profiler",
        native_options,
        Box::new(|cc| {
            let mut app = app::NetProfiler::new(cc);
            app.adapters = adapters;
            Ok(Box::new(app))
        })
    )
}
