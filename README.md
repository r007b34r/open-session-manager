# open Session Manager

`open Session Manager`，简称 `OSM`。

本项目面向 Win11、Linux 和 WSL 用户，目标是把本地终端代码助手的会话、配置、密钥引用和清理动作统一纳入一个本地优先的治理平台。

首期目标聚焦：

- 识别本地安装的 `Codex`、`Claude Code`、`OpenCode`，并为后续扩展更多助手预留适配器接口
- 读取原生会话记录、配置文件、日志和关键运行元数据
- 在网页/桌面界面中展示真实主题、进度、价值密度、风险等级和最后活跃时间
- 在删除前先导出高价值会话为 Markdown，并支持归档、迁移、软删除和审计日志
- 默认对敏感配置和 API key 做脱敏处理，只在显式授权下进行更深层查看或修改

当前仓库先完成了本地 `git` 初始化，以及调研/设计文档沉淀：

- [竞品与数据源分析](docs/research/2026-03-15-agent-session-landscape.md)
- [完整设计方案](docs/plans/2026-03-15-open-session-manager-design.md)
- [实施计划](docs/plans/2026-03-15-open-session-manager.md)

## 当前能力

- Rust 核心已经具备安装/路径发现、Codex/Claude Code/OpenCode 会话解析、洞察评分、配置与凭据审计
- Web / Tauri 桌面控制台已经具备总览页、Session Explorer、Session Detail、Config Center、Audit Center 的初版界面
- 已支持 Markdown 导出、软删除、恢复与审计日志链路，且“先导出 Markdown，再移入隔离区”现在是前后端双重约束
- 默认以脱敏方式展示密钥和敏感配置
- 已支持真实本地 snapshot、持久化审计数据库、桌面调试构建与浏览器 fallback 开发流

## 你可以用它做什么

- 一次性扫描 Win11、Linux、WSL 中多个终端代码助手的本地会话与配置
- 识别会话真实主题、摘要、当前进度、价值分、风险标记和最后活跃时间
- 在删除前先导出高价值会话为 Markdown，保留核心上下文和后续复用材料
- 识别第三方中转、危险权限、shell hook、宽松审批策略等配置风险
- 通过审计日志追踪“导出、隔离、恢复”每一步是谁在什么时间做的

## 安全原则

- 默认脱敏：配置页只显示脱敏后的密钥，不展示明文 key
- 导出优先：未形成 Markdown 留档的会话，不能直接移入隔离区
- 本地优先：会话解析、快照生成、配置审计和清理动作都以本地文件为源
- 可恢复：软删除会先进入隔离区，并伴随 manifest 与审计记录
- 可追踪：导出、软删除、恢复都会写入持久化审计数据库

## 本地运行

### 依赖

- Rust toolchain
- Node.js 20+

### Web 开发

```bash
npm --prefix web install
npm --prefix web run dev
```

### 桌面开发 / 构建

仓库内已经提供了统一桌面入口脚本，会自动在仓库根目录下调用 Tauri，并补齐本机 `cargo` 路径。

```bash
npm --prefix web run tauri:dev
npm --prefix web run tauri:build -- --debug
```

也可以直接使用根目录脚本：

```bash
node scripts/run-tauri.mjs dev
node scripts/run-tauri.mjs build --debug
```

### 发布构建

如果要生成 release 可执行文件，可执行：

```bash
npm --prefix web run tauri:build
```

当前 Windows 调试构建产物位于：

```text
target/debug/open-session-manager-core.exe
```

release 构建产物位于：

```text
target/release/open-session-manager-core.exe
```

### 核心测试

```bash
cargo test -p open-session-manager-core
npm --prefix web run test
```

也可以直接运行统一验证入口：

```bash
powershell -ExecutionPolicy Bypass -File scripts/verify.ps1
```

### 端到端验证

首次运行需要安装 Playwright Chromium：

```bash
npx --prefix web playwright install chromium
npm --prefix web run e2e
```

## 默认输出位置

桌面运行时默认会使用以下目录：

### Windows

- 审计数据库：`%LOCALAPPDATA%/OpenSessionManager/audit/audit.db`
- 隔离区：`%LOCALAPPDATA%/OpenSessionManager/quarantine/`
- Markdown 导出：`%USERPROFILE%/Documents/OpenSessionManager/exports/`

### Linux

- 审计数据库：`$XDG_DATA_HOME/open-session-manager/audit/audit.db`
- 隔离区：`$XDG_DATA_HOME/open-session-manager/quarantine/`
- Markdown 导出：`$HOME/Documents/OpenSessionManager/exports/`

## 当前 UI 演示数据

当前前端和桌面端已经统一成三层数据来源顺序：

1. Tauri 原生命令
2. `web/public/dashboard-snapshot.json`
3. `web/src/lib/api.ts` 内置 typed fixture

如果要把 Rust 核心导出的真实本地快照灌给浏览器开发环境，可执行：

```bash
node scripts/export-dashboard-snapshot.mjs
```

如果要用 fixtures 生成可重现的演示快照，可执行：

```bash
node scripts/export-dashboard-snapshot.mjs --fixtures tests/fixtures
```

默认输出路径是 `web/public/dashboard-snapshot.json`。如果只想验证导出链路、不影响默认 UI 演示数据，可以显式指定输出路径：

```bash
node scripts/export-dashboard-snapshot.mjs --fixtures tests/fixtures --output temp/dashboard-snapshot.json
```

## 桌面运行时

仓库当前已经接上可运行的 Tauri 桌面层：

- `src-tauri/tauri.conf.json`
- `src-tauri/src/desktop.rs`
- `scripts/run-tauri.mjs`

当前桌面运行时能力：

- 无参数启动二进制时进入 Tauri 窗口
- 前端优先通过 Tauri 命令读取真实本地 snapshot
- 导出 Markdown 与软删除动作可直接调用 Rust 原生命令，并返回最新 snapshot
- 调试构建已在当前 Windows 11 环境验证通过

## 项目结构

```text
src-tauri/      Rust 核心、Tauri 运行时、快照与清理动作
web/            React 控制台、i18n、前端测试与桌面桥接
tests/fixtures/ 多助手真实结构夹具
tests/e2e/      Playwright 端到端验证
docs/research/  竞品调研与数据源分析
docs/plans/     设计方案、实施计划、发布就绪计划
docs/release/   支持矩阵与发布材料
scripts/        snapshot 导出、验证入口、Tauri 启动辅助
```

## 当前已知限制

- Linux 桌面实机构建与真实目录回归还未在本仓库内完成证据闭环
- 当前尚未提供“配置修改/删除后写回真实文件”的完整 UI
- 隔离区列表与恢复流程目前以后端能力为主，前端入口还不完整
- 还没有 MSI / AppImage / deb / 签名产物链路
- WSL 仍缺少独立 companion collector 闭环

仍未完成的发布项见：

- [支持矩阵](docs/release/support-matrix.md)
- [发布就绪实施计划](docs/plans/2026-03-15-release-readiness.md)
