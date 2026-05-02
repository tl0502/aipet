import { onBeforeUnmount, onMounted } from 'vue'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { listen } from '@/ipc'

/**
 * 监听全局快捷键事件(A.5,M1 占位)
 *
 * 当前仅 console.log,作为快捷键链路的端到端验证。
 *
 * 后续接管者:
 * - `shortcut:chat`     → B.3 ChatPanel 上线后改为打开/聚焦对话面板
 * - `shortcut:boss-key` → M2 模块 K BossKeyService 接管(多窗口隐藏 + 提醒缓冲)
 *
 * 注:Rust 端已经做了 toggle/hide 主窗口的动作,此处仅订阅事件用于 UI 联动,
 * 不在前端再次操作窗口可见性,避免双重 dispatch。
 */
export function useShortcutListener() {
  const unlisten: UnlistenFn[] = []

  onMounted(async () => {
    unlisten.push(
      await listen('shortcut:chat', () => {
        console.log('[shortcut] chat')
      })
    )
    unlisten.push(
      await listen('shortcut:boss-key', () => {
        console.log('[shortcut] boss-key')
      })
    )
  })

  onBeforeUnmount(() => {
    for (const fn of unlisten) fn()
  })
}
