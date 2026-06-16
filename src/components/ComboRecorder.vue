<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { comboToLabel, keyboardCodeToVk } from '../utils/keyLabels'

const props = defineProps<{
  label: string
  combo: number[]
  commandName: string
}>()

const emit = defineEmits<{
  updated: [combo: number[]]
}>()

const recording = ref(false)
const recordedKeys = ref<number[]>([])
const pressedCodes = ref<Set<string>>(new Set())

function startRecording() {
  recording.value = true
  recordedKeys.value = []
  pressedCodes.value = new Set()
}

function stopRecording() {
  recording.value = false
  if (recordedKeys.value.length > 0) {
    invoke(props.commandName, { combo: recordedKeys.value })
      .then(() => {
        emit('updated', [...recordedKeys.value])
      })
      .catch(() => {})
  }
  recordedKeys.value = []
  pressedCodes.value.clear()
}

function handleKeyDown(e: KeyboardEvent) {
  if (!recording.value) return
  e.preventDefault()
  e.stopPropagation()

  const code = e.code
  if (!code || pressedCodes.value.has(code)) return

  const vk = keyboardCodeToVk(code)
  if (vk !== null && !recordedKeys.value.includes(vk)) {
    pressedCodes.value.add(code)
    recordedKeys.value.push(vk)
  }
}

function handleKeyUp(e: KeyboardEvent) {
  if (!recording.value) return
  e.preventDefault()
  e.stopPropagation()

  pressedCodes.value.delete(e.code)

  if (recordedKeys.value.length > 0 && pressedCodes.value.size === 0) {
    stopRecording()
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown, true)
  window.addEventListener('keyup', handleKeyUp, true)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown, true)
  window.removeEventListener('keyup', handleKeyUp, true)
})
</script>

<template>
  <div class="flex items-center justify-between gap-sm">
    <div class="flex flex-col gap-xs min-w-0 flex-1">
      <span class="font-label-md text-label-md text-on-surface-variant">{{ label }}</span>
      <div class="flex items-center gap-xs min-w-0">
        <template v-if="!recording && combo.length > 0">
          <template v-for="(vk, i) in combo" :key="i">
            <span
              class="inline-flex items-center justify-center px-xs py-[2px] rounded bg-surface-container-high text-on-surface font-mono font-label-lg text-label-lg whitespace-nowrap"
            >
              {{ comboToLabel([vk]) }}
            </span>
            <span v-if="i < combo.length - 1" class="text-on-surface-variant font-label-md text-label-md">+</span>
          </template>
        </template>
        <template v-else-if="recording && recordedKeys.length > 0">
          <template v-for="(vk, i) in recordedKeys" :key="i">
            <span
              class="inline-flex items-center justify-center px-xs py-[2px] rounded bg-primary-container text-on-primary-container font-mono font-label-lg text-label-lg whitespace-nowrap"
            >
              {{ comboToLabel([vk]) }}
            </span>
            <span v-if="i < recordedKeys.length - 1" class="text-on-surface-variant font-label-md text-label-md">+</span>
          </template>
          <span class="text-on-surface-variant font-label-md text-label-md animate-pulse">+</span>
        </template>
        <template v-else-if="recording">
          <span class="text-primary font-body-md animate-pulse">请按下组合键...</span>
        </template>
        <template v-else>
          <span class="text-on-surface-variant font-body-md">未设置</span>
        </template>
      </div>
    </div>
    <button
      @click="recording ? stopRecording() : startRecording()"
      :class="[
        'px-sm py-xs rounded-DEFAULT font-label-md text-label-md transition-colors shrink-0',
        recording
          ? 'bg-error-container text-on-error-container hover:bg-error-container/90'
          : 'bg-secondary-container text-on-secondary-container hover:bg-secondary-container/90'
      ]"
    >
      {{ recording ? '取消' : '录制' }}
    </button>
  </div>
</template>