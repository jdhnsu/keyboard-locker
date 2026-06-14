<script setup lang="ts">
import TopAppBar from './components/TopAppBar.vue'
import PermissionBanner from './components/PermissionBanner.vue'
import KeyboardMapper from './components/KeyboardMapper.vue'
import EngineToggle from './components/EngineToggle.vue'
import { useKeyboardState } from './composables/useKeyboardState'

const {
  config,
  status,
  loading,
  error,
  toggleLock,
  setKeyAllowed,
  resetConfig,
  refresh,
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
</script>

<template>
  <div class="min-h-screen flex flex-col bg-background text-on-background font-body-md antialiased">
    <TopAppBar
      :locked="status?.locked ?? false"
      @toggle-lock="handleToggleLock"
    />
    <PermissionBanner />

    <main v-if="loading" class="flex-1 flex items-center justify-center">
      <p class="text-on-surface-variant font-body-lg">加载中...</p>
    </main>

    <main v-else-if="error" class="flex-1 flex flex-col items-center justify-center gap-md p-md">
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

    <main v-else class="flex-1 max-w-7xl mx-auto w-full flex flex-col p-md gap-md">
      <KeyboardMapper
        :rules="config?.rules ?? []"
        :locked="status?.locked ?? false"
        @toggle-key="handleToggleKey"
      />

      <section class="w-full">
        <EngineToggle
          :locked="status?.locked ?? false"
          :grab-active="status?.grab_active ?? false"
          @toggle="handleToggleLock"
        />
      </section>

      <div v-if="status" class="flex gap-md justify-center text-label-sm text-on-surface-variant">
        <span>拦截: {{ status.total_blocked }}</span>
        <span>放行: {{ status.total_allowed }}</span>
      </div>
    </main>
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