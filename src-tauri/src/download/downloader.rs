use super::{DownloadConfig, DownloadStrategy, DownloadTask};
use crate::error::{DownloadError, Result};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::Duration;

/// 下载进度报告
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub completed: usize,
    pub total: usize,
    pub current_file: String,
}

/// 高速下载器
pub struct HighSpeedDownloader {
    config: DownloadConfig,
    client: reqwest::Client,
}

impl HighSpeedDownloader {
    pub fn new(config: DownloadConfig) -> Self {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(config.connect_timeout))
            .timeout(Duration::from_secs(config.read_timeout))
            .pool_max_idle_per_host(100)
            .pool_idle_timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        Self { config, client }
    }

    /// 下载单个文件
    pub async fn download_file(&self, task: &DownloadTask) -> Result<()> {
        // 检查文件是否已存在且有效
        if Path::new(&task.local_path).exists() {
            if let Some(expected_sha1) = &task.sha1 {
                if let Ok(actual_sha1) = self.calculate_sha1(&task.local_path).await {
                    if &actual_sha1 == expected_sha1 {
                        return Ok(());
                    }
                }
            }
        }

        // 创建目录
        if let Some(parent) = Path::new(&task.local_path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // 根据策略获取URL列表
        let urls = task.get_urls_by_strategy(self.config.strategy);

        if urls.is_empty() {
            return Err(DownloadError::LoaderExecution("没有可用的下载源".to_string()));
        }

        // 判断是否需要分块下载
        let file_size = task.file_size.unwrap_or(0);
        if file_size > self.config.large_file_threshold {
            self.download_large_file(task, &urls).await
        } else {
            self.download_small_file(task, &urls).await
        }
    }

    /// 下载小文件
    async fn download_small_file(&self, task: &DownloadTask, urls: &[String]) -> Result<()> {
        let max_retries = self.config.max_retries;
        let strategy = self.config.strategy;

        for (url_index, url) in urls.iter().enumerate() {
            let retries = match strategy {
                DownloadStrategy::Hybrid => {
                    if url_index < task.official_urls.len() {
                        2
                    } else {
                        max_retries
                    }
                }
                _ => max_retries,
            };

            for attempt in 0..retries {
                match self.try_download_single(url, &task.local_path).await {
                    Ok(()) => {
                        if let Some(expected_sha1) = &task.sha1 {
                            if let Ok(actual_sha1) = self.calculate_sha1(&task.local_path).await {
                                if &actual_sha1 == expected_sha1 {
                                    return Ok(());
                                }
                            }
                        } else {
                            return Ok(());
                        }
                    }
                    Err(_e) => {
                        if attempt < retries - 1 {
                            tokio::time::sleep(Duration::from_millis(500)).await;
                        }
                    }
                }
            }
        }

        Err(DownloadError::LoaderExecution(format!(
            "所有URL都下载失败: {:?}",
            urls
        )))
    }

    /// 尝试单次下载
    async fn try_download_single(&self, url: &str, local_path: &str) -> Result<()> {
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(DownloadError::LoaderExecution(format!(
                "HTTP {}: {}",
                response.status(),
                url
            )));
        }

        let bytes = response.bytes().await?;
        let mut file = File::create(local_path)?;
        file.write_all(&bytes)?;

        Ok(())
    }

    /// 下载大文件（分块多线程）
    async fn download_large_file(&self, task: &DownloadTask, urls: &[String]) -> Result<()> {
        let file_size = task.file_size.unwrap_or(0);
        let chunk_count = self.config.large_file_chunks;
        let chunk_size = file_size / chunk_count as u64;
        let local_path = &task.local_path;
        let strategy = self.config.strategy;

        let temp_dir = format!("{}.parts", local_path);
        fs::create_dir_all(&temp_dir)?;

        let mut handles = vec![];
        let urls = Arc::new(urls.to_vec());

        for i in 0..chunk_count {
            let start = i as u64 * chunk_size;
            let end = if i == chunk_count - 1 {
                file_size - 1
            } else {
                (i as u64 + 1) * chunk_size - 1
            };

            let urls = Arc::clone(&urls);
            let temp_file = format!("{}/part_{}", temp_dir, i);
            let client = self.client.clone();
            let max_retries = self.config.max_retries;

            let handle = tokio::spawn(async move {
                for (url_index, url) in urls.iter().enumerate() {
                    let retries = match strategy {
                        DownloadStrategy::Hybrid => {
                            if url_index == 0 {
                                2
                            } else {
                                max_retries
                            }
                        }
                        _ => max_retries,
                    };

                    for _attempt in 0..retries {
                        match Self::download_chunk(&client, url, &temp_file, start, end).await {
                            Ok(()) => return Ok(()),
                            Err(_e) => {}
                        }
                    }
                }
                Err::<(), DownloadError>(DownloadError::LoaderExecution(format!(
                    "分块 {} 所有URL都失败",
                    i
                )))
            });

            handles.push(handle);
        }

        for handle in handles {
            handle
                .await
                .map_err(|e| DownloadError::LoaderExecution(format!("分块任务失败: {:?}", e)))??;
        }

        self.merge_chunks(&temp_dir, chunk_count, local_path).await?;
        fs::remove_dir_all(&temp_dir)?;

        if let Some(expected_sha1) = &task.sha1 {
            if let Ok(actual_sha1) = self.calculate_sha1(local_path).await {
                if &actual_sha1 != expected_sha1 {
                    return Err(DownloadError::LoaderExecution("文件校验失败".to_string()));
                }
            }
        }

        Ok(())
    }

    /// 下载单个分块
    async fn download_chunk(
        client: &reqwest::Client,
        url: &str,
        temp_file: &str,
        start: u64,
        end: u64,
    ) -> Result<()> {
        let range_header = format!("bytes={}-{}", start, end);

        let response = client
            .get(url)
            .header("Range", range_header)
            .send()
            .await?;

        if !response.status().is_success()
            && response.status() != reqwest::StatusCode::PARTIAL_CONTENT
        {
            return Err(DownloadError::LoaderExecution(format!(
                "HTTP {}: {}",
                response.status(),
                url
            )));
        }

        let bytes = response.bytes().await?;
        let mut file = File::create(temp_file)?;
        file.write_all(&bytes)?;

        Ok(())
    }

    /// 合并分块文件
    async fn merge_chunks(&self, temp_dir: &str, chunk_count: usize, output_path: &str) -> Result<()> {
        let mut output = File::create(output_path)?;

        for i in 0..chunk_count {
            let chunk_path = format!("{}/part_{}", temp_dir, i);
            let mut chunk_file = File::open(&chunk_path)?;
            let mut buffer = Vec::new();
            chunk_file.read_to_end(&mut buffer)?;
            output.write_all(&buffer)?;
        }

        Ok(())
    }

    /// 批量下载文件 - 实时进度版本
    pub async fn download_batch<F>(&self, tasks: Vec<DownloadTask>, on_progress: F) -> Vec<Result<()>>
    where
        F: Fn(usize, usize) + Send + 'static,
    {
        use futures::stream::{self, StreamExt};
        use std::sync::atomic::{AtomicUsize, Ordering};

        let total = tasks.len();
        let semaphore = Arc::new(Semaphore::new(self.config.thread_pool_size));
        let completed = Arc::new(AtomicUsize::new(0));

        let results: Vec<Result<()>> = stream::iter(tasks.into_iter())
            .map(|task| {
                let downloader = self.clone();
                let semaphore = Arc::clone(&semaphore);
                let completed = Arc::clone(&completed);
                let on_progress = &on_progress;

                async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    let result = downloader.download_file(&task).await;
                    
                    if result.is_ok() {
                        let count = completed.fetch_add(1, Ordering::SeqCst) + 1;
                        on_progress(count, total);
                    }
                    
                    result
                }
            })
            .buffer_unordered(self.config.thread_pool_size)
            .collect()
            .await;

        results
    }

    /// 计算文件SHA1
    async fn calculate_sha1(&self, file_path: &str) -> Result<String> {
        use sha1::{Digest, Sha1};

        let mut file = File::open(file_path)?;
        let mut hasher = Sha1::new();
        let mut buffer = [0u8; 8192];

        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }
}

impl Clone for HighSpeedDownloader {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            client: self.client.clone(),
        }
    }
}
