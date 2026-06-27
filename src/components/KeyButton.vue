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

// 状态层次：默认态 (subtle) / 允许态 (蓝调) / 拦截态 (红调)
const stateClass = computed(() => {
  if (props.disabled) {
    return props.allowed
      ? 'bg-primary-container/40 text-on-primary-container/70 cursor-not-allowed'
      : 'bg-error-container/50 text-on-error-container/70 cursor-not-allowed'
  }
  return props.allowed
    ? 'bg-primary-container text-on-primary-container hover:bg-primary-fixed hover:shadow-sm'
    : 'bg-error-container text-on-error-container hover:brightness-95 hover:shadow-sm'
})

function handleClick() {
  if (!props.disabled) {
    emit('toggle', props.code)
  }
}
</script>

<template>
  <button
    :class="[
      'key-btn',
      'h-10 px-2',
      'rounded-md',
      'font-label-md text-sm',
      'flex items-center justify-center',
      'select-none',
      'border border-transparent',
      'transition-all duration-150 ease-out',
      'active:translate-y-px active:scale-[0.97]',
      stateClass,
    ]"
    :style="width ? { width } : {}"
    :disabled="disabled"
    @click="handleClick"
  >
    {{ label }}
  </button>
</template>