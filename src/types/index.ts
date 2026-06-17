export interface Config {
  rules: KeyRule[]
  app_rules: AppRule[]
  auto_unlock_timeout: number | null
  unlock_combo: number[]
  lock_combo: number[]
  version: number
  keyboard_devices: KeyboardDeviceConfig[]
}

export interface KeyRule {
  key: number
  label: string
  allowed: boolean
  modifiers?: number[]
}

export interface AppRule {
  process_names: string[]
  rules: KeyRule[]
}

export interface EngineSnapshot {
  locked: boolean
  grab_active: boolean
  total_blocked: number
  total_allowed: number
  active_app: string | null
  combo_progress: [number, number]
  lock_combo_progress: [number, number]
}

export type KeyboardLayout = 'ansi' | 'iso' | 'laptop'

export interface KeyDef {
  code: number
  label: string
  width: string
}

export interface KeyboardDeviceConfig {
  instance_id: string
  alias: string
  enabled: boolean
  is_target: boolean
}

export interface KeyboardDeviceInfo {
  instance_id: string
  alias: string
  enabled: boolean
  is_target: boolean
  name: string
  vendor_id: number
  product_id: number
}
