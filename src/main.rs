#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;

///TODO: 
/// 1. Abstract network commands for Windows and Linux
/// *2. Remove adapter from profile and create popup to select adapter to apply the profile to
/// *3. Setup egui_toast for notifications
/// *4. Logging to file

fn main() {
    simple_logging::log_to_file("net_profiler.log", log::LevelFilter::Info).expect("Failed to initialize logging");
    
    // Log system information
    log::info!("Operating System: {}", std::env::consts::OS);
    log::info!("Architecture: {}", std::env::consts::ARCH);
    println!("Operating System: {}", std::env::consts::OS);
    println!("Architecture: {}", std::env::consts::ARCH);
    
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    log::info!("Creating eframe native options...");

    match eframe::run_native(
        "Net Profiler",
        native_options,
        Box::new(|cc| {
            log::info!("Creating NetProfiler instance...");
            let app = app::NetProfiler::new(cc);
            log::info!("NetProfiler created successfully");
            Ok(Box::new(app))
        })
    ) {
        Ok(_) => {
            log::info!("Application exited normally");
        }
        Err(e) => {
            log::error!("Failed to run the application: {}", e);
        }
    }
}
