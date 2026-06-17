<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'

defineProps<{
  locked: boolean
}>()

const emit = defineEmits<{
  toggleLock: []
  showDevices: []
}>()

const processing = ref(false)
const maximized = ref(false)
const appWindow = getCurrentWindow()

async function updateMaximized() {
  maximized.value = await appWindow.isMaximized()
}

onMounted(async () => {
  await updateMaximized()
  const unlisten = await appWindow.onResized(() => {
    updateMaximized()
  })
  onUnmounted(() => {
    unlisten()
  })
})

async function handleLock() {
  processing.value = true
  emit('toggleLock')
  setTimeout(() => { processing.value = false }, 300)
}
</script>

<template>
  <header
    data-tauri-drag-region
    class="flex items-center justify-between h-8 px-sm select-none bg-surface border-b border-outline-variant shrink-0"
  >
    <div class="flex items-center gap-xs pl-xs">
      <span class="material-symbols-outlined text-primary text-base" style="font-variation-settings: 'FILL' 1">lock</span>
      <span class="font-label-md text-label-md text-on-surface font-bold">KeyLock Pro</span>
    </div>

    <div class="flex items-center gap-xs">
      <button
        class="flex items-center justify-center w-8 h-6 rounded-DEFAULT hover:bg-surface-container-high text-on-surface-variant hover:text-on-surface transition-colors"
        @click="$emit('showDevices')"
        title="设备管理"
      >
        <span class="material-symbols-outlined text-sm">settings</span>
      </button>

      <button
        @click="handleLock"
        :disabled="processing"
        :class="[
          'flex items-center justify-center px-sm h-5 rounded-DEFAULT transition-colors font-label-sm text-label-sm gap-xs font-semibold',
          locked
            ? 'bg-error text-on-error hover:bg-error/90'
            : 'bg-primary text-on-primary hover:bg-primary/90'
        ]"
      >
        <span class="material-symbols-outlined text-xs" :style="processing ? '' : 'font-variation-settings: \'FILL\' 1'">
          {{ processing ? 'hourglass_top' : (locked ? 'lock_open' : 'lock') }}
        </span>
        {{ locked ? '解锁' : '锁定' }}
      </button>

      <div class="w-px h-4 bg-outline-variant mx-xs" />

      <button
        class="flex items-center justify-center w-8 h-6 rounded-DEFAULT hover:bg-surface-container-high text-on-surface-variant hover:text-on-surface transition-colors"
        @click="appWindow.minimize()"
      >
        <span class="material-symbols-outlined text-xs">remove</span>
      </button>
      <button
        class="flex items-center justify-center w-8 h-6 rounded-DEFAULT hover:bg-surface-container-high text-on-surface-variant hover:text-on-surface transition-colors"
        @click="appWindow.toggleMaximize()"
      >
        <span v-if="maximized" class="material-symbols-outlined text-xs">filter_none</span>
        <span v-else class="material-symbols-outlined text-xs">crop_square</span>
      </button>
      <button
        class="flex items-center justify-center w-8 h-6 rounded-DEFAULT hover:bg-error hover:text-on-error text-on-surface-variant transition-colors"
        @click="appWindow.close()"
      >
        <span class="material-symbols-outlined text-xs">close</span>
      </button>
    </div>
  </header>
</template>

<style scoped>
header[data-tauri-drag-region] {
  will-change: transform;
  isolation: isolate;
}
</style>
