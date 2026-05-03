// DEV-1 开发窗口服务(debug-only)
//
// 全文 #[cfg(debug_assertions)] — release 构建编译期排除。
//
// 行为:
// - 接收 Ctrl+Shift+D 触发(shortcuts.rs 调用 open_dev_window)
// - 已存在 → show + focus(避免重复创建)
// - 不存在 → WebviewWindowBuilder::new 创建,加载 index.html#dev
//   前端 App.vue 看 URL hash 切到 DevPanel 组件
// - label = "dev",capabilities/dev.json 已授权该 window
//
// 设计要点:
// - 失败仅 eprintln,不阻塞桌宠主流程
// - 不持久化 dev 窗口位置/尺寸(每次新建用默认值即可,开发期不需要记忆)

#![cfg(debug_assertions)]

use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

const DEV_WINDOW_LABEL: &str = "dev";

pub fn open_dev_window<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if let Some(win) = app.get_webview_window(DEV_WINDOW_LABEL) {
        win.show()?;
        win.set_focus()?;
        eprintln!("[dev_window] focused existing dev window");
        return Ok(());
    }
    WebviewWindowBuilder::new(
        app,
        DEV_WINDOW_LABEL,
        WebviewUrl::App("index.html#dev".into()),
    )
    .title("AI 桌宠 DevPanel")
    .inner_size(900.0, 700.0)
    .resizable(true)
    .build()?;
    eprintln!("[dev_window] dev window created (Ctrl+Shift+D)");
    Ok(())
}
