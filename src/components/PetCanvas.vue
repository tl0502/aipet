<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useVRMModel } from '@/composables/useVRMModel'
import { startDrag, stopDrag, updateHitbox } from '@/ipc/commands'
import { usePetStore } from '@/stores/pet'

const canvasRef = ref<HTMLCanvasElement | null>(null)
const petStore = usePetStore()

const MODEL_URL = '/avatar/avatar.vrm'
const HITBOX_REPORT_INTERVAL_MS = 250

const { isLoaded, errorMessage, runtime } = useVRMModel(canvasRef, MODEL_URL)

let hitboxTimer: ReturnType<typeof setInterval> | null = null

/**
 * 上报 hitbox(CSS 像素,Rust 侧转物理像素)。
 *
 * 设计要点:
 * - 当 VRM 未加载好(或加载失败)时,fallback 为整个 canvas 区域,
 *   避免 cursor_tracker 因 hitbox=None 让整窗穿透 → 用户既不能点 hint
 *   也不能拖动窗口的死锁
 * - 单位:rect.left + bounds.x 是相对 webview viewport 的 CSS 像素;
 *   Rust 端会读 outer_position + scale_factor 转屏幕物理像素
 */
function reportHitbox() {
  if (!canvasRef.value) return

  const rect = canvasRef.value.getBoundingClientRect()
  const local = runtime.getBounds() ?? {
    x: 0,
    y: 0,
    width: rect.width,
    height: rect.height,
  }

  const cssX = rect.left + local.x
  const cssY = rect.top + local.y
  const cssW = local.width
  const cssH = local.height

  if (
    !Number.isFinite(cssX) ||
    !Number.isFinite(cssY) ||
    !Number.isFinite(cssW) ||
    !Number.isFinite(cssH) ||
    cssW <= 0 ||
    cssH <= 0
  ) {
    return
  }

  petStore.setBounds({ x: cssX, y: cssY, w: cssW, h: cssH })
  updateHitbox(cssX, cssY, cssW, cssH).catch((err) =>
    console.error('[hitbox] update failed', err)
  )
}

function startHitboxReporting() {
  if (hitboxTimer !== null) return
  reportHitbox()
  hitboxTimer = setInterval(reportHitbox, HITBOX_REPORT_INTERVAL_MS)
}

// VRM 加载完成 OR 加载失败,均启动 hitbox 上报
// 失败时 fallback 整 canvas 区域,确保用户至少能拖动 / 关闭窗口
watch([isLoaded, errorMessage], ([loaded, error]) => {
  if (loaded || error) startHitboxReporting()
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

onMounted(() => {
  // mouseup 在 document 监听:鼠标可能在窗口外抬起(拖动到屏幕边缘),
  // 仍需收到通知 → stopDrag → 触发边缘吸附
  document.addEventListener('mouseup', handleMouseUp)
})

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
