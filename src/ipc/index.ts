import { invoke as tauriInvoke } from '@tauri-apps/api/core'

// 架构 §5.3:IPC 字段使用 snake_case(与 SQLite 一致),前端在 binding 层做转换
export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return tauriInvoke<T>(cmd, args)
}

export { listen } from '@tauri-apps/api/event'
