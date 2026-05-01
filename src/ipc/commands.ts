import { invoke } from './index'

// 健康检查 — 主进程返回 ISO8601 时间戳
export function pingCommand(): Promise<string> {
  return invoke<string>('ping')
}
