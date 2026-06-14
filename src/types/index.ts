export interface Config {
  rules: KeyRule[]
  app_rules: AppRule[]
  auto_unlock_timeout: number | null
  unlock_combo: number[]
  version: number
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
}

export type KeyboardLayout = 'ansi' | 'iso' | 'laptop'

export interface KeyDef {
  code: number
  label: string
  width: string
}