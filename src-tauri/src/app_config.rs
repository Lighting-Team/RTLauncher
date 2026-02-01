use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use regex::Regex;

#[derive(Serialize, Deserialize, Clone)]
pub struct LauncherConfig {
    pub initialized: bool,
    pub version: String,
    pub generated_at: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub launcher: LauncherConfig,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigStatus {
    Ok,
    Missing,
    InvalidJson, // Using generic name for format error, though now it's YAML
    InvalidData,
    ReadError,
    ParseErrorInitialized, // New status: Format error but initialized: true detected
}

#[derive(Serialize)]
pub struct ConfigCheckResult {
    pub status: ConfigStatus,
    pub config: Option<AppConfig>,
    pub error: Option<String>,
}

fn is_valid_config(config: &AppConfig) -> bool {
    !config.launcher.version.trim().is_empty() && config.launcher.generated_at > 0
}

fn get_config_status() -> ConfigCheckResult {
    let dir_path = Path::new("RTL");
    let config_path = Path::new("RTL/config.yml");
    if !dir_path.exists() || !config_path.exists() {
        return ConfigCheckResult {
            status: ConfigStatus::Missing,
            config: None,
            error: None,
        };
    }

    let content = match fs::read_to_string(config_path) {
        Ok(content) => content,
        Err(err) => {
            return ConfigCheckResult {
                status: ConfigStatus::ReadError,
                config: None,
                error: Some(err.to_string()),
            }
        }
    };

    if content.trim().is_empty() {
        return ConfigCheckResult {
            status: ConfigStatus::InvalidJson,
            config: None,
            error: Some("empty config file".to_string()),
        };
    }

    match serde_yaml::from_str::<AppConfig>(&content) {
        Ok(config) => {
            if is_valid_config(&config) {
                ConfigCheckResult {
                    status: ConfigStatus::Ok,
                    config: Some(config),
                    error: None,
                }
            } else {
                ConfigCheckResult {
                    status: ConfigStatus::InvalidData,
                    config: Some(config),
                    error: None,
                }
            }
        }
        Err(err) => {
            // Check if initialized: true exists despite parse error
            let re = Regex::new(r"initialized\s*:\s*true").unwrap();
            if re.is_match(&content) {
                ConfigCheckResult {
                    status: ConfigStatus::ParseErrorInitialized,
                    config: None,
                    error: Some(err.to_string()),
                }
            } else {
                ConfigCheckResult {
                    status: ConfigStatus::InvalidJson,
                    config: None,
                    error: Some(err.to_string()),
                }
            }
        },
    }
}

#[tauri::command]
pub fn check_initialization() -> bool {
    let config_path = Path::new("RTL/config.yml");
    
    if !config_path.exists() {
        return false;
    }

    match fs::read_to_string(config_path) {
        Ok(content) => match serde_yaml::from_str::<AppConfig>(&content) {
            Ok(config) => config.launcher.initialized,
            Err(_) => false,
        },
        Err(_) => false,
    }
}

#[tauri::command]
pub fn check_config_files() -> bool {
    matches!(get_config_status().status, ConfigStatus::Ok)
}

#[tauri::command]
pub fn check_config_status() -> ConfigCheckResult {
    get_config_status()
}

#[tauri::command]
pub fn create_config_files() -> Result<(), String> {
    let dir_path = Path::new("RTL");
    let config_path = Path::new("RTL/config.yml");

    if !dir_path.exists() {
        fs::create_dir_all(dir_path).map_err(|e| e.to_string())?;
    }

    let should_write = match fs::read_to_string(config_path) {
        Ok(content) => match serde_yaml::from_str::<AppConfig>(&content) {
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
            serde_yaml::to_string(&default_config).map_err(|e| e.to_string())?;
        fs::write(config_path, config_str).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
pub fn complete_initialization() -> Result<(), String> {
    let config_path = Path::new("RTL/config.yml");
    
    let mut config = match fs::read_to_string(config_path) {
        Ok(content) => serde_yaml::from_str::<AppConfig>(&content).map_err(|e| e.to_string())?,
        Err(e) => return Err(e.to_string()),
    };

    config.launcher.initialized = true;

    let config_str = serde_yaml::to_string(&config).map_err(|e| e.to_string())?;
    fs::write(config_path, config_str).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn repair_config() -> Result<(), String> {
    let config_path = Path::new("RTL/config.yml");
    
    // Create a valid default config but with initialized = true
    let generated_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;
        
    let default_config = AppConfig {
        launcher: LauncherConfig {
            initialized: true, // Force true for repair
            version: env!("CARGO_PKG_VERSION").to_string(),
            generated_at,
        },
    };
    
    let config_str = serde_yaml::to_string(&default_config).map_err(|e| e.to_string())?;
    fs::write(config_path, config_str).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub fn get_config_path() -> Result<String, String> {
    let dir_path = std::env::current_dir().map_err(|e| e.to_string())?.join("RTL");
    Ok(dir_path.to_string_lossy().into_owned())
}
