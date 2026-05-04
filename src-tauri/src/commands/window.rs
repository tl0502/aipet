use crate::services::window_snap;
use crate::state::{lock_or_recover, AppState, Hitbox};
use tauri::{State, WebviewWindow};

const MAX_HITBOX_ABS: f64 = 1_000_000.0;

fn validate_hitbox_input(css_x: f64, css_y: f64, css_w: f64, css_h: f64) -> Result<(), String> {
    for (name, value) in [("css_x", css_x), ("css_y", css_y), ("css_w", css_w), ("css_h", css_h)] {
        if !value.is_finite() {
            return Err(format!("invalid hitbox input: {name} is not finite"));
        }
        if value.abs() > MAX_HITBOX_ABS {
            return Err(format!("invalid hitbox input: {name} out of range"));
        }
    }
    if css_w <= 0.0 || css_h <= 0.0 {
        return Err("invalid hitbox input: width/height must be positive".into());
    }
    Ok(())
}

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
    validate_hitbox_input(css_x, css_y, css_w, css_h)?;

    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let pos = window.outer_position().map_err(|e| e.to_string())?;

    let phys_x = pos.x as i64 + (css_x * scale).round() as i64;
    let phys_y = pos.y as i64 + (css_y * scale).round() as i64;
    let phys_w = (css_w * scale).round() as i64;
    let phys_h = (css_h * scale).round() as i64;

    if phys_w <= 0 || phys_h <= 0 {
        return Err("invalid hitbox after scaling: width/height must be positive".into());
    }
    if phys_x < i32::MIN as i64
        || phys_x > i32::MAX as i64
        || phys_y < i32::MIN as i64
        || phys_y > i32::MAX as i64
        || phys_w > i32::MAX as i64
        || phys_h > i32::MAX as i64
    {
        return Err("invalid hitbox after scaling: out of i32 range".into());
    }

    *lock_or_recover(&state.pet_hitbox) = Some(Hitbox {
        x: phys_x as i32,
        y: phys_y as i32,
        w: phys_w as i32,
        h: phys_h as i32,
    });
    Ok(())
}

#[tauri::command]
pub fn start_drag(state: State<'_, AppState>, window: WebviewWindow) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())?;
    *lock_or_recover(&state.is_dragging) = true;
    Ok(())
}

#[tauri::command]
pub fn stop_drag(state: State<'_, AppState>, window: WebviewWindow) -> Result<(), String> {
    *lock_or_recover(&state.is_dragging) = false;
    window_snap::snap_to_edge(&window).map_err(|e| e.to_string())
}
