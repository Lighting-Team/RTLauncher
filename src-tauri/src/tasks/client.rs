use crate::{
    client_list::DlClientListLoader,
    download::{DownloadConfig, DownloadTask, HighSpeedDownloader},
    error::DownloadError,
    models::{FileChecker, McInstance, NetFile},
    source::dl_source_launcher_or_meta_get,
    task::{Task, TaskProgress, TaskProgressUpdate, TaskStatus, TaskType},
    utils::json_str,
};

use std::path::Path;
use tokio::sync::mpsc;

/// 下载客户端任务
pub struct DownloadClientTask {
    name: String,
    mc_version: String,
    _instance_name: String,
    minecraft_dir: String,
    config: DownloadConfig,
}

impl DownloadClientTask {
    pub fn new(
        mc_version: &str,
        instance_name: &str,
        minecraft_dir: &str,
        config: DownloadConfig,
    ) -> Self {
        Self {
            name: format!("下载 {} 客户端", mc_version),
            mc_version: mc_version.to_string(),
            _instance_name: instance_name.to_string(),
            minecraft_dir: minecraft_dir.to_string(),
            config,
        }
    }

    /// 下载版本 JSON
    async fn download_version_json(&self) -> Option<serde_json::Value> {
        let loader = DlClientListLoader::new();

        match loader.execute(0).await {
            Ok(result) => {
                if let Some(versions) = result.value.get("versions").and_then(|v| v.as_array()) {
                    for version in versions {
                        if let Some(id) = version.get("id").and_then(|v| v.as_str()) {
                            if id == self.mc_version {
                                if let Some(url) = version.get("url").and_then(|v| v.as_str()) {
                                    return self.fetch_and_save_version_json(url).await;
                                }
                            }
                        }
                    }
                }
                None
            }
            Err(_) => None,
        }
    }

    /// 获取并保存版本 JSON
    async fn fetch_and_save_version_json(&self, url: &str) -> Option<serde_json::Value> {
        let client = reqwest::Client::new();
        if let Ok(response) = client.get(url).send().await {
            if let Ok(json) = response.json::<serde_json::Value>().await {
                let version_json_path = format!(
                    "{}/versions/{}/{}.json",
                    self.minecraft_dir, self.mc_version, self.mc_version
                );
                if let Some(parent) = Path::new(&version_json_path).parent() {
                    let _ = tokio::fs::create_dir_all(parent).await;
                }
                if let Ok(json_str) = serde_json::to_string_pretty(&json) {
                    let _ = tokio::fs::write(&version_json_path, json_str).await;
                }
                return Some(json);
            }
        }
        None
    }

    /// 分离官方源和镜像源
    fn separate_urls(urls: Vec<String>) -> (Vec<String>, Vec<String>) {
        let mut official = Vec::new();
        let mut mirror = Vec::new();

        for url in urls {
            if url.contains("bmclapi") || url.contains("mcbbs") {
                mirror.push(url);
            } else {
                official.push(url);
            }
        }

        (official, mirror)
    }

    /// 收集资源文件任务
    fn collect_asset_tasks(&self, index_path: &str) -> Vec<DownloadTask> {
        let mut tasks = Vec::new();

        let index_content = match std::fs::read_to_string(index_path) {
            Ok(content) => content,
            Err(_) => return tasks,
        };

        let index: serde_json::Value = match serde_json::from_str(&index_content) {
            Ok(val) => val,
            Err(_) => return tasks,
        };

        if let Some(objects) = index.get("objects").and_then(|v| v.as_object()) {
            for (_path, info) in objects {
                if let Some(hash) = info.get("hash").and_then(|v| v.as_str()) {
                    let hash_prefix = &hash[..2];
                    let official_url = format!(
                        "https://resources.download.minecraft.net/{}/{}",
                        hash_prefix, hash
                    );
                    let mirror_url = format!(
                        "https://bmclapi2.bangbang93.com/assets/{}/{}",
                        hash_prefix, hash
                    );
                    let local_path = format!(
                        "{}/assets/objects/{}/{}",
                        self.minecraft_dir, hash_prefix, hash
                    );

                    tasks.push(DownloadTask::new(
                        vec![official_url],
                        vec![mirror_url],
                        local_path,
                    ));
                }
            }
        }

        tasks
    }

    /// 创建下载任务
    fn create_download_task(&self, file: &NetFile) -> DownloadTask {
        let (official, mirror) = Self::separate_urls(file.urls.clone());
        DownloadTask::new(
            official,
            mirror,
            format!("{}/{}", self.minecraft_dir, file.local_path),
        )
    }

    /// 下载资源索引
    async fn download_asset_index(
        &self,
        instance: &McInstance,
        downloader: &HighSpeedDownloader,
    ) -> std::result::Result<String, String> {
        match dl_client_asset_index_get(instance) {
            Ok(Some(index_file)) => {
                let task = self.create_download_task(&index_file);
                let index_full_path = format!("{}/{}", self.minecraft_dir, index_file.local_path);

                if let Err(e) = downloader.download_file(&task).await {
                    return Err(format!("下载资源索引失败: {:?}", e));
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                Ok(index_full_path)
            }
            _ => Err("获取资源索引信息失败".to_string()),
        }
    }
}

#[async_trait::async_trait]
impl Task for DownloadClientTask {
    fn task_type(&self) -> TaskType {
        TaskType::DownloadClient
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn execute(
        &self,
        task_id: &str,
        progress_tx: mpsc::Sender<TaskProgressUpdate>,
    ) -> Result<(), String> {
        let downloader = HighSpeedDownloader::new(self.config.clone());
        let task_id = task_id.to_string();

        // 下载版本 JSON
        let version_json = match self.download_version_json().await {
            Some(json) => json,
            None => return Err("获取版本信息失败".to_string()),
        };

        // 创建实例
        let instance = McInstance {
            name: self.mc_version.clone(),
            inherit_name: None,
            json_object: version_json.clone(),
            path_version: format!("versions/{}/", self.mc_version),
        };

        // 下载资源索引
        let index_path = match self.download_asset_index(&instance, &downloader).await {
            Ok(path) => path,
            Err(e) => return Err(e),
        };

        // 收集所有下载任务
        let mut all_tasks: Vec<DownloadTask> = Vec::new();

        // 1. 客户端 Jar
        if let Ok(Some(jar_file)) = dl_client_jar_get(&instance, false) {
            all_tasks.push(self.create_download_task(&jar_file));
        }

        // 2. 资源文件
        let asset_tasks = self.collect_asset_tasks(&index_path);
        all_tasks.extend(asset_tasks);

        // 3. 支持库文件
        if let Ok(libraries) = mc_lib_net_files_from_instance(&instance) {
            for lib in libraries {
                all_tasks.push(self.create_download_task(&lib));
            }
        }

        let total_tasks = all_tasks.len();

        // 发送初始进度
        let _ = progress_tx
            .send(TaskProgressUpdate {
                task_id: task_id.clone(),
                progress: TaskProgress {
                    total: total_tasks as u64,
                    completed: 0,
                    current_speed: 0.0,
                    total_bytes: 0,
                    downloaded_bytes: 0,
                },
                status: TaskStatus::Running,
            })
            .await;

        // 使用优化的批量下载，带实时进度回调
        let progress_tx_clone = progress_tx.clone();
        let task_id_clone = task_id.clone();

        let results = downloader
            .download_batch(all_tasks, move |completed, total| {
                let _ = progress_tx_clone.try_send(TaskProgressUpdate {
                    task_id: task_id_clone.clone(),
                    progress: TaskProgress {
                        total: total as u64,
                        completed: completed as u64,
                        current_speed: 0.0,
                        total_bytes: 0,
                        downloaded_bytes: 0,
                    },
                    status: TaskStatus::Running,
                });
            })
            .await;

        // 检查结果
        let success_count = results.iter().filter(|r| r.is_ok()).count();

        if success_count == total_tasks {
            // 发送完成状态
            let _ = progress_tx
                .send(TaskProgressUpdate {
                    task_id: task_id.clone(),
                    progress: TaskProgress {
                        total: total_tasks as u64,
                        completed: total_tasks as u64,
                        current_speed: 0.0,
                        total_bytes: 0,
                        downloaded_bytes: 0,
                    },
                    status: TaskStatus::Completed,
                })
                .await;
            Ok(())
        } else {
            let error_msg = format!(
                "{}/{} 文件下载失败",
                total_tasks - success_count,
                total_tasks
            );
            let _ = progress_tx
                .send(TaskProgressUpdate {
                    task_id: task_id.clone(),
                    progress: TaskProgress {
                        total: total_tasks as u64,
                        completed: success_count as u64,
                        current_speed: 0.0,
                        total_bytes: 0,
                        downloaded_bytes: 0,
                    },
                    status: TaskStatus::Failed(error_msg.clone()),
                })
                .await;
            Err(error_msg)
        }
    }
}

// ============================================================================
// 客户端下载相关函数（从原 client.rs 合并）
// ============================================================================

/// 获取 Minecraft 客户端 Jar 文件的下载信息
pub fn dl_client_jar_get(
    instance: &McInstance,
    return_nothing_on_file_useable: bool,
) -> std::result::Result<Option<NetFile>, DownloadError> {
    let current_instance = instance.clone();

    let downloads = current_instance
        .json_object
        .get("downloads")
        .ok_or_else(|| {
            DownloadError::DownloadInfoNotFound(format!(
                "底层版本 {} 中无 jar 文件下载信息",
                current_instance.name
            ))
        })?;

    let client = downloads.get("client").ok_or_else(|| {
        DownloadError::DownloadInfoNotFound(format!(
            "底层版本 {} 中无 jar 文件下载信息",
            current_instance.name
        ))
    })?;

    let jar_url = json_str(client, "url").ok_or_else(|| {
        DownloadError::DownloadInfoNotFound(format!(
            "底层版本 {} 中无 jar 文件下载信息",
            current_instance.name
        ))
    })?;

    let size = client.get("size").and_then(|v| v.as_i64()).unwrap_or(-1);
    let sha1 = json_str(client, "sha1");

    let checker = FileChecker::new()
        .with_min_size(1024)
        .with_actual_size(size)
        .with_hash(sha1.unwrap_or_default());

    let jar_path = format!("{}{}.jar", current_instance.path_version, current_instance.name);

    if return_nothing_on_file_useable && checker.check(&jar_path).is_none() {
        return Ok(None);
    }

    let urls = dl_source_launcher_or_meta_get(&jar_url);

    Ok(Some(NetFile {
        urls,
        local_path: jar_path,
        checker,
    }))
}

/// 获取 Minecraft 客户端 AssetIndex 文件的下载信息
pub fn dl_client_asset_index_get(instance: &McInstance) -> std::result::Result<Option<NetFile>, DownloadError> {
    let current_instance = instance.clone();
    let index_info = mc_assets_get_index(&current_instance)?;

    let index_id = json_str(&index_info, "id").unwrap_or_else(|| "legacy".to_string());
    let index_address = format!("assets/indexes/{}.json", index_id);

    log::debug!(
        "[Download] 版本 {} 对应的资源文件索引为 {}",
        current_instance.name,
        index_id
    );

    let index_url = json_str(&index_info, "url");

    match index_url {
        Some(url) if !url.is_empty() => {
            let urls = dl_source_launcher_or_meta_get(&url);
            Ok(Some(NetFile {
                urls,
                local_path: index_address,
                checker: FileChecker::new()
                    .with_can_use_exists(false)
                    .with_is_json(true),
            }))
        }
        _ => Ok(None),
    }
}

/// 获取资源索引信息
fn mc_assets_get_index(instance: &McInstance) -> std::result::Result<serde_json::Value, DownloadError> {
    instance
        .json_object
        .get("assetIndex")
        .cloned()
        .or_else(|| {
            instance.json_object.get("assets").cloned().map(|assets| {
                serde_json::json!({
                    "id": assets.as_str().unwrap_or("legacy"),
                    "url": "",
                })
            })
        })
        .ok_or_else(|| {
            DownloadError::DownloadInfoNotFound(format!(
                "版本 {} 中无资源索引信息",
                instance.name
            ))
        })
}

/// 从实例获取支持库网络文件列表
pub fn mc_lib_net_files_from_instance(instance: &McInstance) -> std::result::Result<Vec<NetFile>, DownloadError> {
    let mut files = Vec::new();

    if let Some(libraries) = instance.json_object.get("libraries").and_then(|v| v.as_array()) {
        for lib in libraries {
            if let Some(downloads) = lib.get("downloads") {
                if let Some(artifact) = downloads.get("artifact") {
                    if let Some(url) = json_str(artifact, "url") {
                        if let Some(path) = json_str(artifact, "path") {
                            let sha1 = json_str(artifact, "sha1");
                            let size = artifact.get("size").and_then(|v| v.as_i64());

                            let checker = FileChecker::new()
                                .with_hash(sha1.unwrap_or_default())
                                .with_actual_size(size.unwrap_or(-1));

                            files.push(NetFile {
                                urls: vec![url],
                                local_path: format!("libraries/{}", path),
                                checker,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(files)
}
