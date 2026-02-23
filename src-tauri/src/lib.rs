use tauri::{WebviewUrl, WebviewWindowBuilder};

#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;

#[cfg(target_os = "macos")]
use cocoa::appkit::NSColor;
#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};

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

            // 为所有平台创建主窗口
            let win_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                .title("RTLauncher")
                .inner_size(1200.0, 800.0)
                .center()
                .resizable(true)
                .fullscreen(false)
                .transparent(true)
                .shadow(true);

            // macOS: 使用透明标题栏
            #[cfg(target_os = "macos")]
            let win_builder = win_builder.title_bar_style(TitleBarStyle::Transparent);

            // Windows/Linux: 使用无边框窗口（完全自定义标题栏）
            #[cfg(not(target_os = "macos"))]
            let win_builder = win_builder.decorations(false);

            let _window = win_builder.build().unwrap();

            // macOS: 设置窗口背景颜色
            #[cfg(target_os = "macos")]
            unsafe {
                let ns_window = _window.ns_window().unwrap() as id;
                let bg_color = NSColor::colorWithRed_green_blue_alpha_(
                    nil,
                    50.0 / 255.0,
                    158.0 / 255.0,
                    163.5 / 255.0,
                    1.0,
                );
                ns_window.setBackgroundColor_(bg_color);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
