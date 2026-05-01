<script setup lang="ts">
// TODO(M1 W1 D2-D3): 引入 Live2D Cubism 4 Web SDK,加载 personas/_builtin/momo 模型(ADR-002)
</script>

<template>
  <div class="pet-canvas-placeholder">
    <span class="placeholder-text">PetCanvas</span>
    <span class="placeholder-hint">Live2D 待 D2 引入</span>
  </div>
</template>

<style scoped>
.pet-canvas-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 200px;
  height: 200px;
  border: 2px dashed rgba(150, 150, 150, 0.7);
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.6);
  user-select: none;
}

.placeholder-text {
  font-size: 14px;
  color: #444;
  font-weight: 500;
}

.placeholder-hint {
  font-size: 10px;
  color: #888;
  margin-top: 4px;
}
</style>
