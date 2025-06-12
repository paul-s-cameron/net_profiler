#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod app;

///TODO: 
/// 1. Abstract network commands for Windows and Linux
/// *2. Remove adapter from profile and create popup to select adapter to apply the profile to
/// *3. Setup egui_toast for notifications
/// 4. Logging to file

fn main() {
    simple_logging::log_to_file("net_profiler.log", log::LevelFilter::Info)
        .expect("Failed to initialize logging");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "Net Profiler",
        native_options,
        Box::new(|cc| Ok(Box::new(app::NetProfiler::new(cc))))
    ) {
        log::error!("Failed to run the application: {}", e);
    }
}
