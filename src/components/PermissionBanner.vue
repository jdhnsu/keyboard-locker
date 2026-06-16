<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

interface PermissionResult {
  status: string
  reason?: string
  can_auto_fix?: boolean
}

interface GrabEvent {
  error?: string
  warning?: string
  detail?: string
  errors?: string[]
}

const bannerState = ref<'hidden' | 'denied' | 'fixing' | 'grab-error'>('hidden')
const bannerMessage = ref('')
const autoFixAttempted = ref(false)
let unlistenGrabError: UnlistenFn | null = null
let unlistenGrabWarning: UnlistenFn | null = null

async function checkPermissions(): Promise<PermissionResult | null> {
  try {
    return await invoke<PermissionResult>('check_permissions')
  } catch {
    return null
  }
}

async function fixPermissions() {
  bannerState.value = 'fixing'
  try {
    const result = await invoke<PermissionResult>('fix_permissions')
    if (result.status === 'granted') {
      bannerState.value = 'hidden'
      await restartGrab()
    } else {
      bannerState.value = 'denied'
      bannerMessage.value = result.reason || '需要系统权限才能拦截键盘输入'
      autoFixAttempted.value = true
    }
  } catch {
    bannerState.value = 'denied'
    bannerMessage.value = '权限修复失败，请手动尝试'
  }
}

async function restartGrab() {
  try {
    await invoke('restart_grab')
  } catch {}
}

async function handleRetry() {
  await fixPermissions()
}

async function handleRestartGrab() {
  await restartGrab()
  bannerState.value = 'hidden'
}

onMounted(async () => {
  try {
    unlistenGrabError = await listen<GrabEvent>('grab-error', (event) => {
      bannerState.value = 'grab-error'
      bannerMessage.value = event.payload.error || '键盘拦截引擎启动失败'
    })

    unlistenGrabWarning = await listen<GrabEvent>('grab-warning', (event) => {
      bannerState.value = 'grab-error'
      bannerMessage.value = event.payload.warning || '键盘拦截可能无法正常工作'
    })
  } catch {}

  const result = await checkPermissions()
  if (!result || result.status === 'granted') {
    bannerState.value = 'hidden'
    return
  }

  bannerMessage.value = result.reason || '需要系统权限才能拦截键盘输入'

  if (result.can_auto_fix && !autoFixAttempted.value) {
    await fixPermissions()
  } else {
    bannerState.value = 'denied'
  }
})

onUnmounted(() => {
  unlistenGrabError?.()
  unlistenGrabWarning?.()
})
</script>

<template>
  <div v-if="bannerState !== 'hidden'" :class="[
    'px-lg py-sm text-center font-label-md shrink-0 flex items-center justify-center gap-sm',
    bannerState === 'fixing'
      ? 'bg-yellow-100 text-yellow-900 border-b border-yellow-300'
      : 'bg-red-100 text-red-900 border-b border-red-300'
  ]">
    <span v-if="bannerState === 'fixing'">正在修复权限...</span>
    <span v-else>{{ bannerMessage || '需要系统权限才能拦截键盘输入' }}</span>
    <button
      v-if="bannerState === 'denied'"
      @click="handleRetry"
      class="underline font-bold hover:no-underline"
    >
      一键修复
    </button>
    <button
      v-if="bannerState === 'grab-error'"
      @click="handleRestartGrab"
      class="underline font-bold hover:no-underline"
    >
      重试拦截
    </button>
  </div>
</template>
