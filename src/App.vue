<script setup lang="ts">
import { ref, onMounted } from 'vue'
import TitleBar from './components/TitleBar.vue'
import PermissionBanner from './components/PermissionBanner.vue'
import KeyboardMapper from './components/KeyboardMapper.vue'
import EngineToggle from './components/EngineToggle.vue'
import ComboRecorder from './components/ComboRecorder.vue'
import StatusBar from './components/StatusBar.vue'
import { useKeyboardState } from './composables/useKeyboardState'
import { getCurrentWindow } from '@tauri-apps/api/window'

const appWindow = getCurrentWindow()
const maximized = ref(false)

async function updateMaximized() {
  maximized.value = await appWindow.isMaximized()
}

onMounted(async () => {
  await updateMaximized()
  await appWindow.onResized(() => {
    updateMaximized()
  })
})

const {
  config,
  status,
  loading,
  error,
  toggleLock,
  setKeyAllowed,
  resetConfig,
  refresh,
  loadConfig,
} = useKeyboardState()

async function handleToggleKey(code: number) {
  if (!config.value) return
  if (status.value?.locked) return
  const rule = config.value.rules.find(r => r.key === code)
  const newAllowed = rule ? !rule.allowed : true
  await setKeyAllowed(code, newAllowed)
}

async function handleToggleLock() {
  await toggleLock()
}

async function handleUnlockComboUpdated() {
  await loadConfig()
}

async function handleLockComboUpdated() {
  await loadConfig()
}
</script>

<template>
  <!-- 外层：填充应用背景色，给 DWM 系统阴影足够可视空间 -->
  <div
    :class="['h-screen w-screen bg-surface text-on-surface font-body-md antialiased flex flex-col', maximized ? 'p-0' : 'p-1']"
    style="isolation: isolate; backface-visibility: hidden; will-change: transform"
  >
    <!-- 内层：应用窗口盒 — 仅靠背景色对比和真系统阴影，无任何边框/描边 -->
    <div
      :class="[
        'h-full w-full flex flex-col bg-surface-container-lowest overflow-hidden',
        maximized ? 'rounded-none' : 'rounded-2xl'
      ]"
    >
      <TitleBar
        :locked="status?.locked ?? false"
        @toggle-lock="handleToggleLock"
      />
      <PermissionBanner class="shrink-0" />

      <main v-if="loading" class="flex-1 min-h-0 flex items-center justify-center">
        <p class="text-on-surface-variant font-body-lg">加载中...</p>
      </main>

      <main v-else-if="error" class="flex-1 min-h-0 flex flex-col items-center justify-center gap-md p-md">
        <p class="text-error font-body-lg">加载失败: {{ error }}</p>
        <button
          @click="refresh"
          class="px-md py-xs bg-primary text-on-primary rounded-DEFAULT font-label-md hover:bg-primary/90 transition-colors duration-150"
        >
          重试
        </button>
        <button
          @click="resetConfig"
          class="px-md py-xs bg-surface-container text-on-surface-variant border border-outline-variant rounded-DEFAULT font-label-md hover:bg-surface-container-high transition-colors duration-150"
        >
          重置配置
        </button>
      </main>

      <main v-else class="flex-1 min-h-0 max-w-7xl mx-auto w-full flex flex-col p-md gap-md overflow-y-auto hide-scrollbar">
        <KeyboardMapper
          :rules="config?.rules ?? []"
          :locked="status?.locked ?? false"
          @toggle-key="handleToggleKey"
        />

        <section class="grid grid-cols-1 md:grid-cols-2 gap-md w-full">
          <EngineToggle
            :locked="status?.locked ?? false"
            :grab-active="status?.grab_active ?? false"
            @toggle="handleToggleLock"
          />
          <div class="bg-surface-container rounded-lg p-md flex flex-col gap-md">
            <div class="flex items-center gap-sm">
              <span class="material-symbols-outlined text-secondary">keyboard</span>
              <span class="font-headline-md text-headline-md text-on-surface">快捷键</span>
            </div>
            <div class="flex flex-col gap-sm">
              <ComboRecorder
                label="解锁快捷键"
                :combo="config?.unlock_combo ?? []"
                command-name="set_unlock_combo"
                @updated="handleUnlockComboUpdated"
              />
              <ComboRecorder
                label="锁定快捷键"
                :combo="config?.lock_combo ?? []"
                command-name="set_lock_combo"
                @updated="handleLockComboUpdated"
              />
            </div>
          </div>
        </section>

        <StatusBar />
      </main>
    </div>
  </div>
</template>

<style scoped>
.key-btn {
  transition: transform 0.1s, border-bottom-width 0.1s;
}
.key-btn:active {
  transform: translateY(2px);
  border-bottom-width: 2px !important;
}
</style>