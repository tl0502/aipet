use crate::services::window_snap;
use crate::state::{AppState, Hitbox};
use tauri::{State, WebviewWindow};

/// 接收 CSS 像素的 hitbox(canvas 在 webview 视口内的位置 + bbox 在 canvas 内的偏移之和),
/// 内部用 window.outer_position() + window.scale_factor() 转为屏幕物理像素后存入 AppState。
///
/// 为什么在 Rust 侧转换:
/// - cursor_tracker 用 GetCursorPos(物理像素)与 hitbox 比较,单位必须一致
/// - 前端调 `@tauri-apps/api/window.scaleFactor / outerPosition` 需要 capability 授权;
///   而本应用 capabilities 为空,放在 Rust 侧零额外授权
/// - 拖动 / DPR 变化时,Rust 在每次上报当下读取最新值,无须前端再订阅 onMoved/onScaleChanged
#[tauri::command]
pub fn update_hitbox(
    css_x: f64,
    css_y: f64,
    css_w: f64,
    css_h: f64,
    state: State<'_, AppState>,
    window: WebviewWindow,
) -> Result<(), String> {
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let pos = window.outer_position().map_err(|e| e.to_string())?;

    let phys_x = pos.x + (css_x * scale).round() as i32;
    let phys_y = pos.y + (css_y * scale).round() as i32;
    let phys_w = (css_w * scale).round() as i32;
    let phys_h = (css_h * scale).round() as i32;

    *state.pet_hitbox.lock().unwrap() = Some(Hitbox {
        x: phys_x,
        y: phys_y,
        w: phys_w,
        h: phys_h,
    });
    Ok(())
}

#[tauri::command]
pub fn start_drag(state: State<'_, AppState>, window: WebviewWindow) -> Result<(), String> {
    *state.is_dragging.lock().unwrap() = true;
    window.start_dragging().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stop_drag(state: State<'_, AppState>, window: WebviewWindow) -> Result<(), String> {
    *state.is_dragging.lock().unwrap() = false;
    window_snap::snap_to_edge(&window).map_err(|e| e.to_string())
}
