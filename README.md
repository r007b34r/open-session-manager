# Agent Session Governance

本项目面向 Win11 和 Linux 用户，目标是把本地终端代码助手的会话、配置、密钥引用和清理动作统一纳入一个本地优先的治理平台。

首期目标聚焦：

- 识别本地安装的 `Codex`、`Claude Code`、`OpenCode`，并为后续扩展更多助手预留适配器接口
- 读取原生会话记录、配置文件、日志和关键运行元数据
- 在网页/桌面界面中展示真实主题、进度、价值密度、风险等级和最后活跃时间
- 在删除前先导出高价值会话为 Markdown，并支持归档、迁移、软删除和审计日志
- 默认对敏感配置和 API key 做脱敏处理，只在显式授权下进行更深层查看或修改

当前仓库先完成了本地 `git` 初始化，以及调研/设计文档沉淀：

- [竞品与数据源分析](docs/research/2026-03-15-agent-session-landscape.md)
- [完整设计方案](docs/plans/2026-03-15-agent-session-governance-design.md)
- [实施计划](docs/plans/2026-03-15-agent-session-governance.md)

## 当前能力

- Rust 核心已经具备安装/路径发现、Codex/Claude Code/OpenCode 会话解析、洞察评分、配置与凭据审计
- Web 控制台已经具备总览页、Session Explorer、Session Detail、Config Center、Audit Center 的初版界面
- 已支持 Markdown 导出、软删除、恢复与审计日志链路
- 默认以脱敏方式展示密钥和敏感配置

## 本地运行

### 依赖

- Rust toolchain
- Node.js 20+

### Web 开发

```bash
npm --prefix web install
npm --prefix web run dev
```

### 核心测试

```bash
cargo test -p agent-session-governance-core
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

## 当前 UI 演示数据

当前前端使用 `web/src/lib/api.ts` 中的 typed fixture snapshot 驱动，用来先验证交互流、布局和风险呈现。下一阶段会把这层替换为 Tauri 命令与真实本地索引数据。

当前前端已经改为“优先读取真实 snapshot，失败时回退到 fixture”。如果要把 Rust 核心导出的真实本地快照灌给前端，可执行：

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
