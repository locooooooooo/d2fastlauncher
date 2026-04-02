# 暗黑破坏神2重制版 (D2R) 多开启动器 - 技术实现文档 (Technical Specification)

## 1. 项目架构概述
本项目采用 **Tauri + Vue 3 (TypeScript) + Rust** 架构。
- **前端 (Vue 3 + Vite + Tailwind CSS)**: 负责渲染精美的卡片式 UI、账号管理逻辑、拖拽排序、系统设置。
- **后端 (Rust)**: 负责与 Windows 底层 API 交互，实现高权限操作，如：内存级读取与释放系统 Mutex、启动进程监控、注册表读取、文件 I/O 拷贝。
- **通信 (Tauri IPC)**: 前后端通过 Tauri 的 IPC (Inter-Process Communication) 机制进行通信。

## 2. 目录结构设计 (Scaffolding)
```text
D2R-Fast-Launcher/
├── src/                  # 前端 Vue3 源码目录
│   ├── assets/           # 静态资源 (背景图、图标等)
│   ├── components/       # UI 组件 (AccountCard, SettingsModal)
│   ├── views/            # 页面级组件 (MainView)
│   ├── store/            # 状态管理 (Pinia) - 用于管理账号列表
│   ├── utils/            # 前端工具类 (加密、拖拽)
│   └── App.vue           # 根组件
├── src-tauri/            # 后端 Rust 源码目录
│   ├── src/
│   │   ├── main.rs       # Rust 主入口
│   │   ├── winapi.rs     # 封装 Windows Native API (NtQuerySystemInformation)
│   │   ├── process.rs    # D2R 进程拉起与窗口重命名控制
│   │   └── registry.rs   # 注册表读取 (定位游戏路径)
│   ├── tauri.conf.json   # Tauri 配置文件
│   └── Cargo.toml        # Rust 依赖配置
├── package.json          # Node.js 依赖配置
└── tailwind.config.js    # TailwindCSS 配置
```

## 3. 核心接口定义 (Tauri IPC)

### 3.1 账号与设置存储
- `invoke('get_accounts') -> Vec<Account>`: 获取本地保存的所有账号配置 (Rust 负责解密读取 JSON)。
- `invoke('save_accounts', { accounts }) -> Result<(), String>`: 加密保存账号列表。
- `invoke('get_settings') -> Settings`: 获取全局设置 (游戏路径、延迟等)。

### 3.2 核心多开逻辑 (Rust 端)
- `invoke('kill_d2r_mutex') -> Result<usize, String>`
  - **描述**: 扫描系统中所有的 `D2R.exe` 进程，强制关闭其 `DiabloII Check For Other Instances` 互斥量。返回成功关闭的句柄数量。
  - **依赖库**: `winapi` 或 `windows` crate (`NtQuerySystemInformation`, `DuplicateHandle`)。
- `invoke('launch_d2r', { account }) -> Result<u32, String>`
  - **描述**: 根据账号配置 (用户名、密码、区服、Mod) 拼接启动参数，拉起 `D2R.exe`。
  - **返回**: 拉起成功后返回进程的 PID。

### 3.3 窗口与环境控制
- `invoke('rename_window', { pid, title }) -> Result<(), String>`
  - **描述**: 根据 PID 寻找主窗口，将其重命名为指定的 `title`（如账号名）。
- `invoke('auto_detect_game_path') -> Result<String, String>`
  - **描述**: 从注册表中嗅探 D2R 安装路径。

### 3.4 战网启动与国服适配机制 (Battle.net Launch)
- `invoke('launch_bnet') -> Result<String, String>`
  - **描述**: 专门针对需要验证码、手机号验证的区服（如国服）或启用了安全令的账号设计的启动模式。该模式通过执行 `Battle.net.exe --exec="launch OSI"` 直接唤起战网客户端并跳转至 D2R 启动页。
  - **工作流**:
    1. 前端调用 `kill_d2r_mutex` 解除当前已运行游戏的多开限制。
    2. 前端调用 `launch_bnet` 唤起战网客户端。
    3. 用户在战网客户端中手动完成登录（处理验证码/安全令）并点击“进入游戏”。
    4. 游戏启动后，用户可继续在启动器中点击“解除多开限制”按钮，为下一次多开做准备。
  - **前端支持**: 账号配置中新增 `loginMethod` 字段（`auto` 直连启动 / `bnet` 战网启动），并在 UI 中提供全局的“解除多开限制”按钮。

## 4. 迭代开发计划 (Execution Plan)

### Iteration 1 (Sprint 1): 环境初始化与 Rust 核心多开 PoC
- 初始化 Tauri + Vue3 项目结构。
- 引入 Rust 的 `windows` crate，编写 `kill_d2r_mutex` 核心函数。
- 编写测试前端页面，一键测试释放句柄并拉起 D2R。

### Iteration 2 (Sprint 2): 前端 UI 原型与账号管理
- 搭建 Vue3 + TailwindCSS 的暗黑风格 UI。
- 实现卡片式的账号增删改查。
- 引入 Pinia 管理状态，引入拖拽库实现卡片排序。
- 实现本地配置文件的加密存储 (Rust 侧实现 AES 加密，前端透明调用)。

### Iteration 3 (Sprint 3): 业务联调与免部署功能
- 将前端点击 "启动" 绑定到 Rust 的 `launch_d2r` 和 `rename_window`。
- 实现批量启动 ("一键全开") 逻辑，带自定义延迟。
- 增加注册表嗅探，首次启动自动寻址游戏目录。

### Iteration 4 (Sprint 4): 测试与打包发布
- 添加文件自动拷贝与存档备份机制。
- 配置 `tauri.conf.json`，定制图标与应用信息。
- 执行 `tauri build`，生成单文件绿色免安装可执行程序 (.exe)。