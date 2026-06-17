<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { EngineSnapshot } from '../types'

const status = ref<Pick<EngineSnapshot, 'total_blocked' | 'total_allowed'> | null>(null)
let unlisten: UnlistenFn | null = null

async function loadCounts() {
  try {
    const s = await invoke<EngineSnapshot>('get_status')
    status.value = { total_blocked: s.total_blocked, total_allowed: s.total_allowed }
  } catch {}
}

onMounted(async () => {
  await loadCounts()
  try {
    unlisten = await listen('lock-state-changed', () => {
      loadCounts()
    })
  } catch {}
  onUnmounted(() => {
    unlisten?.()
  })
})
</script>

<template>
  <div v-if="status" class="flex gap-md justify-center text-label-sm text-on-surface-variant">
    <span>拦截: {{ status.total_blocked }}</span>
    <span>放行: {{ status.total_allowed }}</span>
  </div>
</template>
