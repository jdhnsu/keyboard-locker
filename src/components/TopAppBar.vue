<script setup lang="ts">
import { ref } from 'vue'

defineProps<{
  locked: boolean
}>()

const emit = defineEmits<{
  toggleLock: []
}>()

const processing = ref(false)

async function handleLock() {
  processing.value = true
  emit('toggleLock')
  setTimeout(() => { processing.value = false }, 300)
}
</script>

<template>
  <header class="flex justify-between items-center w-full px-lg max-w-full bg-surface border-b border-outline-variant top-0 z-40 sticky py-sm">
    <div class="flex items-center gap-sm md:hidden">
      <span class="material-symbols-outlined text-primary text-xl" style="font-variation-settings: 'FILL' 1">lock</span>
      <span class="font-headline-lg text-headline-lg font-bold text-primary">KeyLock Pro</span>
    </div>
    <div class="hidden md:block">
      <h2 class="font-headline-md text-headline-md font-bold text-on-surface">
        <span class="bg-clip-text text-transparent bg-gradient-to-r from-primary to-primary/80 font-bold tracking-tight">键盘锁定器</span>
      </h2>
    </div>
    <div class="flex items-center gap-md">
      <button
        @click="handleLock"
        :disabled="processing"
        :class="[
          'flex items-center justify-center px-md py-xs rounded-DEFAULT transition-colors font-label-md text-label-md gap-xs shadow-sm font-bold',
          locked
            ? 'bg-error text-on-error hover:bg-error/90'
            : 'bg-primary text-on-primary hover:bg-primary/90'
        ]"
      >
        <span class="material-symbols-outlined text-sm" :style="processing ? '' : 'font-variation-settings: \'FILL\' 1'">
          {{ processing ? 'hourglass_top' : (locked ? 'lock_open' : 'lock') }}
        </span>
        {{ locked ? '解锁' : '立即锁定' }}
      </button>
    </div>
  </header>
</template>