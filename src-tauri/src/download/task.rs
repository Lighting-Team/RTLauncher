use super::config::DownloadStrategy;

/// 下载任务 - 区分官方源和镜像源
#[derive(Debug, Clone)]
pub struct DownloadTask {
    /// 官方源URL列表
    pub official_urls: Vec<String>,
    /// 镜像源URL列表
    pub mirror_urls: Vec<String>,
    /// 本地保存路径
    pub local_path: String,
    /// 文件大小（如果已知）
    pub file_size: Option<u64>,
    /// SHA1校验值
    pub sha1: Option<String>,
}

impl DownloadTask {
    pub fn new(official_urls: Vec<String>, mirror_urls: Vec<String>, local_path: String) -> Self {
        Self {
            official_urls,
            mirror_urls,
            local_path,
            file_size: None,
            sha1: None,
        }
    }

    pub fn with_file_size(mut self, size: u64) -> Self {
        self.file_size = Some(size);
        self
    }

    pub fn with_sha1(mut self, sha1: String) -> Self {
        self.sha1 = Some(sha1);
        self
    }

    /// 根据策略获取要使用的URL列表
    pub fn get_urls_by_strategy(&self, strategy: DownloadStrategy) -> Vec<String> {
        match strategy {
            DownloadStrategy::Hybrid => {
                // 混合模式：优先官方源，将镜像源放在后面
                let mut urls = self.official_urls.clone();
                urls.extend(self.mirror_urls.clone());
                urls
            }
            DownloadStrategy::OfficialOnly => self.official_urls.clone(),
            DownloadStrategy::MirrorOnly => self.mirror_urls.clone(),
        }
    }
}
