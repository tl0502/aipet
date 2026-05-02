mod commands;
mod error;
mod services;
mod state;

use commands::{ping, window};
use state::AppState;
use tauri::Manager;

const PET_WINDOW_LABEL: &str = "pet";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("\n=== APP PANIC ===\n{info}\n=================");
    }));

    tauri::Builder::default()
        .manage(AppState::default())
        .setup(|app| {
            eprintln!("[setup] reached");
            crate::services::cursor_tracker::spawn(app.handle().clone());
            crate::services::tray::setup(app.handle())?;
            #[cfg(debug_assertions)]
            if let Some(window) = app.get_webview_window(PET_WINDOW_LABEL) {
                window.open_devtools();
                eprintln!("[setup] devtools opened");
            }
            Ok(())
        })
        // CloseRequested 拦截:Alt+F4 / 系统命令关闭主窗口时不退出进程,
        // 而是 hide 窗口。退出唯一路径 = tray "退出" 菜单或 app.exit()。
        // 理由:桌宠是常驻应用,误触关闭就杀进程会损害用户预期。
        .on_window_event(|window, event| {
            if window.label() == PET_WINDOW_LABEL {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            ping::ping,
            window::update_hitbox,
            window::start_drag,
            window::stop_drag,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
