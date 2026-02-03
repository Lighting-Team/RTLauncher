pub mod download {
    use anyhow::{ Context, Result, anyhow };
    use reqwest::header::{ HeaderMap, HeaderValue, CONTENT_LENGTH, RANGE, ACCEPT_RANGES };
    use std::fs::{ File, OpenOptions };
    use std::io::{ Write, Seek, SeekFrom };
    use std::path::{ Path, PathBuf };
    use std::sync::Arc;
    use std::time::{ Duration, Instant };
    use tokio::sync::Mutex;
    use indicatif::{ ProgressBar, ProgressStyle, MultiProgress, HumanBytes };
    use futures::stream::{ StreamExt, TryStreamExt };
    use futures::future::join_all;

    #[derive(Debug, Clone)]
    pub struct DownloadConfig {
        pub url: String,
        pub output_path: PathBuf,
        pub max_retries: u32,
        pub retry_delay: Duration,
        pub timeout: Option<Duration>,
        pub user_agent: Option<String>,
        pub max_connections: usize,
        pub min_chunk_size: u64,
        pub show_progress: bool,
    }

    impl Default for DownloadConfig {
        fn default() -> Self {
            Self {
                url: String::new(),
                output_path: PathBuf::new(),
                max_retries: 3,
                retry_delay: Duration::from_secs(2),
                timeout: Some(Duration::from_secs(30)),
                user_agent: Some(
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string()
                ),
                max_connections: 4,
                min_chunk_size: 1024 * 1024,
                show_progress: true,
            }
        }
    }

    /// 下载状态
    #[derive(Debug, Clone)]
    pub struct DownloadStatus {
        pub total_size: Option<u64>,
        pub downloaded: u64,
        pub speed: f64,
        pub is_complete: bool,
        pub elapsed_time: Duration,
    }

    /// 下载器
    pub struct Downloader {
        config: DownloadConfig,
        client: reqwest::Client,
        progress_bar: Option<ProgressBar>,
        multi_progress: Option<MultiProgress>,
    }

    impl Downloader {
        pub fn new(config: DownloadConfig) -> Result<Self> {
            let mut headers = HeaderMap::new();

            if let Some(ua) = &config.user_agent {
                headers.insert("User-Agent", HeaderValue::from_str(ua)?);
            }

            let mut client_builder = reqwest::Client::builder().default_headers(headers);

            if let Some(timeout) = config.timeout {
                client_builder = client_builder.timeout(timeout);
            }

            let client = client_builder.build()?;

            Ok(Self {
                config,
                client,
                progress_bar: None,
                multi_progress: None,
            })
        }

        pub async fn download(&mut self) -> Result<DownloadStatus> {
            let start_time = Instant::now();

            let file_info = self.get_file_info().await?;

            if self.config.show_progress {
                self.setup_progress_bar(file_info.total_size);
            }

            let downloaded = if file_info.supports_partial && self.config.max_connections > 1 {
                self.download_with_multiple_connections(&file_info).await?
            } else {
                self.download_single_connection(&file_info).await?
            };

            let elapsed_time = start_time.elapsed();
            let speed = (downloaded as f64) / elapsed_time.as_secs_f64();

            if let Some(pb) = &self.progress_bar {
                pb.finish_with_message("Complete");
            }

            Ok(DownloadStatus {
                total_size: Some(file_info.total_size),
                downloaded,
                speed,
                is_complete: true,
                elapsed_time,
            })
        }

        pub async fn download_file(url: &str, output_path: &Path) -> Result<DownloadStatus> {
            let config = DownloadConfig {
                url: url.to_string(),
                output_path: output_path.to_path_buf(),
                ..Default::default()
            };

            let mut downloader = Downloader::new(config)?;
            downloader.download().await
        }

        pub async fn download_files(urls: &[(&str, &Path)]) -> Result<Vec<Result<DownloadStatus>>> {
            let mut tasks = Vec::new();

            for (url, path) in urls {
                let url = (*url).to_string();
                let path = (*path).to_path_buf();

                tasks.push(
                    tokio::spawn(async move { Downloader::download_file(&url, &path).await })
                );
            }

            let results = join_all(tasks).await;
            let mut statuses = Vec::new();

            for result in results {
                statuses.push(result.map_err(|e| anyhow!("Failed: {}", e))?);
            }

            Ok(statuses)
        }

        async fn get_file_info(&self) -> Result<FileInfo> {
            let response = self.client.head(&self.config.url).send().await?;

            if !response.status().is_success() {
                return Err(anyhow!("Failed: {}", response.status()));
            }

            let total_size = response
                .headers()
                .get(CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            let supports_partial = response
                .headers()
                .get(ACCEPT_RANGES)
                .and_then(|v| v.to_str().ok())
                .map(|s| s == "bytes")
                .unwrap_or(false);

            Ok(FileInfo {
                total_size,
                supports_partial,
            })
        }

        async fn download_single_connection(&self, file_info: &FileInfo) -> Result<u64> {
            let mut response = self.client.get(&self.config.url).send().await?;

            if !response.status().is_success() {
                return Err(anyhow!("Failed: {}", response.status()));
            }

            let mut file = self.create_file(0, file_info.total_size)?;
            let mut downloaded = 0;
            let mut last_update = Instant::now();

            while let Some(chunk) = response.chunk().await? {
                file.write_all(&chunk)?;
                downloaded += chunk.len() as u64;

                if self.config.show_progress && last_update.elapsed() > Duration::from_millis(100) {
                    if let Some(pb) = &self.progress_bar {
                        pb.set_position(downloaded);
                    }
                    last_update = Instant::now();
                }
            }

            Ok(downloaded)
        }

        async fn download_with_multiple_connections(&self, file_info: &FileInfo) -> Result<u64> {
            let total_size = file_info.total_size;
            let chunk_size = self.calculate_chunk_size(total_size);
            let num_chunks = (total_size + chunk_size - 1) / chunk_size;

            println!("{} connections", num_chunks.min(self.config.max_connections));

            let mut tasks = Vec::new();
            let progress = Arc::new(Mutex::new(0u64));

            for i in 0..num_chunks.min(self.config.max_connections) {
                let start = i * chunk_size;
                let end = if i == num_chunks - 1 {
                    total_size - 1
                } else {
                    (i + 1) * chunk_size - 1
                };

                let config = self.config.clone();
                let client = self.client.clone();
                let progress = Arc::clone(&progress);
                let pb = self.progress_bar.clone();

                tasks.push(
                    tokio::spawn(async move {
                        Self::download_chunk(&client, &config, start, end, i, progress, pb).await
                    })
                );
            }

            let results = join_all(tasks).await;
            let mut total_downloaded = 0;

            for result in results {
                total_downloaded += result.map_err(|e| anyhow!("Failed: {}", e))??;
            }

            self.merge_chunks(num_chunks.min(self.config.max_connections))?;

            Ok(total_downloaded)
        }

        async fn download_chunk(
            client: &reqwest::Client,
            config: &DownloadConfig,
            start: u64,
            end: u64,
            chunk_id: u64,
            progress: Arc<Mutex<u64>>,
            pb: Option<ProgressBar>
        ) -> Result<u64> {
            let mut retries = 0;

            while retries <= config.max_retries {
                match
                    Self::download_chunk_internal(
                        client,
                        config,
                        start,
                        end,
                        chunk_id,
                        &progress,
                        &pb
                    ).await
                {
                    Ok(size) => {
                        return Ok(size);
                    }
                    Err(e) if retries < config.max_retries => {
                        eprintln!(
                            "Chunk {} Failed (Retry {}/{}): {}",
                            chunk_id,
                            retries + 1,
                            config.max_retries,
                            e
                        );
                        tokio::time::sleep(config.retry_delay).await;
                        retries += 1;
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }

            Err(anyhow!("Failed to download chunk {}, reached max retries", chunk_id))
        }

        async fn download_chunk_internal(
            client: &reqwest::Client,
            config: &DownloadConfig,
            start: u64,
            end: u64,
            chunk_id: u64,
            progress: &Arc<Mutex<u64>>,
            pb: &Option<ProgressBar>
        ) -> Result<u64> {
            let range_header = format!("bytes={}-{}", start, end);

            let mut response = client.get(&config.url).header(RANGE, range_header).send().await?;

            if !response.status().is_success() {
                return Err(anyhow!("Failed to request chunk: {}", response.status()));
            }

            let chunk_path = config.output_path.with_extension(format!("part{}", chunk_id));
            let mut file = File::create(&chunk_path)?;
            let mut downloaded = 0;

            while let Some(chunk) = response.chunk().await? {
                file.write_all(&chunk)?;
                downloaded += chunk.len() as u64;

                let mut total_progress = progress.lock().await;
                *total_progress += chunk.len() as u64;

                if let Some(pb) = pb {
                    pb.set_position(*total_progress);
                }
            }

            Ok(downloaded)
        }

        fn merge_chunks(&self, num_chunks: usize) -> Result<()> {
            let mut output_file = File::create(&self.config.output_path)?;

            for i in 0..num_chunks {
                let chunk_path = self.config.output_path.with_extension(format!("part{}", i));
                let mut chunk_file = File::open(&chunk_path)?;
                std::io::copy(&mut chunk_file, &mut output_file)?;

                let _ = std::fs::remove_file(&chunk_path);
            }

            Ok(())
        }

        fn create_file(&self, start: u64, total_size: u64) -> Result<File> {
            let file = OpenOptions::new().create(true).write(true).open(&self.config.output_path)?;

            if start > 0 && total_size > 0 {
                file.set_len(total_size)?;
            }

            Ok(file)
        }

        fn calculate_chunk_size(&self, total_size: u64) -> u64 {
            let chunk_size = total_size / (self.config.max_connections as u64);
            chunk_size.max(self.config.min_chunk_size)
        }

        fn setup_progress_bar(&mut self, total_size: u64) {
            let pb = ProgressBar::new(total_size);

            pb.set_style(
                ProgressStyle::default_bar()
                    .template(
                        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})"
                    )
                    .unwrap()
                    .progress_chars("#>-")
            );

            self.progress_bar = Some(pb);
        }
    }

    #[derive(Debug)]
    struct FileInfo {
        total_size: u64,
        supports_partial: bool,
    }

    pub fn validate_url(url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }

    pub fn get_filename_from_url(url: &str) -> Option<String> {
        url::Url
            ::parse(url)
            .ok()
            .and_then(|u| {
                u.path_segments().and_then(|segments| segments.last().map(String::from))
            })
            .filter(|name| !name.is_empty())
    }

    pub fn format_speed(speed: f64) -> String {
        if speed >= 1024.0 * 1024.0 {
            format!("{:.2} MB/s", speed / (1024.0 * 1024.0))
        } else if speed >= 1024.0 {
            format!("{:.2} KB/s", speed / 1024.0)
        } else {
            format!("{:.0} B/s", speed)
        }
    }

    pub fn format_duration(duration: Duration) -> String {
        let seconds = duration.as_secs();
        if seconds < 60 {
            format!("{}s", seconds)
        } else if seconds < 3600 {
            format!("{}m{}s", seconds / 60, seconds % 60)
        } else {
            format!("{}h{}m{}s", seconds / 3600, (seconds % 3600) / 60, seconds % 60)
        }
    }
}
