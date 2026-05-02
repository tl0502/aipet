import { invoke } from './index'

// 健康检查 — 主进程返回 ISO8601 时间戳
export function pingCommand(): Promise<string> {
  return invoke<string>('ping')
}

// 上报桌宠包围盒(屏幕物理坐标),供主进程 cursor_tracker 判断穿透
export function updateHitbox(x: number, y: number, w: number, h: number): Promise<void> {
  return invoke<void>('update_hitbox', { x, y, w, h })
}

// 进入拖动状态:主进程切换 is_dragging=true 并调 Tauri start_dragging
export function startDrag(): Promise<void> {
  return invoke<void>('start_drag')
}

// 退出拖动状态:主进程切回 is_dragging=false 并触发屏幕边缘吸附
export function stopDrag(): Promise<void> {
  return invoke<void>('stop_drag')
}
