# KeyLock Pro

> 你是否也曾在图书馆把笔记本摊开放在键盘上写字，结果屏幕突然冒出满屏乱码？KeyLock Pro 是一款跨平台键盘锁定工具 —— 在咖啡厅、图书馆、自习室，只要你想把书本放在笔记本键盘上书写，它就能一键锁定键盘，彻底告别误触烦恼。

<img src="./public/list.png" width="450" height="450" />

## 功能亮点

- **🔒 全局键盘锁定** — 一键锁定所有按键，误触？不存在的
- **✅ 逐键白名单** — 锁定状态下，可单独放行某些按键（如音量键、媒体键）
- **🎯 应用感知规则** — 针对不同前台应用（如游戏、浏览器）自动切换按键策略
- **⌨️ 自定义解锁组合** — 默认 `Ctrl + Alt + L`，你想设成什么都行
- **⏱️ 定时自动解锁** — 可配置超时时间（默认 5 分钟），到点自动解除
- **📋 系统托盘** — 最小化到托盘，左键恢复窗口，右键快速操作
- **📊 统计面板** — 实时展示拦截 / 放行次数，一目了然

## 界面预览

### 主界面

![自定义解锁组合](/public/image.png)

## 下载安装

从 [Releases](https://github.com/jdhnsu/keyboard-locker/releases) 页面下载对应平台的安装包即可使用。

## 使用指南

### 基础操作

1. 打开 KeyLock Pro，主界面展示完整的键盘映射图
2. 点击 **锁定键盘** 按钮或通过系统托盘菜单，键盘即刻进入锁定状态
3. 锁定状态下，仅白名单中的按键可以输入，其余按键全部拦截
4. 按下 **解锁组合键**（可自定义）即可一键解锁

### 多键盘设备管理

进入 **设备管理** 页面（点击标题栏齿轮图标）：

1. 所有已连接的键盘自动列出，显示设备名、VID / PID
2. 点击 **开始识别** 后，敲击任意键盘的按键，对应设备卡片会高亮闪烁
3. 可为设备设置别名方便区分（如"笔记本内置键盘""外接机械键盘"）
4. **参与锁定** — 启用后该键盘受锁定状态控制
5. **目标键盘** — 标记为目标的键盘在锁定后完全拦截，非目标键盘仅通过规则过滤

### 配置白名单

在键盘映射界面，直接点击某个按键即可将其加入或移出白名单：

- <span style="color:#10B981">● 绿色</span> — 已放行，锁定状态下该按键可用
- <span style="color:#EF4444">● 红色</span> — 已拦截，锁定状态下该按键禁用

### 自定义解锁组合

修改配置文件 `config.json` 中的 `unlock_combo` 字段（键码数组），或在程序快捷键面板中直接录制。

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
│  │ App.vue    │ │Components│ │ useKeyboardState│  │
│  │ (Layout)   │ │ (UI 组件)│ │  (状态管理)     │  │
│  └────────────┘ └──────────┘ └────────────────┘  │
│                        │                          │
│              Tauri IPC (invoke / event)           │
├────────────────────────┼──────────────────────────┤
│                    Backend (Rust)                  │
│  ┌──────────────────────────────────────────────┐ │
│  │              commands/                        │ │
│  │  lifecycle.rs  │  config.rs  │  keyboard.rs  │ │
│  │  锁/解锁/重启   │  规则增删改  │  按键状态查询  │ │
│  │  device.rs     │                             │ │
│  │  设备枚举/更新   │                             │ │
│  └──────────────────────┬───────────────────────┘ │
│  ┌──────────────────────┴───────────────────────┐ │
│  │              locker/ (核心引擎)               │ │
│  │  ┌──────────┐ ┌────────┐ ┌────────────────┐  │ │
│  │  │ engine.rs│ │combo.rs│ │  filter.rs     │  │ │
│  │  │ 引擎调度  │ │组合键  │ │  按键过滤判定   │  │ │
│  │  └──────────┘ └────────┘ └────────────────┘  │ │
│  │  ┌──────────┐ ┌───────────────┐              │ │
│  │  │ rules.rs │ │ device_manager│              │ │
│  │  │ 配置结构  │ │ .rs           │              │ │
│  │  └──────────┘ │ 设备枚举/识别  │              │ │
│  │               └───────────────┘              │ │
│  │  ┌──────────┐ ┌───────────────┐              │ │
│  │  │raw_grab  │ │  shortcut.rs  │              │ │
│  │  │.rs       │ │  全局快捷键    │              │ │
│  │  │Raw Input │ │  注册          │              │ │
│  │  │抓取循环  │ │               │              │ │
│  │  └──────────┘ └───────────────┘              │ │
│  └──────────────────────────────────────────────┘ │
│  ┌──────────────────────────────────────────────┐ │
│  │              platform/ (平台适配)             │ │
│  │  windows.rs  │  macos.rs  │  linux.rs         │ │
│  │  Raw Input   │  CGEvent   │   evdev grab      │ │
│  │  设备识别     │  监听      │                   │ │
│  └──────────────────────────────────────────────┘ │
│  ┌──────────────┐ ┌────────────┐                  │
│  │ config/      │ │   tray/    │                  │
│  │ JSON 持久化  │ │  托盘菜单  │                  │
│  └──────────────┘ └────────────┘                  │
└──────────────────────────────────────────────────┘
```

### 数据流

1. **设备枚举**：启动时调用 `enumerate_keyboard_devices()` → `GetRawInputDeviceList` 枚举 HID 键盘 → 合并已保存配置 → 前端展示设备列表
2. **敲击识别**：点击「开始识别」→ 创建消息窗口 + `RIDEV_EXINPUTSINK` → 捕获 `WM_INPUT` → 通过 Tauri event 推送 `keyboard-tapped`（含 `instance_id`）→ 前端高亮对应卡片
3. **锁定流程**：用户点击锁定 → `toggle_lock` IPC → `Engine::lock()` 设置状态 → Raw Input 抓取循环按 `block_map` 拦截指定键盘
4. **按键过滤**：Raw Input 捕获按键 → `filter::evaluate()` 依据规则判定 → 放行的键通过 `SendInput` 重新注入
5. **解锁组合**：按键进入 `ComboTracker::feed_key_press()` → 序列匹配成功 → 自动调用 `Engine::unlock()` 解锁
6. **状态同步**：Rust 通过 Tauri `emit("lock-state-changed", ...)` 推送 → Vue `listen()` 监听并更新 UI

### Windows 平台按键拦截方案

| 方案 | 技术 | 粒度 | 说明 |
|------|------|------|------|
| 设备识别 | Raw Input (`RIDEV_EXINPUTSINK`) | 消息窗口 | 仅监听，不拦截，用于敲击识别 |
| 锁定拦截 | Raw Input (`RIDEV_NOLEGACY`) + `SendInput` 回注 | 按键盘粒度 | 拦截后通过 `SendInput` 放行允许的按键 |

## 技术栈

### 前端

| 项目 | 描述 |
|------|------|
| [Vue 3](https://vuejs.org/) | 渐进式 JavaScript 框架 |
| [TypeScript](https://www.typescriptlang.org/) | JavaScript 的超集，提供静态类型检查 |
| [Vite](https://vitejs.dev/) | 下一代前端构建工具 |
| [Tailwind CSS v4](https://tailwindcss.com/) | 原子化 CSS 框架 |
| [Inter](https://rsms.me/inter/) | 界面字体 |
| [JetBrains Mono](https://www.jetbrains.com/lp/mono/) | 技术数据显示专用等宽字体 |
| [Material Symbols](https://fonts.google.com/icons) | 图标字体 |

### 后端 (Rust)

| 项目 | 描述 |
|------|------|
| [Tauri v2](https://tauri.app/) | 跨平台桌面应用框架 |
| [Tauri Plugin Global Shortcut](https://github.com/tauri-apps/plugins-workspace) | 全局快捷键注册 |
| [Tauri Plugin Single Instance](https://github.com/tauri-apps/plugins-workspace) | 单实例锁 |
| [Tauri Plugin Opener](https://github.com/tauri-apps/plugins-workspace) | 打开系统文件/URL |
| [Tauri Plugin Notification](https://github.com/tauri-apps/plugins-workspace) | 系统通知 |
| [Serde](https://serde.rs/) | 序列化/反序列化框架 |
| [parking_lot](https://github.com/Amanieu/parking_lot) | 高性能同步原语 |
| [log](https://github.com/rust-lang/log) | 日志门面 |

### 平台依赖

| 平台 | 项目 |
|------|------|
| Windows | [windows-sys](https://github.com/microsoft/windows-rs) — Win32 API 绑定 (Raw Input, SendInput) |
| macOS | [objc2](https://github.com/madsmtm/objc2) — Objective-C 运行时绑定 |
| Linux | [evdev](https://github.com/meh/rust-evdev) — 输入事件设备接口 |

## 开发

### 环境准备

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/) >= 1.80
- [Tauri CLI](https://tauri.app/) (`cargo install tauri-cli`)

### 快速启动

```bash
# 克隆仓库
git clone https://github.com/jdhnsu/keyboard-locker.git
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
├── index.html                # HTML 入口
├── src/                      # Vue 前端源码
│   ├── App.vue               # 根组件（视图切换、布局）
│   ├── main.ts               # 入口
│   ├── style.css             # 全局样式（M3 主题、Tailwind）
│   ├── components/           # UI 组件
│   │   ├── TitleBar.vue      # 自定义标题栏（拖动区域、窗口控制）
│   │   ├── KeyboardMapper.vue    # 可视化键盘映射
│   │   ├── KeyButton.vue     # 单个按键按钮
│   │   ├── DeviceManager.vue # 多键盘设备管理
│   │   ├── EngineToggle.vue  # 锁定/解锁开关
│   │   ├── ComboRecorder.vue # 快捷键录制器
│   │   ├── StatusBar.vue     # 拦截/放行统计
│   │   ├── PermissionBanner.vue # 权限提示条
│   │   └── TopAppBar.vue     # 备用标题栏（未使用）
│   ├── composables/          # 组合式 API
│   │   └── useKeyboardState.ts
│   └── types/                # TypeScript 类型
│       └── index.ts
├── src-tauri/                # Rust 后端源码
│   ├── Cargo.toml            # Rust 依赖
│   ├── tauri.conf.json       # Tauri 窗口/插件配置
│   ├── icons/                # 应用图标
│   ├── capabilities/         # 权限声明
│   └── src/
│       ├── main.rs           # 桌面入口
│       ├── lib.rs            # Tauri Builder 装配
│       ├── state.rs          # EngineState 共享状态
│       ├── commands/         # IPC 命令
│       │   ├── lifecycle.rs  # 锁/解锁/状态查询
│       │   ├── config.rs     # 配置读写
│       │   ├── keyboard.rs   # 按键白名单控制
│       │   └── device.rs     # 键盘设备枚举/更新
│       ├── locker/           # 核心锁定引擎
│       │   ├── engine.rs     # 引擎调度、生命周期
│       │   ├── raw_grab.rs   # Windows Raw Input 抓取循环
│       │   ├── device_manager.rs # 键盘枚举、敲击识别
│       │   ├── rules.rs      # 配置数据结构
│       │   ├── filter.rs     # 按键过滤判定
│       │   ├── combo.rs      # 组合键序列匹配
│       │   └── shortcut.rs   # 全局快捷键注册
│       ├── config/           # 配置持久化
│       │   └── store.rs
│       ├── platform/         # 平台适配层
│       │   ├── windows.rs
│       │   ├── macos.rs
│       │   └── linux.rs
│       └── tray/             # 系统托盘
│           └── builder.rs
├── public/                   # 静态资源
├── dist/                     # 前端构建输出
├── package.json
├── vite.config.ts
├── tsconfig.json
├── tsconfig.app.json
└── AGENTS.md                 # 项目上下文记录
```

## 许可证

MIT License
