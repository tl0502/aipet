import { invoke } from './index'

// 健康检查 — 主进程返回 ISO8601 时间戳
export function pingCommand(): Promise<string> {
  return invoke<string>('ping')
}

// 上报桌宠 hitbox 的 CSS 像素坐标(canvas 在 webview viewport 内的偏移 + bbox 在 canvas 内的偏移)。
// Rust 侧用 outer_position + scale_factor 转为屏幕物理像素,供 cursor_tracker 与 GetCursorPos 比较。
// 详见 src-tauri/src/commands/window.rs::update_hitbox 注释。
export function updateHitbox(
  cssX: number,
  cssY: number,
  cssW: number,
  cssH: number
): Promise<void> {
  return invoke<void>('update_hitbox', { cssX, cssY, cssW, cssH })
}

// 进入拖动状态:主进程切换 is_dragging=true 并调 Tauri start_dragging
export function startDrag(): Promise<void> {
  return invoke<void>('start_drag')
}

// 退出拖动状态:主进程切回 is_dragging=false 并触发屏幕边缘吸附
export function stopDrag(): Promise<void> {
  return invoke<void>('stop_drag')
}
