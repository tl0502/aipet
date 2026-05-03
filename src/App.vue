<script setup lang="ts">
import { defineAsyncComponent, shallowRef } from 'vue'
import Pet from '@/views/Pet.vue'

// hash 路由 — `index.html#dev` 切到 DevPanel(仅 debug build);否则桌宠主壳
// import.meta.env.DEV 在 vite production build 时 = false,DevPanel 整段被 dead-code 消除
const isDev = window.location.hash === '#dev' && import.meta.env.DEV
const DevPanel = shallowRef(
  isDev ? defineAsyncComponent(() => import('@/views/DevPanel.vue')) : null
)
</script>

<template>
  <component :is="DevPanel" v-if="isDev" />
  <Pet v-else />
</template>
