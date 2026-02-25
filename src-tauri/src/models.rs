use serde_json::Value as JsonValue;

/// 下载源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadSource {
    /// 优先使用镜像源
    PreferMirror,
    /// 优先使用官方源
    PreferOfficial,
    /// 仅使用官方源
    OfficialOnly,
}

impl Default for DownloadSource {
    fn default() -> Self {
        DownloadSource::PreferOfficial
    }
}

/// 版本列表下载源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionListSource {
    /// 优先使用镜像源
    PreferMirror,
    /// 优先使用官方源
    PreferOfficial,
    /// 仅使用官方源
    OfficialOnly,
}

impl Default for VersionListSource {
    fn default() -> Self {
        VersionListSource::PreferOfficial
    }
}

/// 资源索引存在行为
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetsIndexExistsBehaviour {
    /// 如果文件存在，则不进行下载
    DontDownload,
    /// 如果文件存在，则启动新的下载加载器进行独立的更新
    DownloadInBackground,
    /// 如果文件存在，也同样进行下载
    AlwaysDownload,
}

impl Default for AssetsIndexExistsBehaviour {
    fn default() -> Self {
        AssetsIndexExistsBehaviour::DontDownload
    }
}

/// 加载器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadState {
    Waiting,
    Loading,
    Finished,
    Failed,
    Aborted,
}

impl Default for LoadState {
    fn default() -> Self {
        LoadState::Waiting
    }
}

/// Minecraft 客户端版本列表结果
#[derive(Debug, Clone)]
pub struct DlClientListResult {
    /// 是否为官方的实时数据
    pub is_official: bool,
    /// 获取到的 Json 数据
    pub value: JsonValue,
}

/// 网络文件信息
#[derive(Debug, Clone)]
pub struct NetFile {
    /// 下载地址列表
    pub urls: Vec<String>,
    /// 本地路径
    pub local_path: String,
    /// 文件校验信息
    pub checker: FileChecker,
}

/// 文件校验器
#[derive(Debug, Clone)]
pub struct FileChecker {
    /// 最小文件大小
    pub min_size: Option<i64>,
    /// 实际文件大小
    pub actual_size: Option<i64>,
    /// SHA1 哈希
    pub hash: Option<String>,
    /// 是否可以使用已存在的文件
    pub can_use_exists: bool,
    /// 是否为 JSON 文件
    pub is_json: bool,
}

impl Default for FileChecker {
    fn default() -> Self {
        Self {
            min_size: None,
            actual_size: None,
            hash: None,
            can_use_exists: true,
            is_json: false,
        }
    }
}

impl FileChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_min_size(mut self, size: i64) -> Self {
        self.min_size = Some(size);
        self
    }

    pub fn with_actual_size(mut self, size: i64) -> Self {
        self.actual_size = Some(size);
        self
    }

    pub fn with_hash(mut self, hash: impl Into<String>) -> Self {
        self.hash = Some(hash.into());
        self
    }

    pub fn with_can_use_exists(mut self, can_use: bool) -> Self {
        self.can_use_exists = can_use;
        self
    }

    pub fn with_is_json(mut self, is_json: bool) -> Self {
        self.is_json = is_json;
        self
    }

    /// 检查文件是否有效
    pub fn check(&self, path: &str) -> Option<String> {
        // 简化实现，实际应该检查文件大小和哈希
        if std::path::Path::new(path).exists() {
            None
        } else {
            Some("文件不存在".to_string())
        }
    }
}

/// Minecraft 实例信息
#[derive(Debug, Clone)]
pub struct McInstance {
    /// 版本名称
    pub name: String,
    /// 继承版本名称
    pub inherit_name: Option<String>,
    /// JSON 对象
    pub json_object: JsonValue,
    /// 版本路径
    pub path_version: String,
}
