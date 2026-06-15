# KeyLock Pro

> Keyboard Locker — 一款跨平台键盘锁定桌面工具，保护你的键盘免受误触与熊孩子袭击。

## 功能简介

- **全局键盘锁定** — 一键锁定所有按键，被锁定的按键将无法输入
- **逐键白名单** — 锁定状态下可指定放行某些按键（如音量键、媒体键）
- **应用感知规则** — 针对不同前台应用（如游戏、浏览器）设置不同的按键策略
- **解锁组合键** — 默认 `Ctrl + Alt + L` 解锁，支持自定义组合键
- **定时自动解锁** — 可配置超时时间（默认 5 分钟），到时自动解除锁定
- **系统托盘** — 最小化到托盘，左键点击恢复窗口，右键菜单快速操作
- **统计面板** — 实时显示拦截 / 放行次数

## 截图

![Unlock Combo](/public/image.png)

## 下载 & 安装

推荐从 [Releases](https://github.com/jdhnsu/keyboard-locker/releases) 页面下载对应平台的安装包。

## 使用指南

### 基本操作

1. 打开 KeyLock Pro，主界面展示完整的键盘映射图
2. 点击 **锁定键盘** 按钮或使用系统托盘菜单，键盘即刻进入锁定状态
3. 在锁定状态下，仅白名单中的按键能够输入；其余按键被拦截
4. 按下 **解锁组合键** （可自定义）即可解锁


### 配置白名单

在键盘映射界面，直接点击某个按键即可将其加入/移出白名单：

- <span style="color:#10B981">● 绿色</span> — 已放行，锁定状态下该按键可用
- <span style="color:#EF4444">● 红色</span> — 已拦截，锁定状态下该按键禁用

### 自定义解锁组合

修改配置文件 `config.json` 中的 `unlock_combo` 字段（键码数组），或在 UI 中设置（待实现）。

### 配置文件位置

| 平台 | 路径 |
|------|------|
| Windows | `%APPDATA%\keyboard-locker\config.json` |
| macOS | `~/Library/Application Support/keyboard-locker/config.json` |
| Linux | `~/.local/share/keyboard-locker/config.json` |

## 架构设计

```
┌──────────────────────────────────────────────────┐
│                    Frontend                       │
│           Vue 3 + TypeScript + Tailwind           │
│  ┌────────────┐ ┌──────────┐ ┌────────────────┐  │
│  │ App.vue    │ │ Components│ │ useKeyboardState │  │
│  │ (Layout)   │ │ (UI 组件) │ │ (状态管理)      │  │
│  └────────────┘ └──────────┘ └────────────────┘  │
│                        │                          │
│              Tauri IPC (invoke / event)           │
├────────────────────────┼──────────────────────────┤
│                    Backend (Rust)                  │
│  ┌──────────────────────────────────────────────┐ │
│  │              commands/                        │ │
│  │  lifecycle.rs  │  config.rs  │  keyboard.rs  │ │
│  │  锁/解锁/切换   │  规则增删改  │  按键状态查询  │ │
│  └──────────────────────┬───────────────────────┘ │
│  ┌──────────────────────┴───────────────────────┐ │
│  │              locker/ (核心引擎)               │ │
│  │  ┌──────────┐ ┌────────┐ ┌────────────────┐  │ │
│  │  │ engine.rs │ │combo.rs│ │  filter.rs     │  │ │
│  │  │ 引擎调度   │ │组合键  │ │  按键过滤判定   │  │ │
│  │  └──────────┘ └────────┘ └────────────────┘  │ │
│  │  ┌──────────┐                                │  │
│  │  │ rules.rs │  配置数据结构与默认规则          │  │
│  │  └──────────┘                                │  │
│  └──────────────────────────────────────────────┘ │
│  ┌──────────────────────────────────────────────┐ │
│  │              platform/ (平台适配)             │ │
│  │  windows.rs  │  macos.rs  │  linux.rs         │ │
│  │  WH_KEYBOARD │  CGEvent   │   X11 grab        │ │
│  │  _LL 钩子    │  监听      │                   │ │
│  └──────────────────────────────────────────────┘ │
│  ┌──────────────┐ ┌────────────┐                  │
│  │ config/      │ │   tray/    │                  │
│  │ JSON 持久化  │ │  托盘菜单  │                  │
│  └──────────────┘ └────────────┘                  │
└──────────────────────────────────────────────────┘
```

### 数据流

1. **锁定流程**：用户点击锁定 → Vue 调用 `toggle_lock` IPC → `Engine::lock()` 设置状态 → 键盘钩子开始拦截按键
2. **按键过滤**：底层钩子捕获按键事件 → `Engine::handle_key_press()` → `filter::evaluate()` 依据规则判定 → 返回放行或拦截
3. **解锁组合**：按键进入 `ComboTracker::feed_key_press()` → 序列匹配成功 → 自动调用 `Engine::unlock()`
4. **状态同步**：Rust 通过 Tauri `emit("lock-state-changed", ...)` 推送 → Vue `listen()` 监听更新 UI

## 技术栈

### 前端

| 项目 | 描述 |
|------|------|
| [Vue 3](https://vuejs.org/) | 渐进式 JavaScript 框架 |
| [TypeScript](https://www.typescriptlang.org/) | JavaScript 的超集，提供静态类型检查 |
| [Vite](https://vitejs.dev/) | 下一代前端构建工具 |
| [Tailwind CSS](https://tailwindcss.com/) | 原子化 CSS 框架 |
| [Inter](https://rsms.me/inter/) | 界面字体 |
| [JetBrains Mono](https://www.jetbrains.com/lp/mono/) | 技术数据显示专用等宽字体 |

### 后端 (Rust)

| 项目 | 描述 |
|------|------|
| [Tauri v2](https://tauri.app/) | 跨平台桌面应用框架 |
| [Tauri Plugin Opener](https://github.com/tauri-apps/plugins-workspace) | 打开系统文件/URL |
| [Tauri Plugin Notification](https://github.com/tauri-apps/plugins-workspace) | 系统通知 |
| [Serde](https://serde.rs/) | 序列化/反序列化框架 |
| [parking_lot](https://github.com/Amanieu/parking_lot) | 高性能同步原语 |
| [log](https://github.com/rust-lang/log) | 日志门面 |
| [thiserror](https://github.com/dtolnay/thiserror) | 派生 `Error` trait |

### 平台依赖

| 平台 | 项目 |
|------|------|
| Windows | [windows-sys](https://github.com/microsoft/windows-rs) — Win32 API 绑定 |
| macOS | [objc2](https://github.com/madsmtm/objc2) — Objective-C 运行时绑定 |
| Linux | [x11](https://github.com/AltF02/rust-x11) — X11 协议绑定 |

## 开发

### 环境要求

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/) >= 1.80
- [Tauri CLI](https://tauri.app/) (`cargo install tauri-cli`)

### 快速开始

```bash
# 克隆仓库
git clone https://github.com/example/keyboard-locker.git
cd keyboard-locker

# 安装前端依赖
npm install

# 启动开发模式
npm run tauri dev

# 构建生产版本
npm run tauri build
```

### 项目结构

```
keyboard-locker/
├── src/                    # Vue 前端源码
│   ├── App.vue             # 根组件
│   ├── main.ts             # 入口
│   ├── style.css           # 全局样式
│   ├── components/         # UI 组件
│   │   ├── TopAppBar.vue
│   │   ├── KeyboardMapper.vue
│   │   ├── KeyButton.vue
│   │   ├── EngineToggle.vue
│   │   ├── ModeToggle.vue
│   │   └── PermissionBanner.vue
│   ├── composables/        # 组合式 API
│   │   └── useKeyboardState.ts
│   └── types/              # TypeScript 类型
│       └── index.ts
├── src-tauri/              # Rust 后端源码
│   ├── Cargo.toml          # Rust 依赖
│   ├── tauri.conf.json     # Tauri 配置
│   ├── icons/              # 应用图标
│   ├── capabilities/       # 权限声明
│   └── src/
│       ├── main.rs         # 入口
│       ├── lib.rs          # Tauri Builder 装配
│       ├── state.rs        # 共享状态
│       ├── commands/       # IPC 命令
│       │   ├── lifecycle.rs
│       │   ├── config.rs
│       │   └── keyboard.rs
│       ├── locker/         # 核心锁定引擎
│       │   ├── engine.rs
│       │   ├── rules.rs
│       │   ├── filter.rs
│       │   └── combo.rs
│       ├── config/         # 配置持久化
│       │   └── store.rs
│       ├── platform/       # 平台适配层
│       │   ├── windows.rs
│       │   ├── macos.rs
│       │   └── linux.rs
│       └── tray/           # 系统托盘
│           └── builder.rs
├── public/                 # 静态资源
├── dist/                   # 前端构建输出
├── package.json
├── vite.config.ts
├── tsconfig.json
└── DESIGN.md               # 设计规范
```

## 许可证

MIT License
