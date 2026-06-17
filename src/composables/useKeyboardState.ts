import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { Config, EngineSnapshot } from '../types'

export function useKeyboardState() {
  const config = ref<Config | null>(null)
  const status = ref<EngineSnapshot | null>(null)
  const loading = ref(true)
  const error = ref<string | null>(null)
  let unlisten: UnlistenFn | null = null
  let unlistenFocus: UnlistenFn | null = null

  async function loadConfig() {
    try {
      const c = await invoke<Config>('get_config')
      config.value = c
    } catch (e) {
      error.value = String(e)
    }
  }

  async function loadStatus() {
    try {
      const s = await invoke<EngineSnapshot>('get_status')
      status.value = s
    } catch (e) {
      error.value = String(e)
    }
  }

  async function toggleLock() {
    try {
      const locked = await invoke<boolean>('toggle_lock')
      await loadStatus()
      return locked
    } catch (e) {
      error.value = String(e)
      return null
    }
  }

  async function setKeyAllowed(key: number, allowed: boolean) {
    try {
      await invoke('set_key_allowed', { key, allowed })
      await loadConfig()
    } catch (e) {
      error.value = String(e)
    }
  }

  async function updateConfig(newConfig: Config) {
    try {
      await invoke('update_config', { config: newConfig })
      config.value = newConfig
    } catch (e) {
      error.value = String(e)
    }
  }

  async function resetConfig() {
    try {
      const c = await invoke<Config>('reset_config')
      config.value = c
    } catch (e) {
      error.value = String(e)
    }
  }

  async function refresh() {
    loading.value = true
    error.value = null
    await Promise.all([loadConfig(), loadStatus()])
    loading.value = false
  }

  onMounted(async () => {
    await refresh()
    try {
      unlisten = await listen('lock-state-changed', () => {
        loadStatus()
      })
    } catch {}
    try {
      unlistenFocus = await listen('tauri://focus', () => {
        loadStatus()
        loadConfig()
      })
    } catch {}
  })

  onUnmounted(() => {
    unlisten?.()
    unlistenFocus?.()
  })

  return {
    config,
    status,
    loading,
    error,
    toggleLock,
    setKeyAllowed,
    updateConfig,
    resetConfig,
    refresh,
    loadConfig,
  }
}