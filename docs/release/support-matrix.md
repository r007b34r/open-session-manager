# 支持矩阵

日期：2026-03-22
发布：`v0.3.0 Public Preview`

## 本次公开发布范围

| 维度 | 当前状态 | 说明 |
| --- | --- | --- |
| `Codex` 会话解析 | 已实现 | 支持 JSONL rollout，且已修复脚手架标题污染 |
| `Claude Code` 会话解析 | 已实现 | 支持 `~/.claude/projects/**/*.jsonl`，已跳过纯 `file-history-snapshot`，并可按 UUID 文件名自愈缺失 `sessionId` 的历史 JSONL |
| `OpenCode` 会话解析 | 已实现 | 支持本地 storage 结构 |
| `Gemini CLI` 会话解析 | 已实现 | 支持 `~/.gemini/tmp/**/session-*.json` |
| `GitHub Copilot CLI` 会话解析 | 已实现 | 支持 `~/.copilot/session-state/*.jsonl` |
| `Factory Droid` 会话解析 | 已实现 | 支持 session store 与 stream-json 两类格式 |
| `OpenClaw` 会话解析 | 已实现 | 支持 `~/.openclaw` 与兼容旧目录 |
| `Codex` 配置审计 | 已实现 | 支持用户级配置发现与风险审计 |
| `Claude Code` 配置审计 | 已实现 | 支持 `settings.json`、shell hook、权限风险 |
| `OpenCode` 配置审计 | 已实现 | 支持 provider/base URL/权限风险 |
| `Gemini CLI` 配置审计 | 已实现 | 支持 `settings.json`、同级 `.env`、auth mode、base URL、model 与 MCP 线索 |
| `GitHub Copilot CLI` 配置审计 | 已实现 | 支持用户级 `~/.copilot/config.json` + `mcp-config.json`，并能按会话派生项目级 `.github/copilot/settings.json/settings.local.json` |
| `Factory Droid` 配置审计 | 已实现 | 支持用户级与项目级 `settings.json/settings.local.json` 合并视图，展示 provider/base URL、command allowlist、MCP 与 masked key 风险 |
| `OpenClaw` 配置审计 | 已实现 | 支持 `openclaw.json`、provider/base URL、default model 与 tools profile 风险 |
| `GitHub Copilot CLI / Factory Droid / Gemini CLI / OpenClaw` 安全写回 | 已实现 | 支持可视化编辑、输入校验、备份 manifest、回滚测试与审计事件 |
| Transcript digest / todo snapshot | 已实现 | 支持 transcript highlights、Claude todo 提取与会话详情展示 |
| Markdown 导出 | 已实现 | Rust actions 已有测试覆盖 |
| Session handoff Markdown 导出 | 已实现 | 导出包含 `Next focus`、`Open tasks`、`Completed tasks`、`Resume cue` |
| Cleanup checklist / session-end hooks | 已实现 | 导出会同步生成 `cleanup-<session>.json`，并在项目内存在 `session-end.ps1/.sh` 时执行 hook；软删除前要求 checklist 已成功落地且 hook 未失败 |
| 软删除 / 恢复 | 已实现 | Rust actions 已有测试覆盖，恢复边界已加固 |
| 审计事件写入 | 已实现 | 当前覆盖导出、cleanup checklist、session-end hook、软删除、恢复 |
| Environment doctor / health checks | 已实现 | `doctor` CLI 与总览诊断面板会显示被跳过的 malformed session 文件，以及已知根目录下被静默过滤的未知 session-like 文件 |
| Metadata repair / self-healing | 已实现 | `Claude Code` 历史 JSONL 缺失 `sessionId` 时会优先尝试按 UUID 文件名恢复，无法恢复的才进入 `doctor` 诊断 |
| Session index cache / incremental reindex | 已实现 | snapshot 会把索引结果落到 SQLite，按 `assistant + environment + source_path + size + modified_at` 复用缓存，并记录 `cache_hits / cache_misses / reindexed_files / stale_deleted` |
| Real session resume / continue | 部分实现 | 当前已接 `Codex` 与 `Claude Code`，可执行真实 `resume` / `continue` 命令，并写回控制状态与审计事件 |
| One-click resume in Web detail | 部分实现 | 详情页已接恢复按钮、继续提示和最近控制结果；纯浏览器模式不会伪装成本机可控 |
| Git worktree lifecycle CLI | 已实现 | `node scripts/git-worktree-manager.mjs` 支持在仓库内 `.worktrees/` 下执行 `create / merge / delete / recycle` |
| Fixture drift ledger | 已实现 | `tests/fixtures/fixture-ledger.json` 记录 fixture 版本、来源和文件 hash，`scripts/fixture-ledger.mjs --check` 已进入统一 verify |
| Fixture snapshot golden diff | 已实现 | `tests/fixtures/dashboard-snapshot.golden.json` 提供规范化 snapshot 基线，`scripts/check-fixture-snapshot.mjs` 会输出 JSON 路径级 diff |
| Responsive detail panes | 已实现 | 会话详情改为非 sticky、单列 detail card 布局，并有 E2E 覆盖高 DPI、窄窗和嵌入式选中场景 |
| Web / Tauri UI | 已实现 | 桌面端与浏览器端共用同一套 React UI |
| 中英文切换 | 已实现 | 默认按系统或浏览器语言自动切换 |
| 浅色 / 深色 / 跟随系统主题 | 已实现 | 支持手动覆盖并持久化主题选择 |
| Markdown 导出目录设置 | 已实现 | 可在界面里修改导出目录，并显示当前落盘路径 |
| 会话列表辨识度增强 | 已实现 | 列表显示会话 ID，便于区分相近标题 |
| 真实 snapshot CLI | 已实现 | `cargo run -- snapshot` |
| 前端真实 snapshot 优先加载 | 已实现 | 失败时回退到 fixture |
| Usage / cost analytics | 已实现 | `Codex / Claude Code / OpenCode / Gemini CLI / OpenClaw` 已展示会话级和总览级 token/cost 汇总、本地价格目录估算、`reported / estimated / unknown` 成本来源和日级 usage timeline |
| 会话搜索结果排序与片段 | 已实现 | Sessions 页支持本地 BM25 风格 lexical 搜索、命中片段和来源标签 |
| upstream intake pipeline | 已实现 | 支持 catalog、研究索引、发布致谢和镜像规划产物 |

## 平台支持

| 平台 | 当前状态 | 说明 |
| --- | --- | --- |
| Windows 11 | 已验证 | `cargo test -- --test-threads=1`、`cargo test --test cli_snapshot`、`node --test tests/git-workflow/git-worktree-manager.test.mjs`、`npm --prefix web run test`、`npm --prefix web run e2e`、`npm --prefix web run build`、`powershell -ExecutionPolicy Bypass -File scripts/verify.ps1` 已在当前环境通过 |
| Linux | 核心能力预览 | 会话 / 配置路径模型与 fixtures 已覆盖，桌面构建尚未完成实机验证 |
| WSL | 能力预览 | 当前有路径模型、发现表达和 UI 展示，尚未完成 companion collector |

## 当前已吸收的开源能力

| 上游项目 | 当前状态 | 已落地内容 |
| --- | --- | --- |
| `jazzyalex/agent-sessions` | 已吸收 | `Gemini CLI`、`GitHub Copilot CLI`、`Factory Droid`、`OpenClaw` 会话适配器已落进 OSM clean-room 实现 |
| `daaain/claude-code-log` | 已吸收 | 更丰富的 Markdown 导出分节、transcript highlights、Claude todo snapshot |
| `d-kimuson/claude-code-viewer` | 已吸收 | viewer 风格 transcript detail 面板、session todo evidence 展示 |
| `ChristopherA/claude_code_tools` | 已吸收 | session handoff brief、cleanup checklist、session-end hook 这条会话收尾链路已落进 OSM 的导出与软删除守卫 |
| `kbwo/ccmanager` | 已吸收 | repo-local worktree 生命周期管理方向，当前已落成 OSM 的 clean-room `git-worktree-manager` CLI |
| `farion1231/cc-switch` | 已吸收 | 统一 provider/config 治理面板方向，以及 `Gemini CLI / OpenClaw` 配置路径、auth mode、provider/base URL 风险审计思路已落进 OSM clean-room 实现 |
| `endorhq/rover` | 已吸收 | 为 `GitHub Copilot CLI` companion `mcp-config.json` 路径与治理边界提供了 clean-room 参考 |
| `junhoyeo/tokscale` | 已吸收 | 本地 usage / token / cost 聚合面板、成本来源标注、本地价格目录估算与 usage timeline 已落进 OSM clean-room 实现 |
| `yoavf/ai-sessions-mcp` | 已吸收 | 已落地本地搜索排序、片段和命中来源；MCP `list/search/get` 仍在后续计划里 |
| `coder/agentapi` | 已研究 | HTTP / SSE 控制层方向 |
| `sugyan/claude-code-webui` | 已研究 | 轻量远程壳层、plan mode / permission mode、history loader 方向 |
| `ssdeanx/Gemini-CLI-Web` | Reference-only | 远程 Gemini 工作台样本，但许可证口径冲突，暂不吸收代码 |
| `vultuk/claude-code-web` | 已研究 | 远程浏览器访问和多会话持久化方向 |
| `Dicklesworthstone/coding_agent_session_search` | Reference-only | 只吸收搜索与覆盖面启发，不直接复制代码 |

## 当前不在 `v0.3.0` 承诺范围内

| 项目 | 当前状态 | 说明 |
| --- | --- | --- |
| BM25 / 语义搜索 / search API | 未纳入本版承诺 | 当前已实现本地加权搜索预览，但还没有后台索引和 API 暴露 |
| 更广助手的会话控制 / attach / process control | 未纳入本版承诺 | 当前真实控制只覆盖 `Codex / Claude Code`，还没有统一的 attach/detach、pause/resume 与进程观测层 |
| worktree 编排 / 多项目调度 | 未纳入本版承诺 | 已有基础 worktree lifecycle CLI，但还没有调度器、任务队列和容器隔离层 |
| provider presets / 共享配置片段 / 健康探测 | 未纳入本版承诺 | 当前已支持 `GitHub Copilot CLI / Factory Droid / Gemini CLI / OpenClaw` 的安全写回，但还没有预设编排和健康切换 |
| 高级 analytics / 更宽连接器 | 未纳入本版承诺 | 当前已完成本地价格目录估算、成本来源标注与日级 usage timeline，但还没有 model/platform breakdown、contribution graph、shareable stats 与更宽连接器覆盖 |
| Linux 桌面实机回归 | 未纳入本版承诺 | 当前没有 Linux 环境下的 Tauri 构建与真实助手目录回归证据 |
| 发布安装包与签名 | 未纳入本版承诺 | 目前以源码仓库与本地构建产物为主，没有 MSI / AppImage / deb / 签名流程 |

## 发布后优先项

- 真实 Win11 + WSL 多发行版样本回放
- 符号链接 / 目录联接逃逸专项测试
- 超大历史库性能压测
- 发布包 smoke test
- 搜索、配置写回和恢复能力线的下一阶段集成
