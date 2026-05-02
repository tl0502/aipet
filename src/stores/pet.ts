import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface PetBounds {
  x: number
  y: number
  w: number
  h: number
}

export const usePetStore = defineStore('pet', () => {
  const lastPing = ref<string | null>(null)
  const bounds = ref<PetBounds | null>(null)
  const isDragging = ref(false)

  function setLastPing(value: string) {
    lastPing.value = value
  }

  function setBounds(value: PetBounds) {
    bounds.value = value
  }

  function setDragging(value: boolean) {
    isDragging.value = value
  }

  return { lastPing, bounds, isDragging, setLastPing, setBounds, setDragging }
})
