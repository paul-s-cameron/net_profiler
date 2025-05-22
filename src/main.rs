#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod app;

///TODO: 
/// 1. Abstract network commands for Windows and Linux
/// 2. Remove adapter from profile and create popup to select adapter to apply the profile to

fn main()  -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([480.0, 690.0])
            .with_min_inner_size([400.0, 400.0]),
            // .with_icon(
            //     eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
            //         .expect("Failed to load icon"),
            // ),
        ..Default::default()
    };
    eframe::run_native(
        "Net Profiler",
        native_options,
        Box::new(|cc| Ok(Box::new(app::NetProfiler::new(cc))))
    )
}
