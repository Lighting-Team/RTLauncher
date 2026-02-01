use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize)]
pub struct LauncherConfig {
    pub initialized: bool,
    pub version: String,
    pub generated_at: i64,
}

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub launcher: LauncherConfig,
}

fn is_valid_config(config: &AppConfig) -> bool {
    !config.launcher.version.trim().is_empty() && config.launcher.generated_at > 0
}

#[tauri::command]
pub fn check_initialization() -> bool {
    let config_path = Path::new("RTL/config.json");
    
    if !config_path.exists() {
        return false;
    }

    match fs::read_to_string(config_path) {
        Ok(content) => match serde_json::from_str::<AppConfig>(&content) {
            Ok(config) => config.launcher.initialized,
            Err(_) => false,
        },
        Err(_) => false,
    }
}

#[tauri::command]
pub fn check_config_files() -> bool {
    let dir_path = Path::new("RTL");
    let config_path = Path::new("RTL/config.json");
    if !dir_path.exists() || !config_path.exists() {
        return false;
    }

    match fs::read_to_string(config_path) {
        Ok(content) => match serde_json::from_str::<AppConfig>(&content) {
            Ok(config) => is_valid_config(&config),
            Err(_) => false,
        },
        Err(_) => false,
    }
}

#[tauri::command]
pub fn create_config_files() -> Result<(), String> {
    let dir_path = Path::new("RTL");
    let config_path = Path::new("RTL/config.json");

    if !dir_path.exists() {
        fs::create_dir_all(dir_path).map_err(|e| e.to_string())?;
    }

    let should_write = match fs::read_to_string(config_path) {
        Ok(content) => match serde_json::from_str::<AppConfig>(&content) {
            Ok(config) => !is_valid_config(&config),
            Err(_) => true,
        },
        Err(_) => true,
    };

    if should_write {
        let generated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs() as i64;
        let default_config = AppConfig {
            launcher: LauncherConfig {
                initialized: false,
                version: env!("CARGO_PKG_VERSION").to_string(),
                generated_at,
            },
        };
        let config_str =
            serde_json::to_string_pretty(&default_config).map_err(|e| e.to_string())?;
        fs::write(config_path, config_str).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
pub fn get_config_path() -> Result<String, String> {
    let dir_path = std::env::current_dir().map_err(|e| e.to_string())?.join("RTL");
    Ok(dir_path.to_string_lossy().into_owned())
}
