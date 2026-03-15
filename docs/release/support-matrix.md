# 支持矩阵

日期：2026-03-16
发布：`v0.3.0 Public Preview`

## 本次公开发布范围

| 维度 | 当前状态 | 说明 |
| --- | --- | --- |
| `Codex` 会话解析 | 已实现 | 支持 JSONL rollout，且已修复脚手架标题污染 |
| `Claude Code` 会话解析 | 已实现 | 支持 `~/.claude/projects/**/*.jsonl`，已跳过纯 `file-history-snapshot` |
| `OpenCode` 会话解析 | 已实现 | 支持本地 storage 结构 |
| `Gemini CLI` 会话解析 | 已实现 | 支持 `~/.gemini/tmp/**/session-*.json` |
| `GitHub Copilot CLI` 会话解析 | 已实现 | 支持 `~/.copilot/session-state/*.jsonl` |
| `Factory Droid` 会话解析 | 已实现 | 支持 session store 与 stream-json 两类格式 |
| `OpenClaw` 会话解析 | 已实现 | 支持 `~/.openclaw` 与兼容旧目录 |
| `Codex` 配置审计 | 已实现 | 支持用户级配置发现与风险审计 |
| `Claude Code` 配置审计 | 已实现 | 支持 `settings.json`、shell hook、权限风险 |
| `OpenCode` 配置审计 | 已实现 | 支持 provider/base URL/权限风险 |
| Transcript digest / todo snapshot | 已实现 | 支持 transcript highlights、Claude todo 提取与会话详情展示 |
| Markdown 导出 | 已实现 | Rust actions 已有测试覆盖 |
| Session handoff Markdown 导出 | 已实现 | 导出包含 `Next focus`、`Open tasks`、`Completed tasks`、`Resume cue` |
| 软删除 / 恢复 | 已实现 | Rust actions 已有测试覆盖，恢复边界已加固 |
| 审计事件写入 | 已实现 | 当前覆盖导出、软删除、恢复 |
| Web / Tauri UI | 已实现 | 桌面端与浏览器端共用同一套 React UI |
| 中英文切换 | 已实现 | 默认按系统或浏览器语言自动切换 |
| 浅色 / 深色 / 跟随系统主题 | 已实现 | 支持手动覆盖并持久化主题选择 |
| Markdown 导出目录设置 | 已实现 | 可在界面里修改导出目录，并显示当前落盘路径 |
| 会话列表辨识度增强 | 已实现 | 列表显示会话 ID，便于区分相近标题 |
| 真实 snapshot CLI | 已实现 | `cargo run -- snapshot` |
| 前端真实 snapshot 优先加载 | 已实现 | 失败时回退到 fixture |
| upstream intake pipeline | 已实现 | 支持 catalog、研究索引、发布致谢和镜像规划产物 |

## 平台支持

| 平台 | 当前状态 | 说明 |
| --- | --- | --- |
| Windows 11 | 已验证 | `cargo test --lib`、`cargo test --test cli_snapshot`、`npm --prefix web run test`、`npm --prefix web run build` 已在当前环境通过 |
| Linux | 核心能力预览 | 会话 / 配置路径模型与 fixtures 已覆盖，桌面构建尚未完成实机验证 |
| WSL | 能力预览 | 当前有路径模型、发现表达和 UI 展示，尚未完成 companion collector |

## 当前已吸收的开源能力

| 上游项目 | 当前状态 | 已落地内容 |
| --- | --- | --- |
| `jazzyalex/agent-sessions` | 已吸收 | `Gemini CLI`、`GitHub Copilot CLI`、`Factory Droid`、`OpenClaw` 会话适配器已落进 OSM clean-room 实现 |
| `daaain/claude-code-log` | 已吸收 | 更丰富的 Markdown 导出分节、transcript highlights、Claude todo snapshot |
| `d-kimuson/claude-code-viewer` | 已吸收 | viewer 风格 transcript detail 面板、session todo evidence 展示 |
| `ChristopherA/claude_code_tools` | 已吸收 | session handoff brief 思路已落进 OSM 的 Markdown 导出 |
| `kbwo/ccmanager` | 已研究 | worktree / 多项目调度方向 |
| `farion1231/cc-switch` | 已研究 | provider / MCP / prompts / skills 治理方向 |
| `junhoyeo/tokscale` | 已研究 | token / cost analytics 方向 |
| `yoavf/ai-sessions-mcp` | 已研究 | MCP / headless 数据暴露方向 |
| `coder/agentapi` | 已研究 | HTTP / SSE 控制层方向 |
| `sugyan/claude-code-webui` | 已研究 | 轻量远程壳层、plan mode / permission mode、history loader 方向 |
| `ssdeanx/Gemini-CLI-Web` | Reference-only | 远程 Gemini 工作台样本，但许可证口径冲突，暂不吸收代码 |
| `vultuk/claude-code-web` | 已研究 | 远程浏览器访问和多会话持久化方向 |
| `Dicklesworthstone/coding_agent_session_search` | Reference-only | 只吸收搜索与覆盖面启发，不直接复制代码 |

## 当前不在 `v0.3.0` 承诺范围内

| 项目 | 当前状态 | 说明 |
| --- | --- | --- |
| 全文搜索 / BM25 / 语义搜索 | 未纳入本版承诺 | 当前仍以会话发现、治理和导出为主 |
| 会话恢复 / attach / process control | 未纳入本版承诺 | 当前还没有真实会话进程控制层 |
| worktree 编排 / 多项目调度 | 未纳入本版承诺 | 仍在研究与规格阶段 |
| `Gemini / Copilot / Factory / OpenClaw` 配置审计 | 未纳入本版承诺 | 当前只有会话支持，配置治理尚未接入 |
| token / cost analytics | 未纳入本版承诺 | 仍在研究与设计阶段 |
| Linux 桌面实机回归 | 未纳入本版承诺 | 当前没有 Linux 环境下的 Tauri 构建与真实助手目录回归证据 |
| 发布安装包与签名 | 未纳入本版承诺 | 目前以源码仓库与本地构建产物为主，没有 MSI / AppImage / deb / 签名流程 |

## 发布后优先项

- 真实 Win11 + WSL 多发行版样本回放
- 符号链接 / 目录联接逃逸专项测试
- 超大历史库性能压测
- 发布包 smoke test
- 搜索与恢复能力线的下一阶段集成
