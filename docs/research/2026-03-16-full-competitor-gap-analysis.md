# 2026-03-16 全竞品能力差距总分析

日期：2026-03-16  
工作树：`feat/usability-clarity`  
分析方式：真实本地拉取仓库 + README + 关键源码入口阅读  
目标：列出当前 OSM 相对全部竞品尚未实现的功能，而不是只挑一部分

---

## 1. 本轮实际分析的本地仓库

以下仓库已经真实拉取到 `third_party/upstreams/mirrors/` 并纳入本轮分析：

### 直接竞品

- `jazzyalex/agent-sessions`
- `kbwo/ccmanager`
- `Dimension-AI-Technologies/Entropic`
- `milisp/codexia`
- `farion1231/cc-switch`
- `Dicklesworthstone/coding_agent_session_search`
- `d-kimuson/claude-code-viewer`
- `daaain/claude-code-log`
- `lulu-sk/CodexFlow`
- `yoavf/ai-sessions-mcp`
- `smtg-ai/claude-squad`
- `siteboon/claudecodeui`
- `junhoyeo/tokscale`
- `coder/agentapi`
- `endorhq/rover`
- `kevinelliott/agentpipe`
- `autohandai/commander`
- `pchalasani/claude-code-tools`
- `vultuk/claude-code-web`
- `ssdeanx/Gemini-CLI-Web`
- `sugyan/claude-code-webui`

### 近邻能力仓库

- `udecode/dotai`
- `ChristopherA/claude_code_tools`

`dotai` 不是会话治理台本身，但它在技能、工作流、调试/TDD 插件化方面提供了值得吸收的“方法能力”。

## 2. 当前 OSM 已有能力基线

为了避免“把已有能力也误记成缺口”，先定义当前 OSM 基线：

### 已经真实具备

- `Codex / Claude Code / OpenCode / Gemini CLI / GitHub Copilot CLI / Factory Droid / OpenClaw` 会话发现与解析
- 会话洞察：标题、摘要、进度、价值分、风险标记
- transcript digest：基础 highlights 与 Claude todo 提取
- Markdown 导出
- 软删除 / 恢复
- 配置审计：`Codex / Claude Code / OpenCode / Gemini CLI / OpenClaw`
- 审计事件持久化
- Web/Tauri 桌面 UI
- 中英双语
- 跟随系统语言
- 主题切换
- 导出目录设置
- `Codex` 脚手架标题污染修复
- `Claude Code` file-history-only 会话误收修复

### 当前明显短板

- 配置治理仍缺 `GitHub Copilot CLI / Factory Droid`，且 `Gemini / OpenClaw` 还没进入安全写回
- 缺全文检索 / 跨会话搜索
- 缺恢复/继续/附着真实会话
- 缺活跃会话监控
- 缺 worktree 编排
- 缺 Git 视图
- 缺 token / cost / usage 分析
- 缺 MCP / HTTP / headless 自动化接口
- 缺插件 / 技能 / 提示词治理
- 缺云同步 / 多 profile / 多账号
- 缺远程 / 移动 / 浏览器外部访问

## 3. 逐仓库能力拆解

本节不是营销总结，而是对 OSM 有实际借鉴价值的功能拆解。

### 3.1 `jazzyalex/agent-sessions`

这是当前最接近 OSM 目标的本地会话聚合器。

已确认能力：

- 支持 `Codex CLI / Claude Code / Gemini CLI / GitHub Copilot CLI / Droid / OpenCode / OpenClaw`
- 统一 session indexer
- 统一 transcript 浏览
- 跨会话搜索
- image browsing
- resume 工作流
- active session cockpit
- live activity HUD
- 本地大历史索引
- OpenCode 新旧存储兼容
- schema drift 监控与 fixture 体系

对应 OSM 缺口：

- 缺统一搜索
- 缺 active session cockpit
- 缺 session resume
- 缺 schema drift 监控
- 缺版本跟踪 / fixture 漂移治理
- 缺 OpenCode SQLite 新存储兼容
- `GitHub Copilot CLI / Droid` 配置治理仍未接入
- `Gemini / OpenClaw` 仍缺写回、provider presets、MCP/skills/prompts 治理

### 3.2 `kbwo/ccmanager`

它的重点不是 transcript viewer，而是“多会话 + 多 worktree + 多项目”的调度台。

已确认能力：

- `Claude / Gemini / Codex / Cursor Agent / Copilot / Cline / OpenCode / Kimi`
- 多项目统一管理
- worktree 创建 / 合并 / 删除
- 会话切换
- 状态探测：busy / waiting / idle
- Claude session data 复制到新 worktree
- 命令预设与 fallback
- status hooks / worktree hooks
- devcontainer integration
- auto approval

对应 OSM 缺口：

- 缺多项目工作台
- 缺 worktree 生命周期管理
- 缺会话状态探测
- 缺 session data copy
- 缺 hooks 自动化
- 缺命令预设 / fallback
- 缺 devcontainer/workspace bootstrap
- 缺 auto approval 策略层
- 缺 `Cursor / Cline / Kimi`

### 3.3 `Dimension-AI-Technologies/Entropic`

它更像“治理控制台”，这和 OSM 的方向非常接近。

已确认能力：

- provider-aware dashboard
- per-project TODO monitoring
- user prompt history
- global git view
- commit history view
- provider allow-list
- diagnostics / repair metadata
- unknown session diagnostics
- cleanup/delete actions
- screenshot automation
- 多实现：Electron / PowerShell / Bash

对应 OSM 缺口：

- 缺 live TODO/project monitor
- 缺 provider 级实时切换 / allow-list
- 缺 Git 全局视图
- 缺 commit history
- 缺 metadata repair / self-healing
- 缺 unknown session diagnostics
- 缺自动截图 / 回归快照
- 缺 PowerShell/Bash 辅助模式

### 3.4 `milisp/codexia`

它本质上是 agent IDE/workbench。

已确认能力：

- task scheduler
- headless web server
- WebSocket 实时流
- IDE-like editor
- file tree
- local web preview
- prompt notepad
- PDF / XLSX / CSV preview
- MCP marketplace
- skills marketplace
- usage analytics dashboard
- remote control API

对应 OSM 缺口：

- 缺定时任务 / 自动任务
- 缺 headless web server
- 缺 WebSocket 会话流
- 缺内建编辑器 / 文件树 / 预览
- 缺文档与表格预览
- 缺 MCP marketplace
- 缺 skills marketplace
- 缺 usage analytics dashboard
- 缺对外 API 化能力

### 3.5 `farion1231/cc-switch`

它是当前最强的“配置/Provider/技能/MCP 治理台”之一。

已确认能力：

- `Claude / Codex / Gemini / OpenCode / OpenClaw`
- 50+ provider presets
- provider import/export
- tray quick switch
- unified MCP panel
- prompts panel
- skills installer
- cloud sync
- local proxy + failover + circuit breaker
- request rectifier
- usage & cost dashboard
- session manager
- workspace editor
- deep link import
- auto backup
- 多语言

对应 OSM 缺口：

- 缺 provider presets
- 缺统一 provider 管理
- 缺 tray quick switch
- 缺 MCP 统一管理
- 缺 prompts 管理
- 缺 skills 安装管理
- 缺 cloud sync
- 缺 proxy/failover/circuit-breaker
- 缺 request rectifier
- 缺 usage/cost dashboard
- 缺 workspace editor
- 缺 deep link import
- 缺自动备份治理

### 3.6 `Dicklesworthstone/coding_agent_session_search`

这是连接器覆盖面和搜索面最强的样本之一，但代码只能 reference-only。

已确认能力：

- 超宽助手支持面：`Codex / Claude / Gemini / Cline / OpenCode / Amp / Cursor / ChatGPT / Aider / Pi / Factory`
- 高性能索引
- lexical / semantic / hybrid search
- JSON robot mode
- TUI 三栏详情
- expand/view/search 机器接口
- rich export

对应 OSM 缺口：

- 助手覆盖面远弱于它
- 缺 lexical search
- 缺 semantic search
- 缺 hybrid ranking
- 缺 robot/json mode
- 缺统一 search/view/expand CLI
- 缺搜索驱动的知识重用路径

### 3.7 `d-kimuson/claude-code-viewer`

它在“单助手深度 viewer”上做得非常强。

已确认能力：

- 启动新会话 / 恢复会话 / 继续会话
- 实时会话监控
- 浏览器预览
- 文件上传预览
- Git diff viewer
- commit / push
- branch switcher
- files & tools inspector
- terminal panel
- MCP server viewer
- message scheduler
- 多语言

对应 OSM 缺口：

- 缺真实 session process control
- 缺 browser preview
- 缺文件上传预览
- 缺 Git diff/commit/push
- 缺 branch switcher
- 缺 files/tools inspector
- 缺 terminal panel
- 缺 MCP server viewer
- 缺 message scheduler

### 3.8 `daaain/claude-code-log`

它已经影响了 OSM 的导出设计，但还有一些未吸收点。

已确认能力：

- Markdown / HTML export
- 时间过滤
- summary-centric export
- transcript-friendly rendering

对应 OSM 缺口：

- 缺 HTML 导出
- 缺更丰富的按时间过滤导出
- 缺批量导出 / 批量转换入口

### 3.9 `lulu-sk/CodexFlow`

它代表的是“Windows 11 + WSL + 多引擎工作台”。

已确认能力：

- `Codex / Claude / Gemini / Terminal / Custom`
- 跨引擎历史中心
- Windows / WSL 双路径模型
- Markdown 历史渲染
- in-page search highlight
- one-click resume
- Git worktree 并行工作流
- GUI 输入增强：图片粘贴、拖拽、@文件、全屏输入
- usage/account monitor
- 多 profile / 多实例
- 通知系统

对应 OSM 缺口：

- 缺自定义引擎
- 缺更完整的 Windows + WSL 双态桥接
- 缺历史页全文内搜索高亮
- 缺 one-click resume
- 缺输入增强能力
- 缺多账号 / 多 profile
- 缺引擎层 usage monitor
- 缺多实例模式

### 3.10 `yoavf/ai-sessions-mcp`

它证明“OSM 不该只做 GUI”。

已确认能力：

- MCP server exposing sessions
- `list_sessions / search_sessions / get_session`
- BM25 ranking
- 支持从一个 agent 读取另一个 agent 的历史

对应 OSM 缺口：

- 缺 MCP server
- 缺对外 list/search/get 接口
- 缺 agent-to-agent session reuse 能力

### 3.11 `smtg-ai/claude-squad`

它是 tmux/worktree 型多代理调度器。

已确认能力：

- 多 session 并行
- isolated workspaces
- attach/detach
- review before apply
- checkout/pause/resume
- push workflow
- profiles

对应 OSM 缺口：

- 缺 attach/detach 控制
- 缺 review before apply 流程
- 缺 checkout/pause/resume
- 缺 profile-based launcher

### 3.12 `siteboon/claudecodeui`

这是“远程化 + 多端化 + 插件化”的代表项目。

已确认能力：

- 桌面 / 平板 / 手机响应式
- interactive chat
- integrated shell terminal
- file explorer
- git explorer
- session management
- plugin system
- MCP config via UI
- REST API
- local or remote usage
- 多 agent：`Claude / Cursor / Codex / Gemini`

对应 OSM 缺口：

- 缺移动端友好布局
- 缺 file explorer
- 缺 git explorer
- 缺 integrated shell
- 缺 plugin system
- 缺 REST API
- 缺 remote/self-hosted model
- 缺 Cursor CLI

### 3.13 `junhoyeo/tokscale`

它不是会话清理台，但在 usage/cost 维度几乎完全领先。

已确认能力：

- 超宽助手覆盖
- token & cost analytics
- pricing lookup
- contribution graph
- TUI dashboard
- JSON export
- social leaderboard
- OpenClaw / Amp / Pi / Qwen / Roo / Kilo / Mux / Synthetic 等连接器

对应 OSM 缺口：

- 缺 token 统计
- 缺 cost 统计
- 缺模型/平台用量排行
- 缺 pricing lookup
- 缺 usage 趋势图
- 缺更多连接器

### 3.14 `coder/agentapi`

它代表“程序化控制 agent”能力。

已确认能力：

- HTTP API server
- SSE events
- `/messages /message /status /events`
- chat web UI
- attach 到运行中的 agent
- 支持多 agent 控制

对应 OSM 缺口：

- 缺 HTTP 控制层
- 缺 SSE 实时事件流
- 缺 attach 到真实运行 agent
- 缺 chat-style remote control

### 3.15 `endorhq/rover`

它提供了隔离环境 + 后台任务执行的完整闭环。

已确认能力：

- container-isolated agent tasks
- background execution
- task inspect / diff / merge / push / iterate / shell
- VSCode extension
- agent:model 选择
- Docker/Podman integration

对应 OSM 缺口：

- 缺容器隔离执行
- 缺后台任务编排
- 缺 iterate loop
- 缺 inspect/diff/merge/push 成套工作流
- 缺 VSCode 集成

### 3.16 `kevinelliott/agentpipe`

它更偏多 agent 协作而非历史治理，但依然提供重要能力线。

已确认能力：

- 多 agent room conversation
- 多种对话模式
- 实时指标
- token/cost tracking
- Prometheus metrics
- 保存/恢复 conversation
- 导出 JSON/Markdown/HTML
- doctor / 健康检查
- config hot reload

对应 OSM 缺口：

- 缺多 agent room orchestration
- 缺 Prometheus metrics
- 缺健康检查 / doctor
- 缺 HTML 导出
- 缺 config hot reload

### 3.17 `udecode/dotai`

它不是产品对手，但方法论值得吸收。

已确认能力：

- 插件化 debug / test / git / learn / dig
- 自动技能触发
- prompt system
- workflow commands
- 多 agent review/research roles

对应 OSM 缺口：

- 缺内建工作流插件体系
- 缺结构化 debug / TDD / review 流程助手
- 缺会话知识抽取为技能/规则的闭环

### 3.18 `autohandai/commander`

这是新的本地多代理桌面工作台参考样本。

已确认能力：

- `Claude / Codex / Gemini` 多代理桌面聊天面板
- Git worktree 隔离工作区
- repo clone / open / recent project 启动器
- provider settings 持久化
- diff viewer / commit DAG / branch/worktree selector
- prompt 管理和执行模式切换

对应 OSM 缺口：

- 缺本地多代理协同面板
- 缺 project launcher
- 缺 worktree selector
- 缺 Git 历史与 diff 工作台
- 缺 provider settings 持久化治理

### 3.19 `pchalasani/claude-code-tools`

它不是完整治理台，但在插件、hooks、repair 和 search 工具链上非常强。

已确认能力：

- Claude Code plugin marketplace 集成
- hooks / skills / agents 扩展包
- search binary
- session repair
- safety hooks
- alt provider integrations

对应 OSM 缺口：

- 缺 hooks / skills 安装治理
- 缺 repair/fix-session 工作流
- 缺更结构化的安全 hooks
- 缺插件化扩展机制

### 3.20 `vultuk/claude-code-web`

这是“远程浏览器壳层 + 多会话持久化”的新补充样本。

已确认能力：

- 浏览器远程访问
- WebSocket 实时流
- token 鉴权
- 多会话持久化
- split view
- REST API
- 跨设备继续同一会话

对应 OSM 缺口：

- 缺远程访问
- 缺实时事件流
- 缺多会话浏览器持久化
- 缺 split view
- 缺 API 服务化外壳

### 3.21 `ChristopherA/claude_code_tools`

它不是完整 GUI，但在“会话收尾、恢复、worktree 技能脚本”这条线上很值得吸收。

已确认能力：

- session closure / session resume 技能
- 结构化 resume brief
- session cleanup 检查表
- git worktree create/list/remove/troubleshoot 脚本
- context usage statusline
- session start hooks

对应 OSM 缺口：

- 缺面向“删前提炼”的 handoff brief
- 缺 resume artifact 规范
- 缺 cleanup checklist 机制
- 缺 context budget 可见性
- 缺脚本化 worktree 生命周期工具

### 3.22 `ssdeanx/Gemini-CLI-Web`

这是 Gemini 方向最宽的远程工作台样本之一，但许可证口径冲突，当前只能 reference-only。

已确认能力：

- Gemini 远程 Web UI
- JWT 鉴权
- WebSocket 实时聊天
- file explorer / git explorer / shell
- spec design 工作流
- OpenAPI 文档
- 移动端 / PWA 适配

对应 OSM 缺口：

- 缺 Gemini 远程工作台
- 缺带鉴权的远程访问壳层
- 缺 OpenAPI 文档层
- 缺 spec workflow
- 缺移动端与 PWA 形态

### 3.23 `sugyan/claude-code-webui`

它比大而全的平台更轻，但把“浏览器壳 + plan mode + history loader + 单文件发布”做得很干净。

已确认能力：

- 轻量远程浏览器壳
- plan mode / permission mode 切换
- conversation history loader
- NDJSON/流式后端
- 单二进制打包
- 暗色 / 亮色和移动端适配

对应 OSM 缺口：

- 缺 plan approval 交互
- 缺轻量远程壳层
- 缺对话历史 API
- 缺单文件远程服务化外壳

## 4. OSM 尚未实现的全部功能清单

下面按能力域归档，不再按单仓库重复罗列。

### 4.1 连接器 / 助手覆盖面缺口

OSM 目前没有或没有完整支持：

- `Cursor CLI`
- `Cline CLI`
- `Kimi CLI`
- `Aider`
- `Amp`
- `ChatGPT local export`
- `Pi`
- `Qwen CLI`
- `Roo Code`
- `Kilo`
- `Mux`
- `Synthetic`
- `Amazon Q`
- `Goose`
- `Auggie`
- `Continue`
- `Crush`
- `OpenRouter direct agent`

### 4.2 搜索与索引缺口

- 跨会话全文检索
- BM25 排序
- semantic search
- hybrid search
- snippet preview
- search-as-you-type
- view/expand CLI
- 统一 search API
- 大规模索引缓存
- schema drift 监控与格式回归测试

### 4.3 会话控制缺口

- 启动新会话
- 恢复既有会话
- 继续运行中的会话
- attach/detach
- pause/resume
- session process control
- one-click resume
- 自动 rate-limit continue
- session handoff / resume brief

### 4.4 活跃会话与实时监控缺口

- active session cockpit
- live HUD
- busy/waiting/idle 状态识别
- live TODO/project monitor
- SSE/WebSocket 实时事件
- running session diagnostics

### 4.5 工作区 / 并行执行缺口

- Git worktree create/merge/delete/recycle
- 并行 agent 调度
- session data copy across worktrees
- task queue / scheduler / cron
- container isolated execution
- iteration workflow
- devcontainer/bootstrap hooks

### 4.6 Git 与项目视图缺口

- git diff viewer
- commit / push / branch switch
- global git status dashboard
- commit history
- project grouping
- file explorer
- file preview / editor
- browser preview

### 4.7 配置与 provider 治理缺口

- GitHub Copilot CLI / Factory Droid 配置治理
- Gemini / OpenClaw 配置写回与更深编辑
- provider presets
- provider import/export
- provider health monitor
- proxy/failover/circuit breaker
- request rectifier
- tray quick switch
- auto backup
- cloud sync
- shared config snippets

### 4.8 MCP / Skills / Prompts 缺口

- MCP server 暴露 OSM 数据
- MCP server viewer
- MCP unified management
- prompts panel
- skills panel / installer
- skills marketplace
- prompt/rule cross-app sync
- 会话知识提炼成技能或规则
- cleanup checklist / session-end hooks

### 4.9 分析与可视化缺口

- token usage dashboard
- cost dashboard
- model/platform breakdown
- pricing lookup
- usage timeline
- contribution graph
- leaderboards/shareable stats

### 4.10 自动化与对外接口缺口

- REST API
- HTTP control API
- OpenAPI
- agent automation server
- robot/json mode
- CLI for list/search/get/view/expand
- Prometheus metrics
- health/doctor checks
- 远程壳层鉴权

### 4.11 质量与运维缺口

- metadata repair / self-healing
- unknown session diagnostics
- fixture drift ledger
- screenshot automation
- config hot reload
- desktop auto update
- tray integration
- 多 profile / 多实例

### 4.12 UX 缺口

- 移动端适配
- 远程访问
- terminal panel
- richer diff/review flows
- uploads and previews
- richer filters
- better responsive detail panes

## 5. 最关键的结论

### 5.1 用户的批评完全成立

目前 OSM 还不能说“吸收了竞品能力”，因为真正落地到功能层的范围仍然太窄。已经吸收的主要是：

- Claude transcript viewer 风格细节
- richer markdown export
- `Gemini CLI / GitHub Copilot CLI / Factory Droid / OpenClaw` clean-room 会话适配
- session handoff 风格的 Markdown 导出
- 基础 dashboard 和审计流程

但距离“碾压所有竞品”还差得很远。

### 5.2 OSM 当前最弱的不是 UI，而是能力面

最关键缺口不是再调一层配色，而是：

1. 助手覆盖面离头部竞品还有明显距离
2. 没有搜索
3. 不能恢复/控制会话
4. 没有 worktree / 并行工作流
5. 没有 token/cost 分析
6. 没有 MCP/API/自动化出口
7. 没有配置与 provider 治理平台

### 5.3 真正的“碾压路径”不是复制单个产品，而是合并多条能力线

最合理的目标产品应同时吸收：

- `agent-sessions` 的多助手会话解析和索引
- `coding_agent_session_search` 的搜索能力线
- `ccmanager / claude-squad / rover / CodexFlow` 的 worktree 与并行代理编排
- `cc-switch` 的 provider/MCP/skills/config 治理
- `tokscale` 的 token/cost analytics
- `agentapi / ai-sessions-mcp / sugyan/claude-code-webui / Codexia / Gemini-CLI-Web` 的对外接口与远程能力
- `claude-code-viewer / claude-code-log` 的深度 transcript viewer 与导出质量
- `ChristopherA/claude_code_tools` 的 session handoff 与 worktree 脚本思路
- `Entropic` 的治理台、修复与诊断逻辑

## 6. OSM 进化路线

### Phase A：搜索、恢复与交接

必须尽快补齐：

- 全文检索
- snippet/result view
- 恢复/继续/附着会话
- session handoff / resume brief

### Phase B：配置与工作流治理

必须补：

- `GitHub Copilot CLI / Factory Droid` 配置治理
- `Gemini / OpenClaw` 配置写回与可视化修改
- worktree orchestration
- cleanup checklist / hooks
- Git 视图

### Phase C：平台化与远程壳层

必须补：

- REST/MCP/OpenAPI
- remote/mobile
- active session monitor
- plan approval / permission mode

### Phase D：分析与诊断

必须补：

- token/cost analytics
- diagnostics/repair
- doctor / health checks
- screenshot automation

## 7. 本轮代码集成优先级

在“全部功能都要最终做出来”的长期目标下，本轮代码实现仍需要有执行顺序。

### P0 本轮必须落地

- session handoff / resume brief
- 全文检索入口
- `GitHub Copilot CLI / Factory Droid` 配置治理
- `Gemini / OpenClaw` 配置写回与可视化修改
- 相应 fixtures + tests + snapshot coverage

### P1 本轮若时间允许继续落地

- worktree orchestration
- MCP/HTTP 只读接口
- 轻量远程壳层

### P2 后续继续推进

- token/cost analytics
- provider/MCP/skills 整合面板
- active cockpit

## 8. 对发布口径的影响

在上述缺口补上之前，OSM 不能对外宣称：

- 已经完整吸收竞品能力
- 已经成为最强多助手治理台
- 已经达到发布稳定版标准

它现在最多只能称为：

- 正在从单纯的 `Codex/Claude/OpenCode` 会话整理器，升级成真正的多助手治理平台
- 具备一部分基础治理能力，但还没有完成对搜索、恢复、并行、分析、配置治理的全面吸收

---

## 9. 结论

如果目标真的是“碾压市面上所有竞品”，那 OSM 的正确方向不是继续补几条文案，而是把产品路线明确改成：

**多助手会话治理 + 搜索检索 + 配置/provider治理 + worktree编排 + 活跃会话控制 + token/cost分析 + MCP/API平台化**

这份文档列出的缺口，就是 OSM 还没实现的完整功能债清单。
