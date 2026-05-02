use tauri::{PhysicalPosition, WebviewWindow};

const SNAP_THRESHOLD_PX: i32 = 30;

// 拖动结束时调用。窗口距屏幕边缘 < 30 逻辑像素 → 自动贴边。
pub fn snap_to_edge(window: &WebviewWindow) -> tauri::Result<()> {
    let monitor = match window.current_monitor()? {
        Some(m) => m,
        None => return Ok(()),
    };
    let mon_size = monitor.size();
    let mon_pos = monitor.position();
    let scale = monitor.scale_factor();

    let win_pos = window.outer_position()?;
    let win_size = window.outer_size()?;

    let win_x = win_pos.x;
    let win_y = win_pos.y;
    let win_w = win_size.width as i32;
    let win_h = win_size.height as i32;
    let mon_x = mon_pos.x;
    let mon_y = mon_pos.y;
    let mon_w = mon_size.width as i32;
    let mon_h = mon_size.height as i32;

    let threshold_phys = (SNAP_THRESHOLD_PX as f64 * scale) as i32;

    let mut new_x = win_x;
    let mut new_y = win_y;

    if (win_x - mon_x).abs() < threshold_phys {
        new_x = mon_x;
    }
    let right_dist = (mon_x + mon_w) - (win_x + win_w);
    if right_dist.abs() < threshold_phys {
        new_x = mon_x + mon_w - win_w;
    }
    if (win_y - mon_y).abs() < threshold_phys {
        new_y = mon_y;
    }
    let bottom_dist = (mon_y + mon_h) - (win_y + win_h);
    if bottom_dist.abs() < threshold_phys {
        new_y = mon_y + mon_h - win_h;
    }

    if new_x != win_x || new_y != win_y {
        window.set_position(PhysicalPosition::new(new_x, new_y))?;
    }

    Ok(())
}
