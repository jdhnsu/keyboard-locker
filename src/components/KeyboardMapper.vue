<script setup lang="ts">
import { ref } from 'vue'
import type { KeyRule, KeyboardLayout } from '../types'
import KeyButton from './KeyButton.vue'

const props = defineProps<{
  rules: KeyRule[]
  locked: boolean
  layout?: KeyboardLayout
}>()

const emit = defineEmits<{
  toggleKey: [key: number]
}>()

const currentLayout = ref<KeyboardLayout>('ansi')

const VK = {
  ESC: 0x1B,
  F1: 0x70, F2: 0x71, F3: 0x72, F4: 0x73,
  F5: 0x74, F6: 0x75, F7: 0x76, F8: 0x77,
  F9: 0x78, F10: 0x79, F11: 0x7A, F12: 0x7B,
  BACKTICK: 0xC0,
  N1: 0x31, N2: 0x32, N3: 0x33, N4: 0x34, N5: 0x35,
  N6: 0x36, N7: 0x37, N8: 0x38, N9: 0x39, N0: 0x30,
  MINUS: 0xBD, EQUAL: 0xBB,
  BKSP: 0x08,
  TAB: 0x09,
  Q: 0x51, W: 0x57, E: 0x45, R: 0x52, T: 0x54,
  Y: 0x59, U: 0x55, I: 0x49, O: 0x4F, P: 0x50,
  LBRACKET: 0xDB, RBRACKET: 0xDD, BACKSLASH: 0xDC,
  CAPSLOCK: 0x14,
  A: 0x41, S: 0x53, D: 0x44, F: 0x46, G: 0x47,
  H: 0x48, J: 0x4A, K: 0x4B, L: 0x4C,
  SEMICOLON: 0xBA, QUOTE: 0xDE,
  ENTER: 0x0D,
  LSHIFT: 0xA0,
  Z: 0x5A, X: 0x58, C: 0x43, V: 0x56, B: 0x42,
  N: 0x4E, M: 0x4D,
  COMMA: 0xBC, DOT: 0xBE, SLASH: 0xBF,
  RSHIFT: 0xA1,
  LCTRL: 0xA2, LWIN: 0x5B, LALT: 0xA4,
  SPACE: 0x20,
  RALT: 0xA5, RWIN: 0x5C, MENU: 0x5D, RCTRL: 0xA3,
  INSERT: 0x2D, HOME: 0x24, PGUP: 0x21,
  DELETE: 0x2E, END: 0x23, PGDN: 0x22,
  UP: 0x26, DOWN: 0x28, LEFT: 0x25, RIGHT: 0x27,
} as const

const ansiRows: { label: string; code: number; width: string }[][] = [
  [
    { label: 'Esc', code: VK.ESC, width: '3rem' },
    { label: 'F1', code: VK.F1, width: '3rem' }, { label: 'F2', code: VK.F2, width: '3rem' },
    { label: 'F3', code: VK.F3, width: '3rem' }, { label: 'F4', code: VK.F4, width: '3rem' },
    { label: 'F5', code: VK.F5, width: '3rem' }, { label: 'F6', code: VK.F6, width: '3rem' },
    { label: 'F7', code: VK.F7, width: '3rem' }, { label: 'F8', code: VK.F8, width: '3rem' },
    { label: 'F9', code: VK.F9, width: '3rem' }, { label: 'F10', code: VK.F10, width: '3rem' },
    { label: 'F11', code: VK.F11, width: '3rem' }, { label: 'F12', code: VK.F12, width: '3rem' },
  ],
  [
    { label: '`', code: VK.BACKTICK, width: '3rem' }, { label: '1', code: VK.N1, width: '3rem' },
    { label: '2', code: VK.N2, width: '3rem' }, { label: '3', code: VK.N3, width: '3rem' },
    { label: '4', code: VK.N4, width: '3rem' }, { label: '5', code: VK.N5, width: '3rem' },
    { label: '6', code: VK.N6, width: '3rem' }, { label: '7', code: VK.N7, width: '3rem' },
    { label: '8', code: VK.N8, width: '3rem' }, { label: '9', code: VK.N9, width: '3rem' },
    { label: '0', code: VK.N0, width: '3rem' }, { label: '-', code: VK.MINUS, width: '3rem' },
    { label: '=', code: VK.EQUAL, width: '3rem' }, { label: 'Bksp', code: VK.BKSP, width: '5rem' },
  ],
  [
    { label: 'Tab', code: VK.TAB, width: '4.5rem' }, { label: 'Q', code: VK.Q, width: '3rem' },
    { label: 'W', code: VK.W, width: '3rem' }, { label: 'E', code: VK.E, width: '3rem' },
    { label: 'R', code: VK.R, width: '3rem' }, { label: 'T', code: VK.T, width: '3rem' },
    { label: 'Y', code: VK.Y, width: '3rem' }, { label: 'U', code: VK.U, width: '3rem' },
    { label: 'I', code: VK.I, width: '3rem' }, { label: 'O', code: VK.O, width: '3rem' },
    { label: 'P', code: VK.P, width: '3rem' }, { label: '[', code: VK.LBRACKET, width: '3rem' },
    { label: ']', code: VK.RBRACKET, width: '3rem' }, { label: '\\', code: VK.BACKSLASH, width: '4rem' },
  ],
  [
    { label: 'Caps', code: VK.CAPSLOCK, width: '5.5rem' }, { label: 'A', code: VK.A, width: '3rem' },
    { label: 'S', code: VK.S, width: '3rem' }, { label: 'D', code: VK.D, width: '3rem' },
    { label: 'F', code: VK.F, width: '3rem' }, { label: 'G', code: VK.G, width: '3rem' },
    { label: 'H', code: VK.H, width: '3rem' }, { label: 'J', code: VK.J, width: '3rem' },
    { label: 'K', code: VK.K, width: '3rem' }, { label: 'L', code: VK.L, width: '3rem' },
    { label: ';', code: VK.SEMICOLON, width: '3rem' }, { label: "'", code: VK.QUOTE, width: '3rem' },
    { label: 'Enter', code: VK.ENTER, width: '6rem' },
  ],
  [
    { label: 'Shift', code: VK.LSHIFT, width: '7rem' }, { label: 'Z', code: VK.Z, width: '3rem' },
    { label: 'X', code: VK.X, width: '3rem' }, { label: 'C', code: VK.C, width: '3rem' },
    { label: 'V', code: VK.V, width: '3rem' }, { label: 'B', code: VK.B, width: '3rem' },
    { label: 'N', code: VK.N, width: '3rem' }, { label: 'M', code: VK.M, width: '3rem' },
    { label: ',', code: VK.COMMA, width: '3rem' }, { label: '.', code: VK.DOT, width: '3rem' },
    { label: '/', code: VK.SLASH, width: '3rem' }, { label: 'Shift', code: VK.RSHIFT, width: '1fr' },
  ],
  [
    { label: 'Ctrl', code: VK.LCTRL, width: '4.5rem' },
    { label: 'Win', code: VK.LWIN, width: '3.5rem' },
    { label: 'Alt', code: VK.LALT, width: '3.5rem' },
    { label: '', code: VK.SPACE, width: '20rem' },
    { label: 'Alt', code: VK.RALT, width: '3.5rem' },
    { label: 'Win', code: VK.RWIN, width: '3.5rem' },
    { label: 'Menu', code: VK.MENU, width: '3.5rem' },
    { label: 'Ctrl', code: VK.RCTRL, width: '1fr' },
  ],
]

function isAllowed(code: number): boolean {
  const rule = props.rules.find(r => r.key === code)
  return rule ? rule.allowed : false
}

function handleToggle(code: number) {
  emit('toggleKey', code)
}
</script>

<template>
  <div class="bg-surface-container-lowest border border-outline-variant rounded-xl flex flex-col items-center justify-center shadow-sm overflow-x-auto w-full p-lg">
    <div class="w-full flex justify-between items-center max-w-[850px] flex-wrap mb-sm">
      <div>
        <h2 class="font-headline-lg text-headline-lg text-on-surface">可视化键盘映射</h2>
        <p class="font-body-md text-body-md text-on-surface-variant">
          {{ locked ? '锁定中 — 按键配置已锁定' : '点击按键切换其拦截状态，然后开启锁定' }}
        </p>
      </div>
      <div class="flex gap-md">
        <div class="flex items-center gap-xs mr-md border-r border-outline-variant/50 pr-md">
          <span class="font-label-md text-label-md text-on-surface-variant">键盘布局:</span>
          <select
            v-model="currentLayout"
            class="bg-surface-container border border-outline-variant rounded-sm text-label-md font-label-md px-sm py-xs focus:ring-1 focus:ring-primary outline-none cursor-pointer"
          >
            <option value="ansi">ANSI (US)</option>
            <option value="iso">ISO (UK/EU)</option>
          </select>
        </div>
        <div class="flex items-center gap-xs">
          <span class="w-3 h-3 rounded-full bg-green-600 shadow-sm border border-green-800"></span>
          <span class="font-label-md text-label-md text-on-surface-variant">允许通行</span>
        </div>
        <div class="flex items-center gap-xs">
          <span class="w-3 h-3 rounded-full bg-error shadow-sm border border-on-error-container"></span>
          <span class="font-label-md text-label-md text-on-surface-variant">拦截</span>
        </div>
      </div>
    </div>

    <div class="flex flex-col gap-[6px] bg-surface-container rounded-lg border border-outline-variant/50 min-w-max select-none shadow-inner p-sm">
      <div v-for="(row, ri) in ansiRows" :key="ri" class="flex gap-[6px]">
        <KeyButton
          v-for="key in row"
          :key="`${ri}-${key.code}`"
          :label="key.label"
          :code="key.code"
          :allowed="isAllowed(key.code)"
          :width="key.width === '1fr' ? 'auto' : key.width"
          :disabled="locked"
          :class="{ 'flex-1': key.width === '1fr' }"
          @toggle="handleToggle"
        />
      </div>
    </div>
  </div>
</template>