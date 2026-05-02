// 系统托盘服务(A.4)
//
// 行为:
// - 启动时注册到 Windows 托盘隐藏图标区(任务栏 ^ 展开后可见,用户可手动拖出常驻)
// - 左键双击托盘图标 → toggle 桌宠窗口显示 / 隐藏
// - 右键 → 弹出菜单(显示 / 隐藏 / 设置[占位] / 退出)
// - 主窗口 CloseRequested 事件被拦截(在 lib.rs),改为 hide;退出唯一路径是托盘"退出"菜单
//
// 设计要点:
// - icon 复用 src-tauri/icons/icon.ico(已被 tauri.conf.json 引用作 default window icon),零新增资源
// - tooltip 字符串使用与 productName 一致的"AI 桌宠"
// - 窗口动作 helper 抽到 services/window_actions,被 shortcuts 共用

use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::AppHandle;

use crate::services::window_actions;

const MENU_ID_SHOW: &str = "tray:show";
const MENU_ID_HIDE: &str = "tray:hide";
const MENU_ID_SETTINGS: &str = "tray:settings";
const MENU_ID_QUIT: &str = "tray:quit";

const TOOLTIP: &str = "AI 桌宠";

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let menu = MenuBuilder::new(app)
        .item(&MenuItemBuilder::with_id(MENU_ID_SHOW, "显示桌宠").build(app)?)
        .item(&MenuItemBuilder::with_id(MENU_ID_HIDE, "隐藏桌宠").build(app)?)
        .separator()
        .item(
            &MenuItemBuilder::with_id(MENU_ID_SETTINGS, "设置...")
                .enabled(false) // M3 激活,目前占位灰色
                .build(app)?,
        )
        .separator()
        .item(&MenuItemBuilder::with_id(MENU_ID_QUIT, "退出").build(app)?)
        .build()?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| tauri::Error::AssetNotFound("default window icon".into()))?;

    let _tray = TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip(TOOLTIP)
        .menu(&menu)
        .show_menu_on_left_click(false) // 左键留给双击 toggle;右键才弹菜单(Windows 习惯)
        .on_menu_event(|app, event| match event.id().as_ref() {
            MENU_ID_SHOW => window_actions::show_pet(app),
            MENU_ID_HIDE => window_actions::hide_pet(app),
            MENU_ID_QUIT => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // 左键单击:不动作(避免与"双击 toggle"冲突)
            // 左键双击:toggle 显示 / 隐藏
            if let TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } = event
            {
                window_actions::toggle_pet(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}
