<script setup lang="ts">
import { ref, onMounted } from 'vue'
import TitleBar from './components/TitleBar.vue'
import PermissionBanner from './components/PermissionBanner.vue'
import KeyboardMapper from './components/KeyboardMapper.vue'
import EngineToggle from './components/EngineToggle.vue'
import ComboRecorder from './components/ComboRecorder.vue'
import StatusBar from './components/StatusBar.vue'
import DeviceManager from './components/DeviceManager.vue'
import { useKeyboardState } from './composables/useKeyboardState'
import { getCurrentWindow } from '@tauri-apps/api/window'

const appWindow = getCurrentWindow()
onMounted(async () => {
  try {
    await appWindow.setBackgroundColor({ red: 248, green: 249, blue: 255, alpha: 255 })
  } catch {}
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

const view = ref<'main' | 'devices'>('main')

function showDevices() {
  view.value = 'devices'
}

function showMain() {
  view.value = 'main'
}

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
  <div class="h-screen flex flex-col bg-background text-on-background font-body-md antialiased rounded-xl border border-outline-variant/40 overflow-hidden shadow-[0_8px_40px_rgba(0,0,0,0.12),0_2px_16px_rgba(0,0,0,0.08),0_0_0_1px_rgba(0,0,0,0.06)]" style="isolation: isolate; backface-visibility: hidden">
    <template v-if="view === 'main'">
      <TitleBar
        :locked="status?.locked ?? false"
        @toggle-lock="handleToggleLock"
        @show-devices="showDevices"
      />
      <PermissionBanner class="shrink-0" />

      <main v-if="loading" class="flex-1 min-h-0 flex items-center justify-center">
        <p class="text-on-surface-variant font-body-lg">加载中...</p>
      </main>

      <main v-else-if="error" class="flex-1 min-h-0 flex flex-col items-center justify-center gap-md p-md">
        <p class="text-error font-body-lg">加载失败: {{ error }}</p>
        <button
          @click="refresh"
          class="px-md py-xs bg-primary text-on-primary rounded-DEFAULT font-label-md hover:bg-primary/90"
        >
          重试
        </button>
        <button
          @click="resetConfig"
          class="px-md py-xs bg-surface-container text-on-surface-variant border border-outline-variant rounded-DEFAULT font-label-md"
        >
          重置配置
        </button>
      </main>

      <main v-else class="flex-1 min-h-0 max-w-7xl mx-auto w-full flex flex-col p-md gap-md overflow-y-auto">
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
          <div class="bg-surface-container-lowest border border-outline-variant rounded-lg p-md flex flex-col gap-md shadow-sm">
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
    </template>

    <DeviceManager v-else @close="showMain" />
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
