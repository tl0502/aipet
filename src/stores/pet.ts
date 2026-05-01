import { defineStore } from 'pinia'
import { ref } from 'vue'

export const usePetStore = defineStore('pet', () => {
  const lastPing = ref<string | null>(null)

  function setLastPing(value: string) {
    lastPing.value = value
  }

  return { lastPing, setLastPing }
})
