<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  label: string
  code: number
  allowed: boolean
  width: string
  disabled?: boolean
}>()

const emit = defineEmits<{
  toggle: [code: number]
}>()

const btnClass = computed(() => {
  if (props.disabled) {
    return props.allowed
      ? 'bg-green-600/25 text-green-800 border-green-800/40 cursor-not-allowed'
      : 'bg-error/25 text-on-error/70 border-error-container/40 cursor-not-allowed'
  }
  return props.allowed
    ? 'bg-green-600 text-white border-green-800'
    : 'bg-error text-on-error border-error-container'
})

function handleClick() {
  if (!props.disabled) {
    emit('toggle', props.code)
  }
}
</script>

<template>
  <button
    :class="[btnClass]"
    :style="width ? { width } : {}"
    class="key-btn h-10 border-b-[3px] rounded-DEFAULT font-label-md text-sm flex items-center justify-center select-none transition-colors"
    :disabled="disabled"
    @click="handleClick"
  >
    {{ label }}
  </button>
</template>