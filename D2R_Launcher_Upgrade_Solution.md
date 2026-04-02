# 暗黑破坏神2重制版 (D2R) 多开启动器升级技术方案

## 一、 现状分析与局限性
当前项目 (`D2R-fast-launcher`) 是一款基于 PowerShell 脚本 (`D2R launcher.ps1`) 的纯命令行多开工具。
其核心原理为：
1. **多开突破**：调用外部工具 `handle64.exe` 扫描并强制关闭 D2R.exe 进程中名为 `DiabloII Check For Other Instances` 的防多开 Mutex（互斥量）。
2. **快捷登录**：利用 D2R.exe 支持的 `-username` 和 `-password` 等命令行启动参数，直接绕过战网客户端登录游戏。
3. **窗口管理**：通过循环调用 Windows API (`FindWindow` 和 `SetWindowText`) 来重命名游戏窗口。

**当前痛点与局限：**
1. **UI 体验差**：黑底白字的终端界面，纯键盘输入序号操作，缺乏直观性。
2. **部署繁琐**：玩家必须手动将 `files` 目录及其中的 `handle64.exe` 和脚本拷贝到暗黑2游戏根目录才能使用，易错且不够“绿色”。
3. **启动效率低**：解除多开限制的过程需要将 `handle64.exe` 的结果输出到本地文本文件 (`d2r_handles.txt`)，再通过正则解析后二次调用 `handle64.exe` 杀句柄。这种高频的磁盘 I/O 和外部进程拉起，导致登录速度缓慢。
4. **并发控制弱**：通过全局 `FindWindow` 重命名窗口，如果同时启动多个账号，容易发生窗口句柄错乱。

---

## 二、 升级目标
1. **更智能美观的 UI**：抛弃命令行，提供现代化的图形交互界面（GUI），支持拖拽、点击、卡片式管理。
2. **更快捷的登录**：优化底层多开机制，摆脱 `handle64.exe` 依赖，实现内存级极速释放句柄和并发安全启动。
3. **自动拷贝与免配置**：实现启动器“随处可放”，自动定位游戏路径，自动释放/拷贝必要文件。

---

## 三、 技术选型建议
为了兼顾“绝美的 UI”、“极快的系统底层交互”以及“免安装的便捷性”，建议采用以下两种技术栈之一：

### 方案 A：Tauri + Vue3/React + Rust (强烈推荐)
* **前端 (UI)**：Vue3 或 React + Tailwind CSS，能够打造出现代化、动画丰富的 Web 级精美界面。
* **后端 (逻辑)**：Rust。极低资源占用（打包后体积 < 10MB），可直接调用 Windows Native API (如 `NtQuerySystemInformation`) 处理句柄，运行效率极高。

### 方案 B：C# WPF / WinUI 3 (.NET 8)
* **前端 (UI)**：XAML + MaterialDesignThemes，符合 Windows 11 原生设计语言。
* **后端 (逻辑)**：C#。利用 `P/Invoke` 调用 Win32 API 同样可以实现底层句柄操作，开发效率极高，生态成熟。

---

## 四、 核心功能拓展技术方案

### 1. 更智能美观的 UI 设计 (UX/UI 升级)
* **卡片式账号管理**：将每个账号信息（账号名、区服、Mod、分辨率）封装为一张卡片，直观展示。
* **拖拽排序**：支持通过鼠标拖拽调整账号卡片顺序，该顺序即为“一键全开”时的启动顺序。
* **全局可视化设置**：提供统一的设置页面，管理默认游戏路径、启动间隔延迟、是否自动关闭战网等。
* **密码安全保护**：UI 层隐藏密码明文，底层使用 Windows DPAPI 或 AES-256 对本地配置文件 (`config.json`) 进行加密存储。

### 2. 更快捷的登录与多开底层重构 (性能升级)
* **内存级句柄释放 (摆脱 `handle64.exe`)**：
  * **原理**：使用 Rust 或 C# 直接调用 `NtQuerySystemInformation` 获取系统所有句柄，过滤出所属进程为 `D2R.exe` 且名称匹配 `\Sessions\*\BaseNamedObjects\DiabloII Check For Other Instances` 的 Mutex，然后调用 `DuplicateHandle` 结合 `DUPLICATE_CLOSE_SOURCE` 参数直接将其关闭。
  * **收益**：耗时从几秒钟降至几毫秒，彻底消除磁盘 I/O。
* **精准并发启动控制**：
  * 弃用 `FindWindow`（容易找错窗口）。
  * 改用：在拉起 `D2R.exe` 时记录其 **进程 PID**。通过不断轮询该 PID 对应的 `MainWindowHandle`，一旦主窗口句柄生成，立即调用 `SetWindowText` 进行重命名。这样即使多个账号同时启动，也能 100% 精准对应窗口。

### 3. 自动检测与自动拷贝文件 (免部署升级)
* **注册表自动嗅探**：
  * 程序启动时，自动读取注册表 `HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Diablo II Resurrected` 中的 `InstallLocation` 字段。
  * 若找到则自动绑定游戏目录，用户无需任何手动寻找路径的操作。
* **自动拷贝/内嵌资源释放**：
  * **工具单文件化**：将所有依赖（如图标、甚至 `handle64.exe` 如果作为备选方案保留的话）作为资源内嵌到可执行文件 (`.exe`) 中。
  * 启动器可以放在桌面直接运行。如果检测到游戏目录下缺失特定的 Mod 文件或配置，启动器在拉起游戏前，**自动将内嵌的文件拷贝释放到游戏对应目录**。
* **存档安全备份**：
  * 在启动前，可增加一个步骤：自动将 `C:\Users\<用户名>\Saved Games\Diablo II Resurrected` 目录压缩打包或拷贝到备份文件夹，防止多开异常导致本地单机存档损坏。

---

## 五、 实施路线图 (Roadmap)

1. **Phase 1: 核心技术验证 (Proof of Concept)**
   - 使用 C# 或 Rust 编写 Demo，验证通过 Native API 杀除 D2R.exe Mutex 的稳定性。
   - 验证通过 PID 精准捕获窗口并重命名的逻辑。
2. **Phase 2: UI 原型与前端开发**
   - 使用 Tauri 或 WPF 搭建界面框架，实现账号的增删改查及配置文件 (`JSON` 格式) 读写加密。
3. **Phase 3: 业务逻辑融合与自动化**
   - 接入注册表自动寻找游戏路径逻辑。
   - 接入自动备份和必要文件的自动拷贝逻辑。
4. **Phase 4: 测试与发布**
   - 进行极限多开压力测试（如同时启动 8 个账号）。
   - 打包生成单一绿色免安装 `.exe` 文件发布。