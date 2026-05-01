<script setup lang="ts">
import PetCanvas from '@/components/PetCanvas.vue'
import { usePetStore } from '@/stores/pet'
import { pingCommand } from '@/ipc/commands'

const petStore = usePetStore()

async function handlePing() {
  const result = await pingCommand()
  petStore.setLastPing(result)
  console.log('pong:', result)
}
</script>

<template>
  <div class="pet-window">
    <PetCanvas />
    <div class="debug-panel">
      <button class="ping-btn" @click="handlePing">Ping</button>
      <p v-if="petStore.lastPing" class="ping-result">
        last ping: {{ petStore.lastPing }}
      </p>
    </div>
  </div>
</template>

<style scoped>
.pet-window {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100vw;
  height: 100vh;
  gap: 8px;
}

.debug-panel {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.ping-btn {
  padding: 6px 16px;
  border: 1px solid rgba(120, 120, 120, 0.6);
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.85);
  cursor: pointer;
  font-size: 12px;
}

.ping-btn:hover {
  background: rgba(255, 255, 255, 1);
}

.ping-result {
  font-size: 10px;
  color: #555;
  background: rgba(255, 255, 255, 0.7);
  padding: 2px 6px;
  border-radius: 4px;
  margin: 0;
  max-width: 280px;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
