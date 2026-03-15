# open Session Manager 发布说明

## 项目简介

`open Session Manager`（`OSM`）是一个面向 Win11、Linux 和 WSL 用户的本地优先治理平台，用来统一识别终端代码助手的会话、配置和高风险密钥引用，并在真正删除之前先帮助用户判断哪些内容值得保留、迁移或提炼。

当前版本已经支持：

- Codex
- Claude Code
- OpenCode

## 本版重点

- 增加 Tauri 桌面运行时，桌面端可直接调用 Rust 原生命令读取真实本地 snapshot
- 增加真实 snapshot 管线与持久化审计历史读取
- 增加中英文切换，并按系统/浏览器语言自动选择默认语言
- 增加“导出 Markdown 之后才允许软删除”的前后端双重守卫
- 增加统一验证入口，可一次性运行 Rust、Web、桌面构建和 E2E

## 核心能力

- 识别多种助手的本地会话目录和配置文件
- 提取会话真实标题、摘要、进度、风险和价值信号
- 审计第三方中转、危险权限、shell hook、宽松审批等配置风险
- Markdown 导出、隔离、恢复与审计追踪
- 默认脱敏显示敏感配置和 key

## 快速开始

```bash
npm --prefix web install
powershell -ExecutionPolicy Bypass -File scripts/verify.ps1
npm --prefix web run tauri:dev
```

## 构建

```bash
npm --prefix web run tauri:build
```

调试构建产物：

```text
target/debug/open-session-manager-core.exe
```

发布构建产物：

```text
target/release/open-session-manager-core.exe
```

## 当前限制

- Linux 桌面构建与真实目录回归证据仍需补齐
- 前端尚未提供完整的隔离区管理与恢复 UI
- 暂无正式安装包与签名流程
