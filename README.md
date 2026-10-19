# LlamaDock

管理本地 llama.cpp 启动脚本（`.cmd` / `.bat` / `.sh` / `.ps1`）的小工具：可视化编辑启动参数、切换引擎版本、在工具内启动进程并查看实时日志，全程不弹出独立终端窗口。

> 基于 **Tauri 2 + Rust + SvelteKit**。工具本身只是"壳"，不参与推理，不影响 llama.cpp 性能。

## 功能

- 选择工作目录并扫描脚本，支持搜索 / 刷新 / 打开所在文件夹；目录有变化时自动提示
- 解析脚本中的 llama.cpp 启动参数并可视化编辑（开关 / 数值 / 路径），保存前展示差异
- 原始脚本编辑器（shell 语法高亮），与参数面板双向同步
- 多版本 llama.cpp 引擎路径保存与快速切换，可批量应用到引用同一引擎的脚本
- 工具内启动脚本（Windows 不弹控制台窗口），实时日志终端（自动滚动 / 暂停 / 清空 / 复制）
- 停止进程树，状态栏显示状态 / PID / 端口 / 退出码
- 轻量 JSON 配置（便携模式，随程序目录；不污染系统环境）

## 已知边界

- 只结构化解析「直接调用 llama.cpp 可执行文件的启动行」（续行按脚本类型识别：cmd `^` / sh `\` / PowerShell 反引号）；含 `if` / `for` / `start` 的脚本请使用原始脚本编辑器
- Windows 无法原生执行 `.sh`，该情况下启动按钮会禁用（需 WSL / Git Bash）
- 每个脚本的进程与日志相互独立，可同时启动多个脚本；关闭工具不会自动结束已启动的进程

## 下载

前往 [Releases](../../releases) 下载对应平台的可执行文件：

- Windows：`*-setup.exe`（安装版）
- macOS：`.dmg`
- Linux：`.AppImage` / `.deb`

打 tag（如 `v0.1.0`）会触发 [Release 工作流](./.github/workflows/release.yml) 自动构建。

## 本地开发

### 前置

- Node.js 18+
- Rust 工具链（[rustup](https://rustup.rs/)）
- Windows 另需 MSVC C++ 生成工具（Visual Studio Build Tools，勾选 "Desktop development with C++"）

### 命令

```bash
npm install
npm run tauri dev     # 开发调试
npm run check         # 前端类型检查
npm run tauri build   # 打包（在目标平台本机执行）
cargo test --manifest-path src-tauri/Cargo.toml
```

## 许可证

待定（上传 GitHub 后添加）。
