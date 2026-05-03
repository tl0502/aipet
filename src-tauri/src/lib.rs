mod commands;
mod error;
mod services;
mod state;

use commands::{ping, window};
use services::window_actions::PET_WINDOW_LABEL;
use state::AppState;
use tauri::Manager;
use tauri_plugin_sql::{Migration, MigrationKind};

const DB_URL: &str = "sqlite:aipet.db";

/// SQLite migrations(I.1 MigrationService,M1 D5)。
///
/// 每次新加 migration 必须:
/// - 用单调递增的 version
/// - **不修改**已发布的 migration(sqlx 已记录 hash,改动会导致用户启动失败 "migration X was previously applied but has been modified")
/// - 新增字段用 ALTER TABLE 走新 migration(00X_xxx.sql)
///
/// schema 详见 docs/AIPET-obsidian/架构设计/...v1.0.md §4(已升 v1.1)。
fn migrations() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        description: "init schema v1 per architecture v1.1 §4 (ADR-015 三形态共享 ConversationStore)",
        sql: include_str!("../migrations/001_init.sql"),
        kind: MigrationKind::Up,
    }]
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("\n=== APP PANIC ===\n{info}\n=================");
    }));

    tauri::Builder::default()
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations(DB_URL, migrations())
                .build(),
        )
        .manage(AppState::default())
        .setup(|app| {
            eprintln!("[setup] reached");
            crate::services::cursor_tracker::spawn(app.handle().clone());
            crate::services::tray::setup(app.handle())?;
            crate::services::shortcuts::setup(app.handle())?;
            // H.1 内置人格 seed:plugin migrations 已建表,这里 UPSERT momo 行
            // 异步跑 + 失败仅 eprintln,不阻塞启动也不弹错误 UI(MVP 期)
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = crate::services::persona::seed_builtin(&app_handle).await {
                    eprintln!("[setup] seed_builtin failed: {e}");
                }
            });
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
