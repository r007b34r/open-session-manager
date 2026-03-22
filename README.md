# open Session Manager

`open Session Manager`，简称 `OSM`。

OSM 不是聊天壳，也不是另一个 agent launcher。它做的是另一件更实际的事：把你机器上已经存在的终端代码助手会话、配置和清理动作梳理清楚，让你能判断什么该保留，什么该导出，什么该隔离，什么只是垃圾历史。

当前版本已经有一条清晰主线：

- 发现本地会话
- 抽取真实主题、摘要、进度、风险和最后活跃时间
- 把会话索引缓存到本地 SQLite，并按文件变化做增量更新
- 先导出带 handoff 的 Markdown，再决定是否移入隔离区
- 审计配置里的 provider、base URL、权限放大、中转和敏感键
- 在本地快照里做加权搜索，给出命中片段和来源标签
- 对已支持的助手直接恢复会话，或补一条继续提示再落审计
- 汇总支持助手的 token、cost 和 usage 信号
- 把导出、隔离、恢复动作写进本地审计历史
- 把 Git worktree 的创建、复用、合并和删除脚本化

## 当前状态

- 版本：`v0.3.0 Public Preview`
- Windows 11：桌面运行、真实本地 snapshot、增量索引、恢复入口、导出、软删除守卫，以及统一 verify 已打通
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
| `GitHub Copilot CLI` | 已支持 | 识别用户级 `~/.copilot/config.json`，联动 `mcp-config.json`，并可按会话发现项目级 `.github/copilot/settings.json/settings.local.json` |
| `Factory Droid` | 已支持 | 识别用户级 `settings.json/settings.local.json`，并可按会话发现项目级 `.factory/settings.json/settings.local.json` |
| `OpenClaw` | 已支持 | 识别 `openclaw.json` 的 provider、default model、tools profile 与 key 风险 |

## 本版真实改进

- 会话支持面从 3 个扩到 7 个，不再只停在 `Codex / Claude Code / OpenCode`
- `Codex` 会话不再把 `AGENTS.md`、`permissions instructions`、`environment_context` 误当主题
- `Claude Code` 发现阶段会跳过纯 `file-history-snapshot` JSONL，且对缺失 `sessionId` 但文件名是 UUID 的历史 JSONL 会先尝试自愈，再决定是否诊断
- 已知会话根目录里那些原本会被静默过滤的未知 session-like 文件，现在也会进入 `doctor`，不会再无声消失
- snapshot 现在会把 session index cache 落到 SQLite，并记录 cache hit / miss / reindex / stale prune 统计
- Sessions 列表现在会显示会话 ID，多个相近标题不再像同一条
- 会话详情改成非 sticky、单列 detail card 布局，窄窗、高 DPI 和嵌入式选中时不再被拉伸
- `Codex` 和 `Claude Code` 现在支持真实 resume / continue，结果会写回会话控制状态和审计事件
- Web 详情页现在有一键恢复、继续提示、最近一次控制结果和导出落盘路径提示
- fixtures 现在有版本、来源和文件 hash ledger，统一 verify 会直接检查 drift
- fixtures snapshot 现在也有规范化 golden 基线，漂移时会直接返回具体 JSON 路径 diff
- 配置审计已经扩到 7 个助手，`GitHub Copilot CLI / Factory Droid` 现在也能显示真实 `model`、endpoint、风险标记和脱敏凭据
- `GitHub Copilot CLI / Factory Droid` 现在会按会话 `projectPath` 派生项目级配置，项目覆盖层也能直接出现在配置面板里
- 首页和会话详情现在会展示 `Codex / Claude Code / OpenCode / Gemini CLI / OpenClaw` 的 usage / cost 汇总
- Sessions 搜索现在会做本地 BM25 风格 lexical 排序，显示命中片段并标明命中来源
- 支持中英文切换，默认按系统或浏览器语言决定
- 支持 `跟随系统 / 浅色 / 深色` 三种主题模式
- 支持在界面里修改 Markdown 导出目录，并在导出后明确显示落盘路径
- Markdown 导出现在会额外生成 `Session Handoff`，把 `Next focus / Open tasks / Resume cue` 一起留下来
- Markdown 导出现在还会同步生成 `cleanup-<session>.json`，并在项目里存在 `session-end` hook 时执行它；软删除不再只看 Markdown 导出，而是要求 cleanup checklist 先落地
- 仓库里新增 `git-worktree-manager` CLI，统一处理 `.worktrees/` 下的 create / merge / delete / recycle
- 新增三条本地镜像研究：`ChristopherA/claude_code_tools`、`ssdeanx/Gemini-CLI-Web`、`sugyan/claude-code-webui`

## 现在能拿它做什么

- 看清哪些旧会话还有价值，哪些只是污染历史
- 在删除前先导出成 Markdown，把下一步、未完成项和恢复线索一起留下来
- 在删除前先生成 cleanup checklist，并把项目级 `session-end` hook 的执行结果一起记入本地审计
- 找出代理配置里的第三方中转、危险权限、宽审批和 shell hook
- 直接在界面里修改 `GitHub Copilot CLI / Factory Droid / Gemini CLI / OpenClaw` 配置，并保留备份、回滚和审计痕迹
- 通过环境诊断面板和 `doctor` 命令发现被跳过的坏会话文件，而不是靠终端噪声猜测
- 对 `Codex` 和 `Claude Code` 会话直接执行恢复，或补一条继续提示并保留最近一次控制结果
- 对 `Claude Code` 一类历史 JSONL 缺失 `sessionId` 的情况，优先自动修复；只有无法恢复的坏文件才继续进入 `doctor`
- 对已知根目录中的未知 session-like 文件，`doctor` 现在也会明确报出路径和助手来源，方便后续补适配器
- 用 `tests/fixtures/fixture-ledger.json` 跟踪 fixture 版本、来源和 hash，避免 fixture 静默漂移把回归基线拖偏
- 用 `tests/fixtures/dashboard-snapshot.golden.json` 固定 fixture snapshot 结构，回归失败时可以直接看到是哪个 JSON 路径变了
- 用 `node scripts/git-worktree-manager.mjs` 在仓库内 `.worktrees/` 目录创建、复用、合并和删除工作树
- 用 `cargo run -- list/search/get/view/expand` 直接查会话清单、搜索命中、详情、Markdown 视图和上下文 bundle
- 看最近谁做过导出、隔离、恢复，动作有没有留痕
- 在桌面窗口或浏览器调试环境里统一查看这些信息

## 当前还没完成的部分

- 语义搜索、hybrid ranking、统一 search API 和更大样本下的性能压测
- 除 `Codex / Claude Code` 之外的真实会话控制、attach/detach、pause/resume 和运行态进程观测
- 建立在 worktree CLI 之上的多项目调度、并行 agent 编排、容器隔离执行
- provider presets、共享配置片段、健康探测和自动切换
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
  已吸收：session closure / resume brief、cleanup checklist、session-end hook 这条会话收尾思路，已落进 OSM 的 Markdown 导出与软删除守卫
- `farion1231/cc-switch`
  已吸收：统一 provider/config 治理面板的产品方向，以及 `Gemini CLI / OpenClaw` 配置路径、auth mode、provider/base URL 风险审计思路，已重写为 OSM 自己的实现
- `kbwo/ccmanager`
  已吸收：仓库内 `.worktrees/` 生命周期管理这条产品线，已落成 OSM 自己的 `create / merge / delete / recycle` clean-room CLI
- `endorhq/rover`
  已吸收：`GitHub Copilot CLI` companion `mcp-config.json` 路径线索，已落进 OSM 的 clean-room 配置审计实现
- `junhoyeo/tokscale`
  已吸收：本地 usage / token / cost 字段模型和多助手聚合面板思路，已重写为 OSM 自己的实现
- `jazzyalex/agent-sessions`、`yoavf/ai-sessions-mcp`
  已吸收：本地搜索结果排序、命中片段、来源标签，以及可复用的本地索引方向；当前已落到 Web 工作台和 SQLite 增量索引缓存

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

浏览器预览：

```bash
npm --prefix web run browser
```

桌面开发：

```bash
npm --prefix web run tauri:dev
```

桌面构建：

```bash
npm --prefix web run tauri:build
```

Git worktree 生命周期：

```bash
node scripts/git-worktree-manager.mjs create --repo-root . --branch feature/demo --base main
node scripts/git-worktree-manager.mjs recycle --repo-root . --branch feature/demo --base main
node scripts/git-worktree-manager.mjs merge --repo-root . --branch feature/demo --into main
node scripts/git-worktree-manager.mjs delete --repo-root . --branch feature/demo
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

统一会话查询 CLI：

```bash
cargo run --manifest-path src-tauri/Cargo.toml -- list --fixtures tests/fixtures
cargo run --manifest-path src-tauri/Cargo.toml -- search --fixtures tests/fixtures --query Claude
cargo run --manifest-path src-tauri/Cargo.toml -- get --fixtures tests/fixtures --session claude-ses-1
cargo run --manifest-path src-tauri/Cargo.toml -- view --fixtures tests/fixtures --session claude-ses-1
cargo run --manifest-path src-tauri/Cargo.toml -- expand --fixtures tests/fixtures --session claude-ses-1
```

Git worktree 生命周期：

```bash
node --test tests/git-workflow/git-worktree-manager.test.mjs
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
