<script setup lang="ts">
// DEV-1 Logs 视图 — MVP 占位,显示 dev_get_logs stub 返回的固定字符串
// 真 ringbuffer logger 待 M1 D6+ 接入 tracing-subscriber
import { ref, onMounted } from 'vue'
import { invoke } from '@/ipc'

const logs = ref<string[]>([])
const error = ref<string>('')

async function refresh() {
  error.value = ''
  try {
    logs.value = await invoke<string[]>('dev_get_logs')
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  }
}

onMounted(refresh)
</script>

<template>
  <div>
    <div style="margin-bottom: 12px; display: flex; align-items: center; gap: 12px;">
      <button class="action" @click="refresh">refresh</button>
      <span style="color: var(--text-muted);">
        MVP 占位 — 待 ringbuffer logger 接入(M1 D6+)。当前 eprintln 路由到 stderr,看终端窗口。
      </span>
    </div>

    <pre v-if="error" class="error">{{ error }}</pre>

    <pre v-if="logs.length > 0">{{ logs.join('\n') }}</pre>
  </div>
</template>
