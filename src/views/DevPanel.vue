<script setup lang="ts">
// DEV-1 主面板容器 — 4 个 tab 切换
import { ref, onMounted } from 'vue'
import IpcPlayground from './dev/IpcPlayground.vue'
import TablesView from './dev/TablesView.vue'
import EventLog from './dev/EventLog.vue'
import LogsView from './dev/LogsView.vue'
import '@/styles/dev.css'

type Tab = 'ipc' | 'tables' | 'events' | 'logs'
const activeTab = ref<Tab>('ipc')

const buildMode = import.meta.env.MODE
const buildTime = new Date().toLocaleString()

onMounted(() => {
  document.title = 'AI 桌宠 DevPanel'
})
</script>

<template>
  <div class="dev-panel">
    <header>
      <h1>AI 桌宠 DevPanel</h1>
      <span class="meta">build={{ buildMode }} · loaded={{ buildTime }}</span>
    </header>

    <nav class="tabs">
      <button :class="{ active: activeTab === 'ipc' }" @click="activeTab = 'ipc'">
        IPC Playground
      </button>
      <button :class="{ active: activeTab === 'tables' }" @click="activeTab = 'tables'">
        Tables
      </button>
      <button :class="{ active: activeTab === 'events' }" @click="activeTab = 'events'">
        Events
      </button>
      <button :class="{ active: activeTab === 'logs' }" @click="activeTab = 'logs'">
        Logs
      </button>
    </nav>

    <div class="tab-content">
      <IpcPlayground v-if="activeTab === 'ipc'" />
      <TablesView v-else-if="activeTab === 'tables'" />
      <EventLog v-else-if="activeTab === 'events'" />
      <LogsView v-else-if="activeTab === 'logs'" />
    </div>
  </div>
</template>
