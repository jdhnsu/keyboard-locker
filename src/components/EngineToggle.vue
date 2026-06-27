<script setup lang="ts">
defineProps<{
  locked: boolean
  grabActive: boolean
}>()

defineEmits<{
  toggle: []
}>()
</script>

<template>
  <div class="bg-surface-container rounded-lg p-md flex flex-col justify-between gap-md">
    <div class="flex items-center gap-sm">
      <span class="material-symbols-outlined text-primary">security</span>
      <span class="font-headline-md text-headline-md text-on-surface text-base">锁定引擎</span>
    </div>
    <div
      class="w-full p-sm bg-surface-container-high rounded-lg flex items-center justify-between cursor-pointer hover:bg-surface-container-highest transition-colors h-[42px]"
      @click="$emit('toggle')"
    >
      <span class="font-label-md text-label-md font-bold ml-xs" :class="locked ? 'text-on-surface' : 'text-on-surface-variant'">
        {{ locked ? '已锁定' : '已解锁' }}
      </span>
      <div
        :class="[
          'relative inline-flex h-8 w-14 items-center rounded-full transition-colors',
          locked ? 'bg-primary' : 'bg-outline-variant'
        ]"
      >
        <span
          :class="[
            'inline-block h-6 w-6 transform rounded-full bg-on-primary transition-transform shadow-sm',
            locked ? 'translate-x-7' : 'translate-x-1'
          ]"
        ></span>
      </div>
    </div>
  </div>
</template>