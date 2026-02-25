pub mod client_list;
pub mod download;
pub mod error;
pub mod models;
pub mod source;
pub mod task;
pub mod tasks;
pub mod utils;

pub use download::{DownloadConfig, DownloadStrategy, DownloadTask, HighSpeedDownloader};
pub use error::{DownloadError, Result};
pub use models::*;
pub use task::*;
pub use tasks::*;


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
