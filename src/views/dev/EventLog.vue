<script setup lang="ts">
// DEV-1 Events 日志流 — listen 架构 §711 列出的 IPC events,实时滚动展示
import { ref, onMounted, onUnmounted } from 'vue'
import { listen } from '@/ipc'
import { TRACKED_EVENTS } from '@/ipc/dev'
import type { UnlistenFn } from '@tauri-apps/api/event'

interface EventLine {
  ts: string
  name: string
  payload: string
}

const events = ref<EventLine[]>([])
const MAX_LINES = 200
const unlisteners = ref<UnlistenFn[]>([])

function fmtTs(): string {
  const d = new Date()
  return d.toLocaleTimeString('en-US', { hour12: false }) + '.' + String(d.getMilliseconds()).padStart(3, '0')
}

function pushEvent(name: string, payload: unknown) {
  events.value.push({
    ts: fmtTs(),
    name,
    payload: payload === undefined ? '' : JSON.stringify(payload),
  })
  if (events.value.length > MAX_LINES) {
    events.value.splice(0, events.value.length - MAX_LINES)
  }
}

onMounted(async () => {
  for (const ev of TRACKED_EVENTS) {
    const un = await listen(ev, (e) => pushEvent(ev, e.payload))
    unlisteners.value.push(un)
  }
})

onUnmounted(() => {
  for (const un of unlisteners.value) un()
})

function clear() {
  events.value = []
}
</script>

<template>
  <div>
    <div style="margin-bottom: 12px; display: flex; align-items: center; gap: 12px;">
      <span style="color: var(--text-muted);">
        listening on {{ TRACKED_EVENTS.length }} events · {{ events.length }} captured
      </span>
      <button class="action" @click="clear">clear</button>
    </div>

    <div v-if="events.length === 0" style="color: var(--text-muted);">
      (no events yet — try invoking nickname_set_pet or pressing Ctrl+Alt+Space)
    </div>

    <div v-else>
      <div v-for="(ev, i) in events" :key="i" class="event-line">
        <span class="ts">{{ ev.ts }}</span>
        <span class="name">{{ ev.name }}</span>
        <span>{{ ev.payload }}</span>
      </div>
    </div>
  </div>
</template>
