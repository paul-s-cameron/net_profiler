use std::path::PathBuf;
use net_profiler::NetworkProfile;

pub fn import_profiles_from_file(file_path: PathBuf) -> Result<Vec<NetworkProfile>, String> {
    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Error reading file {}: {}", file_path.display(), e))?;

    serde_json::from_str::<Vec<NetworkProfile>>(&content)
        .map_err(|e| format!("Error parsing profiles JSON: {}", e))
}

pub fn export_profiles_to_file(profiles: &[NetworkProfile], file_path: PathBuf) -> Result<(), String> {
    let file_path = file_path.with_extension("nprf");
    
    let profiles_json = serde_json::to_string(profiles)
        .map_err(|e| format!("Error serializing profiles: {}", e))?;

    std::fs::write(&file_path, profiles_json)
        .map_err(|e| format!("Error saving profiles to file: {}", e))?;

    Ok(())
}
