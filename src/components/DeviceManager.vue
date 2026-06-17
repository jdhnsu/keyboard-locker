<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { KeyboardDeviceInfo, KeyboardDeviceConfig } from '../types'

const emit = defineEmits<{
  close: []
}>()

const devices = ref<KeyboardDeviceInfo[]>([])
const loading = ref(true)
const identifying = ref(false)
const highlightedId = ref<string | null>(null)
const error = ref<string | null>(null)
let unlistenTapped: UnlistenFn | null = null

async function loadDevices() {
  try {
    loading.value = true
    error.value = null
    devices.value = await invoke<KeyboardDeviceInfo[]>('enumerate_keyboards')
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

async function toggleIdentify() {
  if (identifying.value) {
    await stopIdentify()
  } else {
    await startIdentify()
  }
}

async function startIdentify() {
  try {
    await invoke('identify_keyboard_start')
    identifying.value = true
    error.value = null

    unlistenTapped?.()
    unlistenTapped = await listen<{ instance_id: string }>('keyboard-tapped', (event) => {
      handleTapped(event.payload)
    })
  } catch (e) {
    error.value = String(e)
  }
}

async function stopIdentify() {
  try {
    unlistenTapped?.()
    unlistenTapped = null
    await invoke('identify_keyboard_stop')
  } catch { }
  identifying.value = false
  highlightedId.value = null
}

function handleTapped(payload: { instance_id: string }) {
  highlightedId.value = payload.instance_id

  const idx = devices.value.findIndex(d => d.instance_id === payload.instance_id)
  if (idx >= 0) {
    const el = document.getElementById(`kbd-card-${payload.instance_id}`)
    el?.querySelector<HTMLInputElement>('.alias-input')?.focus()
  }

  setTimeout(() => {
    if (highlightedId.value === payload.instance_id) {
      highlightedId.value = null
    }
  }, 2000)
}

async function handleSave(device: KeyboardDeviceInfo) {
  try {
    const config: KeyboardDeviceConfig = {
      instance_id: device.instance_id,
      alias: device.alias,
      enabled: device.enabled,
      is_target: device.is_target,
    }
    await invoke('update_keyboard_device', { device: config })
    error.value = null
  } catch (e) {
    error.value = String(e)
  }
}

function vendorHex(id: number): string {
  return '0x' + id.toString(16).toUpperCase().padStart(4, '0')
}

onMounted(async () => {
  await loadDevices()
})

onUnmounted(async () => {
  if (identifying.value) {
    unlistenTapped?.()
    await invoke('identify_keyboard_stop').catch(() => { })
  }
  unlistenTapped?.()
})
</script>

<template>
  <div class="flex flex-col h-full">
    <header class="flex items-center gap-sm px-md py-sm border-b border-outline-variant shrink-0">
      <button
        class="flex items-center gap-xs text-on-surface-variant hover:text-on-surface transition-colors font-label-md"
        @click="$emit('close')"
      >
        <span class="material-symbols-outlined text-base">arrow_back</span>
        返回
      </button>
      <span class="font-headline-md text-headline-md text-on-surface">设备管理</span>
    </header>

    <div v-if="error" class="px-md py-xs bg-error-container text-on-error-container font-body-md">
      {{ error }}
    </div>

    <div class="flex-1 overflow-y-auto p-md flex flex-col gap-md">
      <div class="bg-surface-container-lowest border border-outline-variant rounded-lg p-md flex items-center justify-between gap-md">
        <div class="flex items-center gap-sm text-on-surface-variant font-body-md">
          <span class="material-symbols-outlined text-base">touch_app</span>
          <span>敲击任意键盘的按键来识别对应设备</span>
        </div>
        <button
          @click="toggleIdentify"
          :class="[
            'flex items-center gap-xs px-md py-xs rounded-DEFAULT font-label-md transition-colors shrink-0',
            identifying
              ? 'bg-error text-on-error hover:bg-error/90'
              : 'bg-primary text-on-primary hover:bg-primary/90'
          ]"
        >
          <span class="material-symbols-outlined text-sm">
            {{ identifying ? 'stop' : 'radar' }}
          </span>
          {{ identifying ? '停止识别' : '开始识别' }}
        </button>
      </div>

      <div v-if="loading" class="flex items-center justify-center py-xl text-on-surface-variant font-body-lg">
        加载中...
      </div>

      <template v-else-if="devices.length === 0">
        <div class="flex flex-col items-center justify-center py-xl gap-sm text-on-surface-variant">
          <span class="material-symbols-outlined text-4xl">keyboard</span>
          <p class="font-body-lg">未检测到键盘设备</p>
        </div>
      </template>

      <div
        v-for="device in devices"
        :id="`kbd-card-${device.instance_id}`"
        :key="device.instance_id"
        :class="[
          'bg-surface-container-lowest border rounded-lg p-md flex flex-col gap-sm transition-all duration-300',
          highlightedId === device.instance_id
            ? 'border-green-500 shadow-[0_0_20px_rgba(34,197,94,0.5)]'
            : 'border-outline-variant'
        ]"
      >
        <div class="flex items-start justify-between">
          <div class="flex items-center gap-sm">
            <span class="material-symbols-outlined text-primary">keyboard</span>
            <span class="font-label-md text-label-md text-on-surface font-semibold">
              {{ device.name }}
            </span>
          </div>
          <div
            v-if="highlightedId === device.instance_id"
            class="px-xs py-0 rounded-full bg-green-500 text-on-success text-xs font-bold animate-pulse"
          >
            IDENTIFIED
          </div>
        </div>

        <div class="flex gap-md text-label-sm text-on-surface-variant font-mono">
          <span>VID {{ vendorHex(device.vendor_id) }}</span>
          <span>PID {{ vendorHex(device.product_id) }}</span>
        </div>

        <div class="flex items-center gap-sm mt-xs">
          <span class="text-label-sm text-on-surface-variant shrink-0">别名</span>
          <input
            class="alias-input flex-1 px-sm py-xs bg-surface border border-outline-variant rounded-DEFAULT text-body-md text-on-surface placeholder:text-on-surface-variant/50 outline-none focus:border-primary transition-colors"
            placeholder="输入设备别名..."
            :value="device.alias"
            @input="device.alias = ($event.target as HTMLInputElement).value"
          />
          <button
            @click="handleSave(device)"
            class="px-sm py-xs bg-primary text-on-primary rounded-DEFAULT font-label-sm hover:bg-primary/90 transition-colors"
          >
            保存
          </button>
        </div>

        <div class="flex items-center gap-lg mt-xs">
          <label class="flex items-center gap-xs cursor-pointer select-none">
            <input
              type="checkbox"
              :checked="device.enabled"
              @change="device.enabled = ($event.target as HTMLInputElement).checked; handleSave(device)"
              class="w-4 h-4 accent-primary"
            />
            <span class="font-label-sm text-on-surface-variant">参与锁定</span>
          </label>
          <label class="flex items-center gap-xs cursor-pointer select-none">
            <input
              type="checkbox"
              :checked="device.is_target"
              @change="device.is_target = ($event.target as HTMLInputElement).checked; handleSave(device)"
              class="w-4 h-4 accent-secondary"
            />
            <span class="font-label-sm text-on-surface-variant">目标键盘</span>
          </label>
        </div>
      </div>
    </div>
  </div>
</template>
