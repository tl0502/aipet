mod commands;
mod error;
mod services;
mod state;

use commands::{ping, window};
use state::AppState;
use tauri::Manager;

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
            #[cfg(debug_assertions)]
            if let Some(window) = app.get_webview_window("pet") {
                window.open_devtools();
                eprintln!("[setup] devtools opened");
            }
            Ok(())
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
