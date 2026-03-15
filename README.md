# open Session Manager

`open Session Manager`，简称 `OSM`。

这是一个本地工具，用来整理终端编程助手留下的会话和配置。它的重点不是“聊天”，而是把你机器上已经存在的历史记录、配置风险和清理动作先看清，再决定哪些该保留、哪些该导出、哪些该隔离。

当前仓库主打这几件事：

- 扫描 `Codex`、`Claude Code`、`OpenCode` 的本地会话和配置
- 提取会话标题、摘要、进度、价值分、风险标记、最后活跃时间
- 导出高价值会话为 Markdown，再决定是否移入隔离区
- 审计代理配置里的 provider、base URL、权限放大和第三方中转痕迹
- 记录导出、隔离、恢复这些动作的历史

## 当前状态

- Windows 11：桌面运行、真实本地快照、Markdown 导出、软删除守卫、前端测试和 E2E 都已打通
- Linux / WSL：发现、解析、路径模型和配置审计已做，桌面实机验证还没收口
- 当前版本仍然是公开预览，不是已经封版的稳定版

## 现在能直接看到的功能

- 会话列表整行可点，不再只剩标题按钮有响应
- 导出后会明确显示 Markdown 文件路径
- 可以在界面里直接修改 Markdown 导出目录，并恢复默认目录
- 支持 `跟随系统 / 浅色 / 深色` 三种主题模式
- 首页会直接展示已经吸收的上游能力，不再只藏在研究文档里
- 会话详情已改成更稳的自适应布局，避免右侧详情被拉成长条

## 已支持的助手

| 助手 | 会话 | 配置 | 说明 |
| --- | --- | --- | --- |
| `Codex` | 已支持 | 已支持 | 识别本地 JSONL rollout 和用户级配置 |
| `Claude Code` | 已支持 | 已支持 | 识别 `projects/**/*.jsonl` 和 `settings.json` |
| `OpenCode` | 已支持 | 已支持 | 识别本地 storage 结构和全局配置 |

## 可以用它做什么

- 找出哪些旧会话只是垃圾历史，哪些值得继续跟进
- 删除前先导出成 Markdown，把真正有用的上下文保下来
- 检查代理配置里有没有第三方中转、危险权限、宽审批、shell hook
- 看最近谁做过导出、隔离、恢复，动作有没有留痕
- 用浏览器界面或桌面窗口统一查看这些信息

## 暂时还没有的东西

- Linux 桌面实机回归证据
- 完整的隔离区浏览和恢复 UI
- 直接改写真实配置文件的编辑 UI
- MSI / AppImage / deb / 签名发布链路
- 真实 Win11 + WSL 多发行版样本回放和大库性能压测

## 本地运行

依赖：

- Rust toolchain
- Node.js 20+

安装前端依赖：

```bash
npm --prefix web install
```

浏览器开发：

```bash
npm --prefix web run dev
```

桌面开发：

```bash
npm --prefix web run tauri:dev
```

桌面构建：

```bash
npm --prefix web run tauri:build
```

也可以直接走仓库脚本：

```bash
node scripts/run-tauri.mjs dev
node scripts/run-tauri.mjs build --debug
```

## 默认存储位置

### Windows

- 审计数据库：`%LOCALAPPDATA%/OpenSessionManager/audit/audit.db`
- 偏好设置：`%LOCALAPPDATA%/OpenSessionManager/preferences.json`
- 隔离区：`%LOCALAPPDATA%/OpenSessionManager/quarantine/`
- Markdown 导出：`%USERPROFILE%/Documents/OpenSessionManager/exports/`

### Linux

- 审计数据库：`$XDG_DATA_HOME/open-session-manager/audit/audit.db`
- 偏好设置：`$XDG_DATA_HOME/open-session-manager/preferences.json`
- 隔离区：`$XDG_DATA_HOME/open-session-manager/quarantine/`
- Markdown 导出：`$HOME/Documents/OpenSessionManager/exports/`

如果你在界面里改了导出目录，`OSM` 会把覆盖值写进偏好设置文件。

## 测试和验证

Rust 单测：

```bash
cargo test --lib
```

前端单测：

```bash
npm --prefix web run test
```

前端构建：

```bash
npm --prefix web run build
```

Playwright E2E：

```bash
npx --prefix web playwright install chromium
npm --prefix web run e2e
```

统一验证入口：

```bash
powershell -ExecutionPolicy Bypass -File scripts/verify.ps1
```

## 快照来源顺序

前端当前按下面顺序取数据：

1. Tauri 原生命令
2. `web/public/dashboard-snapshot.json`
3. `web/src/lib/api.ts` 里的内置 typed fixture

如果要把 Rust 生成的真实快照导给浏览器开发环境：

```bash
node scripts/export-dashboard-snapshot.mjs
```

如果要用 fixtures 生成演示快照：

```bash
node scripts/export-dashboard-snapshot.mjs --fixtures tests/fixtures
```

## 已吸收的上游能力

当前已经落到产品里的两条吸收链路：

- `daaain/claude-code-log`
  - 已吸收：更完整的 Markdown 分节、transcript highlights、Claude todo snapshot
- `d-kimuson/claude-code-viewer`
  - 已吸收：viewer 风格详情面板、todo evidence 展示、Claude transcript todo 提取思路

仍在持续研究但还没正式吸收进代码的方向：

- `jazzyalex/agent-sessions`
- `lulu-sk/CodexFlow`
- `Dimension-AI-Technologies/Entropic`
- `yoavf/ai-sessions-mcp`
- `Dicklesworthstone/coding_agent_session_search`（只做 reference-only 研究，不直接拷贝代码）

相关资料在这些文件里：

- [上游研究索引](docs/research/upstreams/index.md)
- [开源致谢](docs/release/open-source-attribution.md)
- [设计方案](docs/plans/2026-03-15-open-session-manager-design.md)
- [发布边界](docs/release/support-matrix.md)

## 项目结构

```text
src-tauri/      Rust 核心、Tauri 运行时、会话动作和快照
web/            React UI、i18n、浏览器 fallback、前端测试
tests/fixtures/ 多助手夹具
tests/e2e/      Playwright 端到端测试
third_party/    上游 catalog、intake 资料和边界说明
docs/           设计、研究、发布说明
scripts/        构建、验证、快照导出脚本
```
