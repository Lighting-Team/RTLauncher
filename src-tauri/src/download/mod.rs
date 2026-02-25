//! 下载模块
//!
//! 提供高速下载功能，支持多线程、分块下载和多种下载策略

pub mod config;
pub mod downloader;
pub mod task;

pub use config::{DownloadConfig, DownloadStrategy};
pub use downloader::HighSpeedDownloader;
pub use task::DownloadTask;
