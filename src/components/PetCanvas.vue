<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue'
import { useVRMModel } from '@/composables/useVRMModel'
import { startDrag, stopDrag, updateHitbox } from '@/ipc/commands'
import { usePetStore } from '@/stores/pet'

const canvasRef = ref<HTMLCanvasElement | null>(null)
const petStore = usePetStore()

const MODEL_URL = '/avatar/avatar.vrm'
const HITBOX_REPORT_INTERVAL_MS = 250

const { isLoaded, errorMessage, runtime } = useVRMModel(canvasRef, MODEL_URL)

let hitboxTimer: ReturnType<typeof setInterval> | null = null

function reportHitbox() {
  if (!canvasRef.value) return
  const bounds = runtime.getBounds()
  if (!bounds) return

  const rect = canvasRef.value.getBoundingClientRect()
  const screenX = Math.round(window.screenX + rect.left + bounds.x)
  const screenY = Math.round(window.screenY + rect.top + bounds.y)
  const w = Math.round(bounds.width)
  const h = Math.round(bounds.height)

  petStore.setBounds({ x: screenX, y: screenY, w, h })
  updateHitbox(screenX, screenY, w, h).catch((err) =>
    console.error('[hitbox] update failed', err)
  )
}

watch(isLoaded, (loaded) => {
  if (loaded && hitboxTimer === null) {
    reportHitbox()
    hitboxTimer = setInterval(reportHitbox, HITBOX_REPORT_INTERVAL_MS)
  }
})

function handleMouseDown(_event: MouseEvent) {
  petStore.setDragging(true)
  startDrag().catch((err) => console.error('[drag] start failed', err))
}

function handleMouseUp() {
  if (!petStore.isDragging) return
  petStore.setDragging(false)
  stopDrag().catch((err) => console.error('[drag] stop failed', err))
}

document.addEventListener('mouseup', handleMouseUp)

onBeforeUnmount(() => {
  if (hitboxTimer !== null) {
    clearInterval(hitboxTimer)
    hitboxTimer = null
  }
  document.removeEventListener('mouseup', handleMouseUp)
})
</script>

<template>
  <div class="pet-stage">
    <canvas ref="canvasRef" class="pet-canvas" width="320" height="320" @mousedown="handleMouseDown"></canvas>
    <div v-if="!isLoaded && !errorMessage" class="hint">Loading VRM…</div>
    <div v-else-if="errorMessage" class="hint hint-error">
      VRM 加载失败:{{ errorMessage }}<br />
      请把一个 .vrm 文件放在 <code>public/avatar/avatar.vrm</code>(下载来源见 README)
    </div>
  </div>
</template>

<style scoped>
.pet-stage {
  position: relative;
  width: 320px;
  height: 320px;
}

.pet-canvas {
  display: block;
  width: 320px;
  height: 320px;
}

.hint {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: 12px;
  color: #555;
  background: rgba(255, 255, 255, 0.9);
  padding: 4px 12px;
  border-radius: 6px;
  text-align: center;
  pointer-events: none;
  white-space: nowrap;
}

.hint-error {
  background: rgba(255, 220, 220, 0.95);
  color: #722;
  font-size: 10px;
  max-width: 290px;
  white-space: normal;
  line-height: 1.4;
}

.hint-error code {
  background: rgba(0, 0, 0, 0.07);
  padding: 1px 4px;
  border-radius: 3px;
  font-family: ui-monospace, monospace;
}
</style>