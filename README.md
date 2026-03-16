# open Session Manager

`open Session Manager`，简称 `OSM`。

OSM 不是聊天壳，也不是另一个 agent launcher。它做的是另一件更实际的事：把你机器上已经存在的终端代码助手会话、配置和清理动作梳理清楚，让你能判断什么该保留，什么该导出，什么该隔离，什么只是垃圾历史。

当前版本已经有一条清晰主线：

- 发现本地会话
- 抽取真实主题、摘要、进度、风险和最后活跃时间
- 先导出带 handoff 的 Markdown，再决定是否移入隔离区
- 审计配置里的 provider、base URL、权限放大、中转和敏感键
- 在本地快照里做加权搜索，给出命中片段和来源标签
- 汇总支持助手的 token、cost 和 usage 信号
- 把导出、隔离、恢复动作写进本地审计历史

## 当前状态

- 版本：`v0.3.0 Public Preview`
- Windows 11：桌面运行、真实本地 snapshot、导出、软删除守卫、前端测试和 Rust 测试已打通
- Linux / WSL：发现与路径模型可用，但还没完成完整桌面实机验证
- 当前仍是公开预览，不是稳定版

## 已支持的助手

### 会话支持

| 助手 | 状态 | 说明 |
| --- | --- | --- |
| `Codex` | 已支持 | 识别本地 JSONL rollout |
| `Claude Code` | 已支持 | 识别 `~/.claude/projects/**/*.jsonl` |
| `OpenCode` | 已支持 | 识别本地 storage 结构 |
| `Gemini CLI` | 已支持 | 识别 `~/.gemini/tmp/**/session-*.json` |
| `GitHub Copilot CLI` | 已支持 | 识别 `~/.copilot/session-state/*.jsonl` |
| `Factory Droid` | 已支持 | 识别 session store 与 stream-json 两类方言 |
| `OpenClaw` | 已支持 | 识别 `~/.openclaw` 与兼容旧目录 |

### 配置审计

| 助手 | 状态 | 说明 |
| --- | --- | --- |
| `Codex` | 已支持 | 识别用户级配置与风险项 |
| `Claude Code` | 已支持 | 识别 `settings.json`、shell hook、权限风险 |
| `OpenCode` | 已支持 | 识别 provider/base URL/权限风险 |
| `Gemini CLI` | 已支持 | 识别 `settings.json`、同级 `.env`、auth mode、base URL、model 与 MCP 线索 |
| `GitHub Copilot CLI` | 规划中 | 尚未接配置审计 |
| `Factory Droid` | 规划中 | 尚未接配置审计 |
| `OpenClaw` | 已支持 | 识别 `openclaw.json` 的 provider、default model、tools profile 与 key 风险 |

## 本版真实改进

- 会话支持面从 3 个扩到 7 个，不再只停在 `Codex / Claude Code / OpenCode`
- `Codex` 会话不再把 `AGENTS.md`、`permissions instructions`、`environment_context` 误当主题
- `Claude Code` 发现阶段会跳过纯 `file-history-snapshot` JSONL，启动时不再刷这类噪声
- Sessions 列表现在会显示会话 ID，多个相近标题不再像同一条
- `Gemini CLI` 与 `OpenClaw` 配置审计已落地，配置卡片现在能看到真实 `model`
- 首页和会话详情现在会展示 `Codex / Claude Code / OpenCode / Gemini CLI / OpenClaw` 的 usage / cost 汇总
- Sessions 搜索现在会做本地加权排序，显示命中片段并标明命中来源
- 支持中英文切换，默认按系统或浏览器语言决定
- 支持 `跟随系统 / 浅色 / 深色` 三种主题模式
- 支持在界面里修改 Markdown 导出目录，并在导出后明确显示落盘路径
- Markdown 导出现在会额外生成 `Session Handoff`，把 `Next focus / Open tasks / Resume cue` 一起留下来
- 新增三条本地镜像研究：`ChristopherA/claude_code_tools`、`ssdeanx/Gemini-CLI-Web`、`sugyan/claude-code-webui`

## 现在能拿它做什么

- 看清哪些旧会话还有价值，哪些只是污染历史
- 在删除前先导出成 Markdown，把下一步、未完成项和恢复线索一起留下来
- 找出代理配置里的第三方中转、危险权限、宽审批和 shell hook
- 看最近谁做过导出、隔离、恢复，动作有没有留痕
- 在桌面窗口或浏览器调试环境里统一查看这些信息

## 当前还没完成的部分

- 大历史索引、BM25、语义搜索和 hybrid ranking
- 会话恢复、attach、pause/resume、真实进程控制
- worktree 编排、多项目调度、容器隔离执行
- GitHub Copilot CLI / Factory Droid 配置审计
- pricing lookup、usage 趋势图、更多助手用量连接器
- MCP / HTTP / headless 自动化接口
- Linux 桌面实机回归、发布包 smoke test、大库性能压测

## 吸收与致谢

OSM 当前已经把一部分上游能力真实落进代码，不只是停在文档：

- `jazzyalex/agent-sessions`
  已吸收：`Gemini CLI`、`GitHub Copilot CLI`、`Factory Droid`、`OpenClaw` 会话适配思路，已重写为 OSM 自己的实现
- `daaain/claude-code-log`
  已吸收：更完整的 Markdown 导出分节、transcript highlights、todo snapshot
- `d-kimuson/claude-code-viewer`
  已吸收：viewer 风格详情面板和 todo evidence 展示思路
- `ChristopherA/claude_code_tools`
  已吸收：session closure / resume 的 brief 思路，已落进 OSM 的 `Session Handoff` Markdown 导出
- `farion1231/cc-switch`
  已吸收：`Gemini CLI` 与 `OpenClaw` 的配置路径、auth mode、provider/base URL 风险审计思路，已重写为 OSM 自己的实现
- `junhoyeo/tokscale`
  已吸收：本地 usage / token / cost 字段模型和多助手聚合面板思路，已重写为 OSM 自己的实现
- `jazzyalex/agent-sessions`、`yoavf/ai-sessions-mcp`
  已吸收：本地搜索结果排序、命中片段和来源标签这条能力线，当前先落在 Web 工作台，BM25 / MCP 接口仍在后续计划里

本地已镜像并纳入研究的项目远不止这几个，完整列表见：

- [全竞品差距分析](docs/research/2026-03-16-full-competitor-gap-analysis.md)
- [上游研究索引](docs/research/upstreams/index.md)
- [开源致谢](docs/release/open-source-attribution.md)

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

## 验证命令

Rust 单测：

```bash
cargo test --lib
```

snapshot CLI：

```bash
cargo test --test cli_snapshot
```

前端单测：

```bash
npm --prefix web run test
```

前端构建：

```bash
npm --prefix web run build
```

统一验证入口：

```bash
powershell -ExecutionPolicy Bypass -File scripts/verify.ps1
```
