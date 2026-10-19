# llama.cpp 启动脚本中控台

管理本地 llama.cpp 启动脚本（`.cmd` / `.bat` / `.sh`）的小工具：可视化编辑启动参数、切换引擎版本、在工具内启动进程并查看日志，全程不弹出独立终端窗口。

> 基于 **Tauri 2 + Rust + Svelte**。工具本身只是"壳"，不参与推理，不影响 llama.cpp 性能。

## 状态

开发中（WIP）。需求与设计文档暂存本地，不随仓库上传。

## 功能（规划）

- 选择工作目录，扫描脚本列表，支持刷新 / 打开所在文件夹
- 解析脚本中的 llama.cpp 参数并可视化编辑，保存前差异预览
- 多版本 llama.cpp 引擎路径保存与快速切换
- 工具内启动脚本（不弹窗），实时日志（自动滚动 / 暂停 / 清空 / 复制 / 停止）
- 原始脚本编辑器（语法高亮）
- 轻量 JSON 配置（便携模式，不污染系统环境）

## 下载

前往 [Releases](../../releases) 下载对应平台的可执行文件：

- Windows：`*-setup.exe`（安装版）或免安装 `.exe`
- macOS：`.dmg`
- Linux：`.AppImage` / `.deb`

## 本地开发

### 前置

- Node.js 18+
- Rust 工具链（[rustup](https://rustup.rs/)）
- Windows 另需 MSVC C++ 生成工具（Visual Studio Build Tools，勾选 "Desktop development with C++"）

### 命令

```bash
npm install
npm run tauri dev     # 开发调试
npm run tauri build   # 打包（在目标平台本机执行）
```

## 许可证

待定
