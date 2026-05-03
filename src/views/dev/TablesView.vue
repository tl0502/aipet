<script setup lang="ts">
// DEV-1 Tables 查看器 — 选表名 + 拉前 N 行
import { ref, onMounted } from 'vue'
import { invoke } from '@/ipc'

const tables = ref<string[]>([])
const selected = ref<string>('')
const rows = ref<Record<string, unknown>[]>([])
const error = ref<string>('')
const loading = ref(false)

const ALLOWED = ['personas', 'messages', 'nicknames', 'persona_snapshots', 'conversations']

onMounted(async () => {
  try {
    tables.value = await invoke<string[]>('dev_list_tables')
  } catch (e) {
    error.value = `dev_list_tables failed: ${e}`
  }
})

async function loadTable(name: string) {
  selected.value = name
  rows.value = []
  error.value = ''
  loading.value = true
  try {
    const r = await invoke<Record<string, unknown>[]>('dev_query_table', { table: name, limit: 100 })
    rows.value = r
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    loading.value = false
  }
}

function columns(): string[] {
  if (rows.value.length === 0) return []
  return Object.keys(rows.value[0])
}

function fmtCell(v: unknown): string {
  if (v === null || v === undefined) return '∅'
  if (typeof v === 'string') return v
  return JSON.stringify(v)
}
</script>

<template>
  <div>
    <div style="margin-bottom: 12px;">
      <span style="color: var(--text-muted); margin-right: 8px;">tables:</span>
      <button
        v-for="t in tables"
        :key="t"
        :disabled="!ALLOWED.includes(t)"
        :title="ALLOWED.includes(t) ? '' : '不在 dev_query_table 白名单里'"
        :class="['action', selected === t ? '' : '']"
        :style="{
          marginRight: '6px',
          background: selected === t ? 'var(--accent)' : 'var(--bg-alt)',
          color: ALLOWED.includes(t) ? 'var(--text)' : 'var(--text-muted)',
          border: '1px solid var(--border)',
        }"
        @click="loadTable(t)"
      >
        {{ t }}
      </button>
    </div>

    <div v-if="loading" style="color: var(--text-muted);">loading...</div>

    <pre v-if="error" class="error">{{ error }}</pre>

    <div v-if="rows.length > 0">
      <div style="color: var(--text-muted); margin-bottom: 8px;">
        {{ selected }}: {{ rows.length }} rows (limit 100)
      </div>
      <table>
        <thead>
          <tr>
            <th v-for="c in columns()" :key="c">{{ c }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(row, i) in rows" :key="i">
            <td v-for="c in columns()" :key="c" :title="fmtCell(row[c])">
              {{ fmtCell(row[c]) }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-else-if="selected && !loading && !error" style="color: var(--text-muted);">
      (table is empty)
    </div>
  </div>
</template>
