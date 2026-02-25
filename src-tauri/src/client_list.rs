use crate::error::{DownloadError, Result};
use crate::models::DlClientListResult;
use crate::utils::get_time_ms;
use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use tokio::time::timeout;

/// 所有正式版的 Minecraft Drop 序数缓存
static ALL_DROPS: Lazy<Mutex<Vec<i32>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// 客户端版本列表加载器
pub struct DlClientListLoader;

impl DlClientListLoader {
    pub fn new() -> Self {
        Self
    }

    /// 主加载器执行
    pub async fn execute(&self, version_list_source: i32) -> Result<DlClientListResult> {
        match version_list_source {
            0 => {
                // 优先 BMCLAPI
                if let Ok(result) = timeout(Duration::from_secs(30), Self::load_bmclapi()).await {
                    return result;
                }
                timeout(Duration::from_secs(90), Self::load_official()).await
                    .map_err(|_| DownloadError::Timeout)?
            }
            1 => {
                // 优先官方源
                if let Ok(result) = timeout(Duration::from_secs(5), Self::load_official()).await {
                    return result;
                }
                timeout(Duration::from_secs(35), Self::load_bmclapi()).await
                    .map_err(|_| DownloadError::Timeout)?
            }
            _ => {
                // 仅官方源
                if let Ok(result) = timeout(Duration::from_secs(60), Self::load_official()).await {
                    return result;
                }
                timeout(Duration::from_secs(120), Self::load_bmclapi()).await
                    .map_err(|_| DownloadError::Timeout)?
            }
        }
    }

    /// 从官方源加载
    async fn load_official() -> Result<DlClientListResult> {
        let start_time = get_time_ms();

        let client = reqwest::Client::new();
        let response = client
            .get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
            .timeout(Duration::from_secs(30))
            .send()
            .await?;

        let json: Value = response.json().await?;

        // 验证版本列表
        let versions = json
            .get("versions")
            .and_then(|v| v.as_array())
            .ok_or_else(|| DownloadError::VersionListParse("获取到的版本列表格式错误".to_string()))?;

        if versions.len() < 200 {
            return Err(DownloadError::VersionListParse(
                format!("获取到的版本列表长度不足: {}", versions.len())
            ));
        }

        // 确定官方源是否可用
        let delta_time = get_time_ms() - start_time;
        let prefer_official = delta_time < 4000;
        log::info!(
            "[Download] Mojang 官方源加载耗时: {}ms, {}",
            delta_time,
            if prefer_official { "可优先使用官方源" } else { "不优先使用官方源" }
        );

        Ok(DlClientListResult {
            is_official: true,
            value: json,
        })
    }

    /// 从 BMCLAPI 加载
    async fn load_bmclapi() -> Result<DlClientListResult> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://bmclapi2.bangbang93.com/mc/game/version_manifest.json")
            .timeout(Duration::from_secs(30))
            .send()
            .await?;
        
        let json: Value = response.json().await?;
        
        // 验证版本列表
        let versions = json
            .get("versions")
            .and_then(|v| v.as_array())
            .ok_or_else(|| DownloadError::VersionListParse("获取到的版本列表格式错误".to_string()))?;
        
        if versions.len() < 200 {
            return Err(DownloadError::VersionListParse(
                format!("获取到的版本列表长度不足: {}", versions.len())
            ));
        }
        
        Ok(DlClientListResult {
            is_official: false,
            value: json,
        })
    }
}

impl Default for DlClientListLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// 获取所有 Drop 序数
pub fn get_all_drops() -> Option<Vec<i32>> {
    ALL_DROPS.lock().ok().map(|drops| {
        if drops.is_empty() {
            None
        } else {
            Some(drops.clone())
        }
    }).flatten()
}

/// 获取某个版本的 Json 下载地址
pub async fn dl_client_list_get(id: &str, loader: &DlClientListLoader, version_list_source: i32) -> Result<Option<String>> {
    let mut search_id = id.replace('_', "-");
    
    // 处理版本格式
    if search_id != "1.0" && search_id.ends_with(".0") {
        search_id = search_id[..search_id.len() - 2].to_string();
    }
    
    // 获取版本列表
    let result = loader.execute(version_list_source).await?;
    
    // 查找版本
    if let Some(versions) = result.value.get("versions").and_then(|v| v.as_array()) {
        for version in versions {
            if let Some(version_id) = version.get("id").and_then(|v| v.as_str()) {
                if version_id == search_id {
                    return Ok(version.get("url").and_then(|v| v.as_str()).map(|s| s.to_string()));
                }
            }
        }
    }
    
    log::debug!("[Download] 未发现版本 {} 的 json 下载地址", id);
    Ok(None)
}

/// 版本列表缓存
static VERSION_CACHE: Lazy<Mutex<HashMap<String, Value>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// 获取版本列表（带缓存）
pub async fn get_version_list(force_refresh: bool, version_list_source: i32) -> Result<DlClientListResult> {
    let cache_key = format!("source_{}", version_list_source);
    
    if !force_refresh {
        if let Ok(cache) = VERSION_CACHE.lock() {
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(DlClientListResult {
                    is_official: false,
                    value: cached.clone(),
                });
            }
        }
    }
    
    let loader = DlClientListLoader::new();
    let result = loader.execute(version_list_source).await?;
    
    // 更新缓存
    if let Ok(mut cache) = VERSION_CACHE.lock() {
        cache.insert(cache_key, result.value.clone());
    }
    
    Ok(result)
}

/// 检查是否有新版本可用
pub fn check_for_updates(current_version: &str, version_list: &Value) -> Option<String> {
    if let Some(latest) = version_list.get("latest") {
        // 检查快照版
        if let Some(snapshot) = latest.get("snapshot").and_then(|v| v.as_str()) {
            if snapshot != current_version {
                return Some(snapshot.to_string());
            }
        }
        
        // 检查正式版
        if let Some(release) = latest.get("release").and_then(|v| v.as_str()) {
            if release != current_version {
                return Some(release.to_string());
            }
        }
    }
    
    None
}

