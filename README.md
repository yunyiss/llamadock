# LlamaDock

管理本地 llama.cpp 启动脚本（`.cmd` / `.bat` / `.sh` / `.ps1`）的小工具：可视化编辑启动参数、切换引擎版本、在工具内启动进程并查看实时日志，全程不弹出独立终端窗口。

> 基于 **Tauri 2 + Rust + SvelteKit**。工具本身只是"壳"，不参与推理，不影响 llama.cpp 性能。

## 功能

- 选择工作目录并扫描脚本，支持搜索 / 刷新 / 打开所在文件夹；目录有变化时自动提示
- 解析脚本中的 llama.cpp 启动参数并可视化编辑（开关 / 数值 / 路径），保存前展示差异
- 参数面板默认只显示脚本中已有的参数，需要时从已登记的 190+ 个参数里搜索添加
- 原始脚本编辑器（shell 语法高亮），与参数面板双向同步
- 多版本 llama.cpp 引擎路径保存与快速切换，可批量应用到引用同一引擎的脚本
- 工具内启动脚本（Windows 不弹控制台窗口）；日志终端启动后自动展开，可拖拽调整高度 / 一键收起，支持清空 / 复制 / Ctrl+点击链接
- 停止进程树，状态栏显示状态 / PID / 端口 / 退出码
- 轻量 JSON 配置（便携模式，随程序目录；不污染系统环境）

## 平台与下载

发布产物全部为**免安装压缩包**：外层文件名带版本与平台便于区分，解压后统一得到 `LlamaDock`。前往 [Releases](../../releases) 下载对应平台的文件：

| 平台 | 文件 | 解压后 | 前提 |
| --- | --- | --- | --- |
| Windows | `LlamaDock_<版本>_windows-portable.zip` | `LlamaDock.exe` | 需系统 WebView2（Win10 1803+ / Win11 自带）；双击即可运行 |
| macOS | `LlamaDock_<版本>_macos-arm64-portable.zip` | `LlamaDock.app` | Apple Silicon（M 系列）专用；未签名，首次打开需右键 →「打开」 |
| Linux | `LlamaDock_<版本>_linux-bundled-portable.zip` | `LlamaDock.AppImage` | 自带 WebKitGTK，任意发行版可跑（约 78MB）；需 FUSE，未装时用 `--appimage-extract-and-run` |
| Linux | `LlamaDock_<版本>_linux-system-portable.tar.gz` | `LlamaDock` | 仅裸二进制，体积最小（约 7MB）；需系统已装 webkit2gtk-4.1 与 GTK3 |

Linux `system` 版的依赖：

```bash
sudo apt install libwebkit2gtk-4.1-0 libgtk-3-0   # Debian / Ubuntu / Kali
sudo dnf install webkit2gtk4.1 gtk3               # Fedora
sudo pacman -S webkit2gtk-4.1 gtk3                # Arch
```

打 tag（如 `v0.1.0`）会触发 [Release 工作流](./.github/workflows/release.yml) 自动构建；手动触发只构建产物、不创建 Release。

## 已知边界

- 只结构化解析「直接调用 llama.cpp 可执行文件的启动行」（续行按脚本类型识别：cmd `^` / sh `\` / PowerShell 反引号）；含 `if` / `for` / `start` 的脚本请使用原始脚本编辑器
- Windows 无法原生执行 `.sh`，该情况下启动按钮会禁用（需 WSL / Git Bash）
- 每个脚本的进程与日志相互独立，可同时启动多个脚本；关闭工具不会自动结束已启动的进程
- macOS 仅提供 Apple Silicon 构建，Intel Mac 不在支持范围
- 解压后的 AppImage 若丢失执行权限，需先 `chmod +x LlamaDock.AppImage`
- 便携模式下配置写在程序所在目录；若该目录不可写（例如 Linux 的 AppImage 挂载点只读），会回退到系统配置目录

## 本地开发

### 前置

- Node.js 18+
- Rust 工具链（[rustup](https://rustup.rs/)）
- Windows 另需 MSVC C++ 生成工具（Visual Studio Build Tools，勾选 "Desktop development with C++"）
- Linux 另需 `libwebkit2gtk-4.1-dev` / `libappindicator3-dev` / `librsvg2-dev` / `patchelf`

### 命令

```bash
npm install
npm run tauri dev     # 开发调试
npm run check         # 前端类型检查
npm run tauri build   # 打包（在目标平台本机执行）
npm run log:check     # 日志缓冲逻辑校验（预算 / 淘汰 / 游标语义）
cargo test --manifest-path src-tauri/Cargo.toml
```

## 性能自测

日志面板的内存表现可以用现成脚本复核（Windows / WebView2）：

```powershell
# 1) 日志缓冲的预算与淘汰逻辑（纯逻辑，秒级出结果）
npm run log:check

# 2) WebView2 renderer 的实际内存趋势（默认 10 分钟，每 10 秒采一次）
pwsh -File tools/monitor-webview2.ps1
```

**判读方式**：只看「是否持续爬升」，即平均每点增量接近 0（脚本以 ≤0.5MB 为合格）。
V8 不会立刻把释放的内存还给系统，**绝对值不下降是正常的**，别因此误判。

**实测记录（2026-10，空闲 10 分钟、无脚本运行）**：

```
首 59.0 MB → 末 53.5 MB | min 53.5 | max 59.0 | 平均每点 −0.10 MB → 稳定
（前 6 分钟回落 5.5MB，之后 3 分钟完全水平）
```

**待补**：高频日志场景——启动一个 `--verbose` 脚本后再跑同一条命令；以及点「停止」后再采 1 分钟，观察是否继续平稳。

**改造前后的结构变化**（改造前未留基线数值，故此处是结构对比而非数值对比）：

| | 改造前 | 改造后 |
| --- | --- | --- |
| 日志存储 | 每脚本一个大字符串，每批日志整体复制一次 | 块数组 O(1) 追加，零复制 |
| 容量上限 | 单脚本 400KB，**无全局上限** | 单脚本 512KB + **严格全局 4MB**（`npm run log:check` 覆盖） |
| 日志终端 | scrollback 5000、每批写一次、折叠时仍写入与重排 | 1500、150ms 合并写入、折叠时不写不重排 |

## 许可证

本项目采用 [MIT License](LICENSE)。
