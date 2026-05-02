<script setup lang="ts">
import PetCanvas from '@/components/PetCanvas.vue'
</script>

<template>
  <div class="pet-window">
    <PetCanvas />
  </div>
</template>

<style scoped>
.pet-window {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100vw;
  height: 100vh;
}
</style>
