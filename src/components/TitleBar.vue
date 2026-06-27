<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'

defineProps<{
  locked: boolean
}>()

const emit = defineEmits<{
  toggleLock: []
}>()

const processing = ref(false)
const focused = ref(true)
const appWindow = getCurrentWindow()

async function updateMaximized() {
  // App.vue 监听 maximized 状态变化，通过组件引用 / 提供注入同步
  // 这里仅触发本组件内部最大化图标更新
  // maximized 状态通过 onResized 反馈给 App.vue
}

onMounted(async () => {
  await appWindow.isMaximized()
  const unlistenResize = await appWindow.onResized(() => {
    // 由 App.vue 通过 getCurrentWindow() 主动获取最新状态
  })
  const unlistenFocus = await appWindow.onFocusChanged(({ payload }) => {
    focused.value = payload
  })
  onUnmounted(() => {
    unlistenResize()
    unlistenFocus()
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
    :class="[
      'flex items-center justify-between h-8 min-h-8 px-sm select-none shrink-0',
      'bg-surface-container-lowest',
      'transition-opacity duration-200',
      focused ? 'opacity-100' : 'opacity-70'
    ]"
    style="box-shadow: inset 0 -1px 0 rgba(0, 0, 0, 0.04)"
  >
    <div class="flex items-center gap-xs pl-xs">
      <span class="material-symbols-outlined text-primary text-base" style="font-variation-settings: 'FILL' 1">lock</span>
      <span class="font-label-md text-label-md text-on-surface font-bold">KeyLock Pro</span>
    </div>

    <div class="flex items-center gap-xs">
      <button
        @click="handleLock"
        :disabled="processing"
        :class="[
          'flex items-center justify-center px-sm h-5 rounded-DEFAULT font-label-sm text-label-sm gap-xs font-semibold',
          'transition-colors duration-150 ease-out',
          locked
            ? 'bg-error text-on-error hover:bg-error/90 active:bg-error/80'
            : 'bg-primary text-on-primary hover:bg-primary/90 active:bg-primary/80'
        ]"
      >
        <span class="material-symbols-outlined text-xs" :style="processing ? '' : 'font-variation-settings: \'FILL\' 1'">
          {{ processing ? 'hourglass_top' : (locked ? 'lock_open' : 'lock') }}
        </span>
        {{ locked ? '解锁' : '锁定' }}
      </button>

      <div class="w-px h-4 bg-outline-variant mx-xs" />

      <button
        class="flex items-center justify-center w-8 h-6 rounded-DEFAULT hover:bg-surface-container-high active:bg-surface-container-highest text-on-surface-variant hover:text-on-surface transition-colors duration-150 ease-out"
        @click="appWindow.minimize()"
      >
        <span class="material-symbols-outlined text-xs">remove</span>
      </button>
      <button
        id="titlebar-maximize"
        class="flex items-center justify-center w-8 h-6 rounded-DEFAULT hover:bg-surface-container-high active:bg-surface-container-highest text-on-surface-variant hover:text-on-surface transition-colors duration-150 ease-out"
        @click="appWindow.toggleMaximize()"
      >
        <span class="material-symbols-outlined text-xs">crop_square</span>
      </button>
      <button
        class="flex items-center justify-center w-8 h-6 rounded-DEFAULT hover:bg-error hover:text-on-error active:bg-error/90 text-on-surface-variant transition-colors duration-150 ease-out"
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

button {
  transition: background-color 120ms ease-out, color 120ms ease-out, transform 80ms ease-out;
}
button:active {
  transform: translateY(0.5px);
}
</style>