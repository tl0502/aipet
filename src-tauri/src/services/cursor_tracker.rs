use crate::state::AppState;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager};
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

// 60Hz 光标追踪 + 5px 滞后区,平滑切换 set_ignore_cursor_events。
// 详见 progress/m1.md § A.3 智能穿透 + 边缘吸附。
const TICK_INTERVAL_MS: u64 = 16;
const HYSTERESIS_PX: i32 = 5;

pub fn spawn(app: AppHandle) {
    thread::spawn(move || {
        // None = 从未调用过 set_ignore_cursor_events,首次 want_ignore 必触发 set
        // (修补 commit 42bb4c7 的 regression:老代码 loop 前显式 init 为 true,
        //  改进时丢了这一步,导致启动 1-2s 内桌宠区域实际不穿透)
        let mut last_ignore: Option<bool> = None;

        loop {
            thread::sleep(Duration::from_millis(TICK_INTERVAL_MS));

            let Some(window) = app.get_webview_window("pet") else {
                break;
            };

            let cursor = match get_cursor_pos() {
                Some(p) => p,
                None => continue,
            };

            let state = app.state::<AppState>();
            let is_dragging = *state.is_dragging.lock().unwrap();
            let hitbox = *state.pet_hitbox.lock().unwrap();

            let want_ignore = if is_dragging {
                false
            } else if let Some(box_) = hitbox {
                // 滞后区判断:None(首次)按"目前在穿透"处理,走 contains 严格判断
                if last_ignore.unwrap_or(true) {
                    !box_.contains(cursor.0, cursor.1)
                } else {
                    !box_.expand(HYSTERESIS_PX).contains(cursor.0, cursor.1)
                }
            } else {
                true
            };

            if last_ignore != Some(want_ignore) {
                if window.set_ignore_cursor_events(want_ignore).is_err() {
                    break;
                }
                last_ignore = Some(want_ignore);
            }
        }
    });
}

fn get_cursor_pos() -> Option<(i32, i32)> {
    let mut point = POINT::default();
    unsafe {
        GetCursorPos(&mut point).ok()?;
    }
    Some((point.x, point.y))
}
