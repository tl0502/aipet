<script setup lang="ts">
// DEV-1 IPC Playground — 选 command + 表单填参 + 调用 + 显示返回值
import { ref, computed } from 'vue'
import { invoke } from '@/ipc'
import { COMMAND_REGISTRY, type CommandMeta } from '@/ipc/dev'

const selected = ref<CommandMeta | null>(null)
const paramValues = ref<Record<string, string>>({})
const result = ref<string>('')
const error = ref<string>('')
const loading = ref(false)

// 按 category 分组
const grouped = computed(() => {
  const map = new Map<string, CommandMeta[]>()
  for (const cmd of COMMAND_REGISTRY) {
    if (!map.has(cmd.category)) map.set(cmd.category, [])
    map.get(cmd.category)!.push(cmd)
  }
  return Array.from(map.entries())
})

function selectCommand(cmd: CommandMeta) {
  selected.value = cmd
  paramValues.value = {}
  for (const p of cmd.params) {
    paramValues.value[p.name] = ''
  }
  result.value = ''
  error.value = ''
}

async function callCommand() {
  if (!selected.value) return
  loading.value = true
  result.value = ''
  error.value = ''

  try {
    const args: Record<string, unknown> = {}
    for (const p of selected.value.params) {
      const raw = paramValues.value[p.name] ?? ''
      if (p.required && raw === '') {
        throw new Error(`required param missing: ${p.name}`)
      }
      if (raw === '' && !p.required) continue
      // 类型转换
      if (p.type === 'number' || p.type === 'number?') {
        const n = Number(raw)
        if (Number.isNaN(n)) throw new Error(`${p.name} not a number: ${raw}`)
        args[p.name] = n
      } else if (p.type === 'boolean') {
        args[p.name] = raw === 'true' || raw === '1'
      } else {
        args[p.name] = raw
      }
    }
    const r = await invoke(selected.value.name, args)
    result.value = JSON.stringify(r, null, 2)
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="split">
    <div class="left">
      <div v-for="[cat, cmds] in grouped" :key="cat" style="margin-bottom: 12px;">
        <div style="color: var(--text-muted); font-size: 11px; margin-bottom: 4px; text-transform: uppercase;">
          {{ cat }}
        </div>
        <ul class="command-list">
          <li
            v-for="cmd in cmds"
            :key="cmd.name"
            :class="{ active: selected?.name === cmd.name }"
            @click="selectCommand(cmd)"
          >
            {{ cmd.name }}
          </li>
        </ul>
      </div>
    </div>

    <div class="right">
      <div v-if="!selected" style="color: var(--text-muted);">
        ← 选一个 command
      </div>

      <div v-else>
        <h3 style="margin-top: 0;">{{ selected.name }}</h3>
        <p style="color: var(--text-muted); margin-bottom: 16px;">{{ selected.description }}</p>

        <div v-if="selected.params.length === 0" style="color: var(--text-muted); margin-bottom: 12px;">
          (无参数)
        </div>
        <div v-else style="margin-bottom: 12px;">
          <div v-for="p in selected.params" :key="p.name" class="form-row">
            <label>{{ p.name }} <span style="color: var(--text-muted); font-size: 10px;">{{ p.type }}{{ p.required ? '*' : '' }}</span></label>
            <input v-model="paramValues[p.name]" :placeholder="p.description ?? ''" />
          </div>
        </div>

        <button class="action" :disabled="loading" @click="callCommand">
          {{ loading ? 'calling...' : 'invoke' }}
        </button>

        <div v-if="result" style="margin-top: 16px;">
          <div style="color: var(--success); margin-bottom: 4px;">return:</div>
          <pre class="success">{{ result }}</pre>
        </div>

        <div v-if="error" style="margin-top: 16px;">
          <div style="color: var(--error); margin-bottom: 4px;">error:</div>
          <pre class="error">{{ error }}</pre>
        </div>
      </div>
    </div>
  </div>
</template>
