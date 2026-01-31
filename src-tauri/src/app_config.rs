use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub initialize: i32,
}

#[tauri::command]
pub fn check_initialization() -> bool {
    let config_path = Path::new("RTL/config.json");
    
    if !config_path.exists() {
        return false;
    }

    match fs::read_to_string(config_path) {
        Ok(content) => {
            match serde_json::from_str::<AppConfig>(&content) {
                Ok(config) => config.initialize != 0,
                Err(_) => false,
            }
        },
        Err(_) => false,
    }
}

#[tauri::command]
pub fn check_config_files() -> bool {
    let dir_path = Path::new("RTL");
    let config_path = Path::new("RTL/config.json");
    dir_path.exists() && config_path.exists()
}

#[tauri::command]
pub fn create_config_files() -> Result<(), String> {
    let dir_path = Path::new("RTL");
    let config_path = Path::new("RTL/config.json");

    if !dir_path.exists() {
        fs::create_dir(dir_path).map_err(|e| e.to_string())?;
    }

    if !config_path.exists() {
        // 创建一个空的或默认的配置文件，内容后续补充
        let default_config = AppConfig { initialize: 0 };
        let config_str = serde_json::to_string_pretty(&default_config).map_err(|e| e.to_string())?;
        fs::write(config_path, config_str).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}
