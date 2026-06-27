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
  NUM0: 0x60, NUM1: 0x61, NUM2: 0x62, NUM3: 0x63, NUM4: 0x64,
  NUM5: 0x65, NUM6: 0x66, NUM7: 0x67, NUM8: 0x68, NUM9: 0x69,
  NUMMUL: 0x6A, NUMADD: 0x6B,
  NUMSUB: 0x6D, NUMDOT: 0x6E, NUMDIV: 0x6F,
  NUMLK: 0x90,
  SNAPSHOT: 0x2C, SCROLL: 0x91, PAUSE: 0x13,
} as const

// ---------- Key width constants (CSS values) ----------
const W_STD = '2.5rem'
const W_TAB = '3.75rem'
const W_CAPS = '4.75rem'
const W_LSHIFT = '6rem'
const W_BKSP = '4.5rem'
const W_ENTER = '5.25rem'
const W_LCTRL = '3.75rem'
const W_MOD = '3rem'         // Win / Alt / Menu / Backslash
const W_FSPACER_BIG = '2.5rem'
const W_FSPACER_SM = '1.5rem'

function isAllowed(code: number): boolean {
  const rule = props.rules.find(r => r.key === code)
  return rule ? rule.allowed : false
}

function handleToggle(code: number) {
  emit('toggleKey', code)
}
</script>

<template>
  <div class="bg-surface-container rounded-2xl flex flex-col items-center p-md select-none overflow-x-auto">
    <!-- Header: title + legend + layout selector -->
    <div class="w-full flex justify-between items-center flex-wrap gap-sm mb-sm">
      <div>
        <h2 class="font-headline-lg text-headline-lg text-on-surface">可视化键盘映射</h2>
        <p class="font-body-md text-body-md text-on-surface-variant">
          {{ locked ? '锁定中 — 按键配置已锁定' : '点击按键切换其拦截状态，然后开启锁定' }}
        </p>
      </div>
      <div class="flex gap-md items-center flex-wrap">
        <div class="flex items-center gap-xs mr-md border-r border-outline-variant/50 pr-md">
          <span class="font-label-md text-label-md text-on-surface-variant">键盘布局:</span>
          <select
            v-model="currentLayout"
            class="bg-surface-container-high border border-outline-variant/30 rounded-md text-label-md font-label-md px-sm py-xs focus:ring-1 focus:ring-primary outline-none cursor-pointer hover:bg-surface-container-highest transition-colors duration-150"
          >
            <option value="ansi">ANSI (US)</option>
            <option value="iso">ISO (UK/EU)</option>
          </select>
        </div>
        <div class="flex items-center gap-xs">
          <span class="w-3 h-3 rounded-full bg-primary-container border border-on-primary-container/40"></span>
          <span class="font-label-md text-label-md text-on-surface-variant">允许通行</span>
        </div>
        <div class="flex items-center gap-xs">
          <span class="w-3 h-3 rounded-full bg-error-container border border-on-error-container/40"></span>
          <span class="font-label-md text-label-md text-on-surface-variant">拦截</span>
        </div>
      </div>
    </div>

    <!-- ========== 键盘整体容器 ========== -->
    <div class="bg-surface-container-lowest rounded-xl shadow-inner p-md min-w-max">
      <div class="flex gap-4">

        <!-- ========== 主键盘区 (左) ========== -->
        <div class="flex flex-col gap-1">
          <!-- 功能键行 -->
          <div class="flex gap-1 mb-1">
            <KeyButton label="Esc" :code="VK.ESC" :allowed="isAllowed(VK.ESC)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <span class="flex-shrink-0" :style="{ width: W_FSPACER_BIG }" />
            <KeyButton label="F1" :code="VK.F1" :allowed="isAllowed(VK.F1)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="F2" :code="VK.F2" :allowed="isAllowed(VK.F2)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="F3" :code="VK.F3" :allowed="isAllowed(VK.F3)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="F4" :code="VK.F4" :allowed="isAllowed(VK.F4)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <span class="flex-shrink-0" :style="{ width: W_FSPACER_SM }" />
            <KeyButton label="F5" :code="VK.F5" :allowed="isAllowed(VK.F5)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="F6" :code="VK.F6" :allowed="isAllowed(VK.F6)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="F7" :code="VK.F7" :allowed="isAllowed(VK.F7)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="F8" :code="VK.F8" :allowed="isAllowed(VK.F8)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <span class="flex-shrink-0" :style="{ width: W_FSPACER_SM }" />
            <KeyButton label="F9" :code="VK.F9" :allowed="isAllowed(VK.F9)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="F10" :code="VK.F10" :allowed="isAllowed(VK.F10)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="F11" :code="VK.F11" :allowed="isAllowed(VK.F11)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="F12" :code="VK.F12" :allowed="isAllowed(VK.F12)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
          </div>

          <!-- 数字键行 -->
          <div class="flex gap-1">
            <KeyButton label="`" :code="VK.BACKTICK" :allowed="isAllowed(VK.BACKTICK)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="1" :code="VK.N1" :allowed="isAllowed(VK.N1)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="2" :code="VK.N2" :allowed="isAllowed(VK.N2)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="3" :code="VK.N3" :allowed="isAllowed(VK.N3)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="4" :code="VK.N4" :allowed="isAllowed(VK.N4)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="5" :code="VK.N5" :allowed="isAllowed(VK.N5)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="6" :code="VK.N6" :allowed="isAllowed(VK.N6)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="7" :code="VK.N7" :allowed="isAllowed(VK.N7)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="8" :code="VK.N8" :allowed="isAllowed(VK.N8)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="9" :code="VK.N9" :allowed="isAllowed(VK.N9)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="0" :code="VK.N0" :allowed="isAllowed(VK.N0)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="-" :code="VK.MINUS" :allowed="isAllowed(VK.MINUS)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="=" :code="VK.EQUAL" :allowed="isAllowed(VK.EQUAL)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="Bksp" :code="VK.BKSP" :allowed="isAllowed(VK.BKSP)" :width="W_BKSP" :disabled="locked" @toggle="handleToggle" />
          </div>

          <!-- Tab 行 -->
          <div class="flex gap-1">
            <KeyButton label="Tab" :code="VK.TAB" :allowed="isAllowed(VK.TAB)" :width="W_TAB" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="Q" :code="VK.Q" :allowed="isAllowed(VK.Q)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="W" :code="VK.W" :allowed="isAllowed(VK.W)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="E" :code="VK.E" :allowed="isAllowed(VK.E)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="R" :code="VK.R" :allowed="isAllowed(VK.R)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="T" :code="VK.T" :allowed="isAllowed(VK.T)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="Y" :code="VK.Y" :allowed="isAllowed(VK.Y)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="U" :code="VK.U" :allowed="isAllowed(VK.U)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="I" :code="VK.I" :allowed="isAllowed(VK.I)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="O" :code="VK.O" :allowed="isAllowed(VK.O)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="P" :code="VK.P" :allowed="isAllowed(VK.P)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="[" :code="VK.LBRACKET" :allowed="isAllowed(VK.LBRACKET)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="]" :code="VK.RBRACKET" :allowed="isAllowed(VK.RBRACKET)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="\" :code="VK.BACKSLASH" :allowed="isAllowed(VK.BACKSLASH)" :width="W_MOD" :disabled="locked" @toggle="handleToggle" />
          </div>

          <!-- Caps 行 -->
          <div class="flex gap-1">
            <KeyButton label="Caps" :code="VK.CAPSLOCK" :allowed="isAllowed(VK.CAPSLOCK)" :width="W_CAPS" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="A" :code="VK.A" :allowed="isAllowed(VK.A)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="S" :code="VK.S" :allowed="isAllowed(VK.S)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="D" :code="VK.D" :allowed="isAllowed(VK.D)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="F" :code="VK.F" :allowed="isAllowed(VK.F)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="G" :code="VK.G" :allowed="isAllowed(VK.G)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="H" :code="VK.H" :allowed="isAllowed(VK.H)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="J" :code="VK.J" :allowed="isAllowed(VK.J)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="K" :code="VK.K" :allowed="isAllowed(VK.K)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="L" :code="VK.L" :allowed="isAllowed(VK.L)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label=";" :code="VK.SEMICOLON" :allowed="isAllowed(VK.SEMICOLON)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="'" :code="VK.QUOTE" :allowed="isAllowed(VK.QUOTE)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="Enter" :code="VK.ENTER" :allowed="isAllowed(VK.ENTER)" :width="W_ENTER" :disabled="locked" @toggle="handleToggle" />
          </div>

          <!-- Shift 行 -->
          <div class="flex gap-1">
            <KeyButton label="Shift" :code="VK.LSHIFT" :allowed="isAllowed(VK.LSHIFT)" :width="W_LSHIFT" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="Z" :code="VK.Z" :allowed="isAllowed(VK.Z)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="X" :code="VK.X" :allowed="isAllowed(VK.X)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="C" :code="VK.C" :allowed="isAllowed(VK.C)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="V" :code="VK.V" :allowed="isAllowed(VK.V)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="B" :code="VK.B" :allowed="isAllowed(VK.B)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="N" :code="VK.N" :allowed="isAllowed(VK.N)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="M" :code="VK.M" :allowed="isAllowed(VK.M)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="," :code="VK.COMMA" :allowed="isAllowed(VK.COMMA)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="." :code="VK.DOT" :allowed="isAllowed(VK.DOT)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="/" :code="VK.SLASH" :allowed="isAllowed(VK.SLASH)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="Shift" :code="VK.RSHIFT" :allowed="isAllowed(VK.RSHIFT)" width="" :disabled="locked" @toggle="handleToggle" class="flex-1" />
          </div>

          <!-- 底行 Ctrl 行 -->
          <div class="flex gap-1">
            <KeyButton label="Ctrl" :code="VK.LCTRL" :allowed="isAllowed(VK.LCTRL)" :width="W_LCTRL" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="Win" :code="VK.LWIN" :allowed="isAllowed(VK.LWIN)" :width="W_MOD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="Alt" :code="VK.LALT" :allowed="isAllowed(VK.LALT)" :width="W_MOD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="" :code="VK.SPACE" :allowed="isAllowed(VK.SPACE)" width="" :disabled="locked" @toggle="handleToggle" class="flex-1" />
            <KeyButton label="Alt" :code="VK.RALT" :allowed="isAllowed(VK.RALT)" :width="W_MOD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="Win" :code="VK.RWIN" :allowed="isAllowed(VK.RWIN)" :width="W_MOD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="Menu" :code="VK.MENU" :allowed="isAllowed(VK.MENU)" :width="W_MOD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="Ctrl" :code="VK.RCTRL" :allowed="isAllowed(VK.RCTRL)" width="" :disabled="locked" @toggle="handleToggle" class="flex-1" />
          </div>
        </div>

        <!-- ========== 中间导航区：系统键 + 编辑键 + 方向键 ========== -->
        <div class="flex flex-col justify-between gap-3 pt-2">
          <!-- 系统键行 -->
          <div class="flex gap-1">
            <KeyButton label="PrtSc" :code="VK.SNAPSHOT" :allowed="isAllowed(VK.SNAPSHOT)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="ScrLk" :code="VK.SCROLL"   :allowed="isAllowed(VK.SCROLL)"   :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            <KeyButton label="Pause" :code="VK.PAUSE"    :allowed="isAllowed(VK.PAUSE)"    :width="W_STD" :disabled="locked" @toggle="handleToggle" />
          </div>
          <!-- 编辑键区 -->
          <div class="flex flex-col gap-1">
            <div class="flex gap-1">
              <KeyButton label="Ins" :code="VK.INSERT" :allowed="isAllowed(VK.INSERT)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
              <KeyButton label="Home" :code="VK.HOME" :allowed="isAllowed(VK.HOME)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
              <KeyButton label="PgUp" :code="VK.PGUP" :allowed="isAllowed(VK.PGUP)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            </div>
            <div class="flex gap-1">
              <KeyButton label="Del" :code="VK.DELETE" :allowed="isAllowed(VK.DELETE)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
              <KeyButton label="End" :code="VK.END" :allowed="isAllowed(VK.END)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
              <KeyButton label="PgDn" :code="VK.PGDN" :allowed="isAllowed(VK.PGDN)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            </div>
          </div>

          <!-- 方向键区 -->
          <div class="flex flex-col gap-1">
            <div class="flex gap-1 justify-center">
              <KeyButton label="↑" :code="VK.UP" :allowed="isAllowed(VK.UP)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            </div>
            <div class="flex gap-1">
              <KeyButton label="←" :code="VK.LEFT" :allowed="isAllowed(VK.LEFT)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
              <KeyButton label="↓" :code="VK.DOWN" :allowed="isAllowed(VK.DOWN)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
              <KeyButton label="→" :code="VK.RIGHT" :allowed="isAllowed(VK.RIGHT)" :width="W_STD" :disabled="locked" @toggle="handleToggle" />
            </div>
          </div>
        </div>

        <!-- ========== 数字小键盘区 (Grid 布局) ========== -->
        <div class="grid grid-cols-4 gap-1 pt-2 content-start">
          <KeyButton label="NumLk" :code="VK.NUMLK"   :allowed="isAllowed(VK.NUMLK)"   :width="W_STD" :disabled="locked" @toggle="handleToggle" />
          <KeyButton label="Num/"  :code="VK.NUMDIV"  :allowed="isAllowed(VK.NUMDIV)"  :width="W_STD" :disabled="locked" @toggle="handleToggle" />
          <KeyButton label="Num*"  :code="VK.NUMMUL"  :allowed="isAllowed(VK.NUMMUL)"  :width="W_STD" :disabled="locked" @toggle="handleToggle" />
          <KeyButton label="Num-"  :code="VK.NUMSUB"  :allowed="isAllowed(VK.NUMSUB)"  :width="W_STD" :disabled="locked" @toggle="handleToggle" />

          <KeyButton label="Num7"  :code="VK.NUM7"    :allowed="isAllowed(VK.NUM7)"    :width="W_STD" :disabled="locked" @toggle="handleToggle" />
          <KeyButton label="Num8"  :code="VK.NUM8"    :allowed="isAllowed(VK.NUM8)"    :width="W_STD" :disabled="locked" @toggle="handleToggle" />
          <KeyButton label="Num9"  :code="VK.NUM9"    :allowed="isAllowed(VK.NUM9)"    :width="W_STD" :disabled="locked" @toggle="handleToggle" />
          <KeyButton label="Num+"  :code="VK.NUMADD"  :allowed="isAllowed(VK.NUMADD)"  width=""         :disabled="locked" @toggle="handleToggle" class="row-span-2 h-full" />

          <KeyButton label="Num4"  :code="VK.NUM4"    :allowed="isAllowed(VK.NUM4)"    :width="W_STD" :disabled="locked" @toggle="handleToggle" />
          <KeyButton label="Num5"  :code="VK.NUM5"    :allowed="isAllowed(VK.NUM5)"    :width="W_STD" :disabled="locked" @toggle="handleToggle" />
          <KeyButton label="Num6"  :code="VK.NUM6"    :allowed="isAllowed(VK.NUM6)"    :width="W_STD" :disabled="locked" @toggle="handleToggle" />

          <KeyButton label="Num1"  :code="VK.NUM1"    :allowed="isAllowed(VK.NUM1)"    :width="W_STD" :disabled="locked" @toggle="handleToggle" />
          <KeyButton label="Num2"  :code="VK.NUM2"    :allowed="isAllowed(VK.NUM2)"    :width="W_STD" :disabled="locked" @toggle="handleToggle" />
          <KeyButton label="Num3"  :code="VK.NUM3"    :allowed="isAllowed(VK.NUM3)"    :width="W_STD" :disabled="locked" @toggle="handleToggle" />
          <KeyButton label="Enter" :code="VK.ENTER"   :allowed="isAllowed(VK.ENTER)"   width=""         :disabled="locked" @toggle="handleToggle" class="row-span-2 h-full" />

          <KeyButton label="Num0"  :code="VK.NUM0"    :allowed="isAllowed(VK.NUM0)"    width=""         :disabled="locked" @toggle="handleToggle" class="col-span-2" />
          <KeyButton label="Num."  :code="VK.NUMDOT"  :allowed="isAllowed(VK.NUMDOT)"  :width="W_STD" :disabled="locked" @toggle="handleToggle" />
        </div>

      </div>
    </div>
  </div>
</template>
