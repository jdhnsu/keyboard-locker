<script setup lang="ts">
defineProps<{
  mode: 'whitelist' | 'blacklist'
}>()

defineEmits<{
  toggle: []
}>()
</script>

<template>
  <div class="bg-surface-container-lowest border border-outline-variant rounded-lg p-md flex flex-col justify-between gap-md shadow-sm">
    <div class="flex items-center gap-sm">
      <span class="material-symbols-outlined text-primary">filter_alt</span>
      <span class="font-headline-md text-headline-md text-on-surface text-base">运行模式</span>
    </div>
    <div class="flex bg-surface-container rounded-DEFAULT p-xs">
      <button
        :class="[
          'flex-1 py-sm px-sm text-center rounded-sm font-label-md font-bold transition-colors',
          mode === 'whitelist'
            ? 'bg-surface-container-lowest text-primary shadow-sm'
            : 'text-on-surface-variant hover:text-on-surface'
        ]"
        @click="$emit('toggle')"
      >
        白名单
      </button>
      <button
        :class="[
          'flex-1 py-sm px-sm text-center rounded-sm font-label-md font-bold transition-colors',
          mode === 'blacklist'
            ? 'bg-surface-container-lowest text-primary shadow-sm'
            : 'text-on-surface-variant hover:text-on-surface'
        ]"
        @click="$emit('toggle')"
      >
        黑名单
      </button>
    </div>
  </div>
</template>