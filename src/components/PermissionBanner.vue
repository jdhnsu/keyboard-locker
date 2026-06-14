<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const permissionState = ref<'granted' | 'warning' | 'error' | 'hidden'>('hidden')
const permissionReason = ref('')

async function checkPermissions() {
  try {
    const result = await invoke<string>('check_permissions')
    if (result === 'granted') {
      permissionState.value = 'hidden'
    } else {
      permissionState.value = 'error'
      permissionReason.value = result
    }
  } catch {
    permissionState.value = 'hidden'
  }
}

async function openSettings() {
  await invoke('open_permission_settings')
}

onMounted(() => {
  checkPermissions()
})
</script>

<template>
  <div v-if="permissionState !== 'hidden'" :class="[
    'px-lg py-sm text-center font-label-md',
    permissionState === 'warning' ? 'bg-yellow-100 text-yellow-900 border-b border-yellow-300' : 'bg-red-100 text-red-900 border-b border-red-300'
  ]">
    <span>{{ permissionReason || '需要系统权限才能拦截键盘输入' }}</span>
    <button
      v-if="permissionState === 'error'"
      @click="openSettings"
      class="ml-sm underline font-bold hover:no-underline"
    >
      打开系统设置
    </button>
  </div>
</template>