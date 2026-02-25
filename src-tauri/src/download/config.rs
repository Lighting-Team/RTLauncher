/// 下载策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadStrategy {
    /// 混合模式：优先官方源，失败2次后切换镜像源
    Hybrid,
    /// 仅官方源
    OfficialOnly,
    /// 仅镜像源
    MirrorOnly,
}

impl Default for DownloadStrategy {
    fn default() -> Self {
        DownloadStrategy::Hybrid
    }
}

/// 下载配置
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    /// 线程池大小（默认64）
    pub thread_pool_size: usize,
    /// 大文件阈值（字节），超过此大小的文件将使用分块下载
    pub large_file_threshold: u64,
    /// 大文件分块数（4-8个线程）
    pub large_file_chunks: usize,
    /// 下载策略
    pub strategy: DownloadStrategy,
    /// 单文件最大重试次数
    pub max_retries: u32,
    /// 连接超时（秒）
    pub connect_timeout: u64,
    /// 读取超时（秒）
    pub read_timeout: u64,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            thread_pool_size: 64,
            large_file_threshold: 5 * 1024 * 1024, // 5MB
            large_file_chunks: 8,
            strategy: DownloadStrategy::Hybrid,
            max_retries: 3,
            connect_timeout: 30,
            read_timeout: 60,
        }
    }
}
