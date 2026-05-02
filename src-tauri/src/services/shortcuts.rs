// 全局快捷键(A.5)
//
// M1 范围(最小可行):
// - Ctrl+Alt+Space → toggle 主窗口显示/隐藏 + emit("shortcut:chat")
//   (M1 后期 B.3 ChatPanel 上线后,前端监听此事件打开对话面板)
// - Ctrl+Shift+B  → hide 主窗口 + emit("shortcut:boss-key")
//   (M2 BossKeyService 接管,届时变成多窗口隐藏 + 提醒缓冲 + 托盘图标变更)
//
// 设计要点:
// - 纯 Rust 端注册;不暴露 register/unregister 给前端,因此不需 capabilities/ 配置
//   (M3 设置面板让用户改键时再加 capability + npm 包)
// - 仅按下边沿(ShortcutState::Pressed)触发,减误触
// - register 失败仅 log 不阻塞:其他应用(输入法 / IDE 等)占用快捷键是常态,
//   托盘菜单仍可作为 backup 入口
// - 事件命名空间 `shortcut:*`,与 tray 的 `tray:*` 区分

use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::services::window_actions;

const EVENT_CHAT: &str = "shortcut:chat";
const EVENT_BOSS_KEY: &str = "shortcut:boss-key";

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let chat = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::Space);
    let boss = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyB);

    app.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, sc, event| {
                if event.state() != ShortcutState::Pressed {
                    return;
                }
                if sc == &chat {
                    window_actions::toggle_pet(app);
                    let _ = app.emit(EVENT_CHAT, ());
                } else if sc == &boss {
                    window_actions::hide_pet(app);
                    let _ = app.emit(EVENT_BOSS_KEY, ());
                }
            })
            .build(),
    )?;

    if let Err(e) = app.global_shortcut().register(chat) {
        eprintln!("[shortcuts] failed to register Ctrl+Alt+Space: {e}");
    } else {
        eprintln!("[shortcuts] registered Ctrl+Alt+Space → chat");
    }
    if let Err(e) = app.global_shortcut().register(boss) {
        eprintln!("[shortcuts] failed to register Ctrl+Shift+B: {e}");
    } else {
        eprintln!("[shortcuts] registered Ctrl+Shift+B → boss-key");
    }
    Ok(())
}
