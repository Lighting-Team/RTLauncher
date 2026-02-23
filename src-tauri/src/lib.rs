use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;

#[cfg(target_os = "macos")]
use objc2_app_kit::{NSColor, NSWindow};

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

            // 为所有平台创建主窗口（如果还不存在）
            let window = if let Some(window) = app.get_webview_window("main") {
                window
            } else {
                let win_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                    .title("RTLauncher")
                    .inner_size(1200.0, 800.0)
                    .center()
                    .resizable(true)
                    .fullscreen(false)
                    .shadow(true);

                // macOS: 使用透明标题栏
                #[cfg(target_os = "macos")]
                let win_builder = win_builder.title_bar_style(TitleBarStyle::Transparent);

                // Windows/Linux: 使用无边框窗口（完全自定义标题栏）
                #[cfg(not(target_os = "macos"))]
                let win_builder = win_builder.decorations(false);

                win_builder.build()?
            };

            let _window = window;

            // macOS: 设置窗口背景颜色
            #[cfg(target_os = "macos")]
            unsafe {
                // 获取原生窗口句柄并转换为 objc2 的 NSWindow
                let ns_window_ptr = _window.ns_window().unwrap() as *mut objc2::runtime::AnyObject;
                let ns_window = &*(ns_window_ptr as *const NSWindow);

                // 创建背景颜色 (sRGB 色彩空间)
                let bg_color = NSColor::colorWithSRGBRed_green_blue_alpha(
                    50.0 / 255.0,
                    158.0 / 255.0,
                    163.5 / 255.0,
                    1.0,
                );

                // 设置窗口背景色
                ns_window.setBackgroundColor(Some(&bg_color));
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
