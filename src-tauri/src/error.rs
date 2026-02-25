use thiserror::Error;

#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("网络请求失败: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("JSON解析失败: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("正则表达式错误: {0}")]
    Regex(#[from] regex::Error),
    
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("版本列表解析失败: {0}")]
    VersionListParse(String),
    
    #[error("未找到下载信息: {0}")]
    DownloadInfoNotFound(String),
    
    #[error("加载器执行失败: {0}")]
    LoaderExecution(String),
    
    #[error("超时")]
    Timeout,
    
    #[error("任务被中止")]
    Aborted,
    
    #[error("未知错误: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, DownloadError>;
