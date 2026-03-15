# open Session Manager

`open Session Manager`，简称 `OSM`。

`OSM` 是一个面向 Win11、Linux 和 WSL 用户的本地优先治理平台，用来统一识别终端代码助手的会话、配置、密钥引用和清理动作，并在真正删除之前先帮助用户判断哪些内容值得保留、迁移或提炼。

当前公开版本定位为 `v0.2.1 Public Preview`：

- Windows 11 已完成桌面调试构建、功能链路和 E2E 验证
- Linux / WSL 当前以发现、解析、审计和数据模型能力预览为主
- 这是一个可公开发布、可继续迭代吸收生态能力的首个公开预览版本，而不是安装包和跨平台实机验证都已收口的稳定版

首期目标聚焦：

- 识别本地安装的 `Codex`、`Claude Code`、`OpenCode`，并为后续扩展更多助手预留适配器接口
- 读取原生会话记录、配置文件、日志和关键运行元数据
- 在网页/桌面界面中展示真实主题、进度、价值密度、风险等级和最后活跃时间
- 在删除前先导出高价值会话为 Markdown，并支持归档、迁移、软删除和审计日志
- 默认对敏感配置和 API key 做脱敏处理，只在显式授权下进行更深层查看或修改

仓库内同时保留了调研、设计、吸收和发布材料：

- [竞品与数据源分析](docs/research/2026-03-15-agent-session-landscape.md)
- [上游 intake 索引](docs/research/upstreams/index.md)
- [完整设计方案](docs/plans/2026-03-15-open-session-manager-design.md)
- [实施计划](docs/plans/2026-03-15-open-session-manager.md)

## 当前已实现的产品能力

- 发现与解析：Rust 核心已经具备安装/路径发现、Win11 / Linux / WSL 路径模型、Codex / Claude Code / OpenCode 会话解析
- 洞察与治理：可以提取真实标题、摘要、进度、价值信号、垃圾会话信号、风险标记和最后活跃时间
- 配置审计：已支持配置与凭据审计、第三方中转检测、危险权限 / shell hook / 宽松审批等风险识别，并默认脱敏显示敏感项
- 导出与清理：已支持 Markdown 导出、软删除、恢复与审计日志链路，且“先导出 Markdown，再移入隔离区”现在是前后端双重守卫
- Transcript 细节：Session Detail 已接入 transcript highlights 与 todo snapshot，并修复 Claude `TodoWrite` 提取
- UI 与桌面壳：Web / Tauri 桌面控制台已经具备总览页、Session Explorer、Session Detail、Config Center、Audit Center 的初版界面
- 可用性：已支持中英文双语文案、按浏览器语言自动选择默认语言、响应式详情布局、更稳定的显式选中与筛选回退
- 本地数据链路：已支持真实本地 snapshot、持久化审计数据库、桌面调试构建与浏览器 fallback 开发流

## 当前已吸收的开源能力

已完成“真实吸收并落地”的上游能力：

- `daaain/claude-code-log`
  - 已吸收：更丰富的 Markdown 导出分节、transcript highlights、Claude todo snapshot
  - 当前对应能力：更像资产沉淀而不是纯文本转存，适合删除前保留核心价值
- `d-kimuson/claude-code-viewer`
  - 已吸收：viewer 风格 transcript detail 面板、session todo evidence 细节展示、Claude transcript todo 提取思路
  - 当前对应能力：会话详情可直接展示更真实的上下文、待办快照和进度痕迹

已完成“研究建档但尚未代码级吸收”的方向性来源：

- `jazzyalex/agent-sessions`：本地索引、搜索、analytics 方向
- `lulu-sk/CodexFlow`：Windows / WSL / 多助手工作区方向
- `Dimension-AI-Technologies/Entropic`：provider-aware 治理控制台方向
- `yoavf/ai-sessions-mcp`：后续 headless / MCP 暴露方向
- `Dicklesworthstone/coding_agent_session_search`：只做 reference-only 宽覆盖研究，不直接吸收代码

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
- Windows release 构建现在使用 GUI 子系统，不再额外弹出终端窗口
- 前端优先通过 Tauri 命令读取真实本地 snapshot
- 导出 Markdown 与软删除动作可直接调用 Rust 原生命令，并返回最新 snapshot
- 调试构建已在当前 Windows 11 环境验证通过
- Sessions 工作区已支持更稳定的显式选中、筛选回退和响应式详情布局

## 发布证据

当前 `v0.2.1 Public Preview` 的本地验证证据包括：

- `cargo test --lib`
- `cargo test --test cli_snapshot`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `npm --prefix web run test`
- `npm --prefix web run build`
- `npm --prefix web run e2e`
- `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`

## 项目结构

```text
src-tauri/      Rust 核心、Tauri 运行时、快照与清理动作
third_party/    上游 catalog、intake manifest 与本地镜像规划
web/            React 控制台、i18n、前端测试与桌面桥接
tests/fixtures/ 多助手真实结构夹具
tests/e2e/      Playwright 端到端验证
docs/research/  竞品调研与数据源分析
docs/plans/     设计方案、实施计划、发布就绪计划
docs/release/   支持矩阵与发布材料
scripts/        snapshot 导出、验证入口、Tauri 启动辅助
```

## 竞品吸收与上游治理

这一轮已经把“竞品研究”落成可执行流程，而不是只停留在文档摘要：

- 权威清单：`third_party/upstreams/catalog.json`
- 生成索引：`docs/research/upstreams/index.md`
- 发布致谢：`docs/release/open-source-attribution.md`
- 本地镜像规划：`third_party/upstreams/intake-manifest.json`
- 生成脚本：`scripts/intake-upstreams.mjs`

已落地的真实吸收链路：

- 上游：`daaain/claude-code-log`
- 当前已吸收：更丰富的 Markdown 导出分节、transcript highlights、Claude todo snapshot
- 上游参考文件：`claude_code_log/markdown/renderer.py`、`claude_code_log/parser.py`、`claude_code_log/converter.py`
- 上游：`d-kimuson/claude-code-viewer`
- 当前已吸收：viewer 风格 transcript detail 面板、session todo evidence 细节展示
- 上游参考文件：`src/routes/projects/$projectId/session.tsx`、`src/lib/todo-viewer/extractLatestTodos.ts`、`src/server/core/claude-code/functions/parseJsonl.ts`

刷新一次完整 intake 产物：

```bash
node scripts/intake-upstreams.mjs
```

只查看将要写入的文件：

```bash
node scripts/intake-upstreams.mjs --dry-run
```

## 本次公开发布暂不包含

- Linux 桌面实机构建与真实目录回归证据闭环
- 配置修改 / 删除后写回真实文件的完整 UI
- 隔离区列表、批量恢复和完整恢复工作流 UI
- MSI / AppImage / deb / 签名产物链路
- WSL companion collector 闭环

更完整的范围界定见：

- [支持矩阵](docs/release/support-matrix.md)
- [发布就绪实施计划](docs/plans/2026-03-15-release-readiness.md)

## 发布后优先项

以下四项按你的要求留到发布之后继续推进：

- 真实 Win11 + WSL 多发行版样本回放
- 符号链接 / 目录联接逃逸专项测试
- 超大历史库性能压测
- 发布包 smoke test

## 致谢

以下开源项目为 OSM 的产品方向、数据模型或交互设计提供了重要启发：

- `jazzyalex/agent-sessions`
- `lulu-sk/CodexFlow`
- `d-kimuson/claude-code-viewer`
- `daaain/claude-code-log`
- `Dimension-AI-Technologies/Entropic`
- `yoavf/ai-sessions-mcp`
- `Dicklesworthstone/coding_agent_session_search`

当前仓库已经把这些项目的吸收姿态、许可证边界和发布致谢收口到以下文件：

- [上游研究索引](docs/research/upstreams/index.md)
- [开源致谢](docs/release/open-source-attribution.md)
- [claude-code-log 吸收记录](docs/research/upstreams/daaain-claude-code-log.md)

OSM 会持续吸收这些项目中适合公开继承的实现与协议兼容思路，并在发布说明中明确标注来源、许可证与致谢信息。
