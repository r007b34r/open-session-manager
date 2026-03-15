# Terminal Agent Session Governance Landscape

日期：2026-03-15

## 1. 需求收敛

你的指向性需求不是“再做一个聊天记录查看器”，而是一个本地优先的多助手会话治理平台，至少要同时覆盖以下能力：

- 识别 Win11 原生环境、Linux 原生环境，以及 Win11 下的 WSL 环境
- 发现本地安装的终端代码助手及其配置层、日志层、会话层、密钥层
- 读取真实会话主题、内容、进度、最后运行日期、文件/仓库上下文、错误与待办
- 帮用户判断哪些会话应该保留、导出、迁移、归档、软删除或彻底删除
- 删除前自动提炼高价值内容并导出为 Markdown
- 对第三方中转 key、非官方 base URL、危险权限配置给出明显标记
- 所有破坏性动作都需要可追溯日志和可恢复路径

这意味着产品必须同时具备：

- 多源解析能力
- 本地索引能力
- 内容理解能力
- 安全治理能力
- 可视化操作能力

## 2. 目标数据源和已确认路径

### 2.1 Codex

- OpenAI 官方文档确认用户级配置位于 `~/.codex/config.toml`，项目级覆盖位于 `.codex/config.toml`
- OpenAI 官方配置参考确认 `history.persistence` 会把会话 transcript 保存到 `history.jsonl`
- OpenAI 官方配置参考确认日志目录默认位于 `$CODEX_HOME/log`，状态 SQLite DB 位于 `sqlite_home` 或 `CODEX_HOME`
- OpenAI 官方 issue 日志中可见 `rolloutPath` 指向 `~/.codex/sessions/...jsonl`
- 社区项目 `agent-sessions` 也明确按 `~/.codex/sessions` 读取 Codex 会话

结论：

- `Codex` 的配置层和状态层是官方可证实的
- 会话 rollout 文件位于 `~/.codex/sessions` 这点可以较高置信度支持，但当前更像“官方运行输出 + 社区实践共同佐证”，应在产品中标成“已验证实现路径”，而不是死写为唯一来源

### 2.2 Claude Code

- Anthropic 官方文档确认用户配置位于 `~/.claude/settings.json`
- 项目级配置位于 `.claude/settings.json` 与 `.claude/settings.local.json`
- 企业托管配置路径对 Windows/Linus/WSL 也有官方说明
- Anthropic 官方 `statusline` 文档暴露了 `transcript_path` 字段，说明运行时会向外提供 transcript 文件路径
- 社区项目 `claude-code-viewer` 和 `claude-code-log` 都把 `~/.claude/projects/<project>/<session-id>.jsonl` 作为标准数据源

结论：

- `Claude Code` 的配置层是官方明确的
- transcript 的“存在”和“可定位”是官方明确的
- `~/.claude/projects/...jsonl` 目前主要由社区实现稳定验证，适合在 v1 中支持，但需要保留适配器升级能力

### 2.3 OpenCode

- OpenCode 官方文档确认全局配置位于 `~/.config/opencode/opencode.json`
- 项目配置位于项目根的 `opencode.json`
- `.opencode` 目录承载 agents、commands、plugins、skills 等扩展层
- OpenCode 官方 Windows(WSL) 文档明确建议在 WSL 中运行，并明确说明配置和 sessions 存储在 `~/.local/share/opencode/`
- 官方文档同时说明 Web/Desktop 可以通过本地 server 访问

结论：

- `OpenCode` 对 WSL 场景尤其友好，适合作为 Win11 首批支持对象
- 其 client/server 结构使“本地网页可视化”方向天然成立

## 3. GitHub 同类项目调研

## 3.1 `Dicklesworthstone/coding_agent_session_search`

链接：<https://github.com/Dicklesworthstone/coding_agent_session_search>

优点：

- 覆盖面很广，已经统一聚合 Codex、Claude Code、Gemini CLI、OpenCode、Cursor、Aider 等多个来源
- 有统一索引模型和搜索体验，说明“多助手归一化”是可行的
- 支持 Markdown/HTML 导出，而且 HTML 导出带加密能力
- 支持 Windows 安装，说明不是只能做 macOS 工具

缺点：

- 重心是 TUI/CLI 搜索，不是面向普通用户的图形化治理台
- 重点在“找得到”，不在“应不应该删”“删前该提炼什么”
- 对配置、密钥、第三方 relay 风险的治理不构成产品主线

启发：

- 借鉴其“统一会话模型”和“多 connector”思路
- 不走纯 TUI 路线，要把“决策辅助 + 治理动作 + 审计日志”做成一等公民

## 3.2 `jazzyalex/agent-sessions`

链接：<https://github.com/jazzyalex/agent-sessions>

优点：

- 产品完成度高，索引、搜索、分析、resume、archive 都比较成熟
- 本地优先和隐私叙事清晰
- 已覆盖 Codex、Claude Code、OpenCode 等多个 agent

缺点：

- 明确要求 `macOS 14+`
- 核心价值偏浏览、分析、resume，不是跨平台治理工具
- 对密钥配置、代理配置、危险设置和删除前提炼支持不突出

启发：

- 索引和 analytics 可以做得很深
- 但你的产品不能被锁死在 macOS，也不能停留在“会话浏览器”

## 3.3 `lulu-sk/CodexFlow`

链接：<https://github.com/lulu-sk/CodexFlow>

优点：

- 非常接近你的 Win11/WSL 用户场景
- 已经证明跨 Windows/WSL 路径转换、多引擎聚合、历史中心、resume 都可以一起成立
- 对项目维度组织历史比按单纯 session 列表更接近真实使用方式

缺点：

- 更像“统一工作台/启动器”，不是“会话治理和证据提炼平台”
- 虽然有 read-only history center，但删除、归档、提炼、风险审查还不是主轴
- 配置层、密钥层的审计不够深

启发：

- Win11 第一版必须把 WSL 视为一等公民
- 项目聚合视角比单纯 transcript 列表更实用

## 3.4 `d-kimuson/claude-code-viewer`

链接：<https://github.com/d-kimuson/claude-code-viewer>

优点：

- Web 形态很贴近你的“网页可视化呈现”要求
- 对 transcript 结构保真比较好，强调 schema validation 和 zero data loss
- 有 diff、todo、MCP 配置查看、session 控制等完整 Claude 生态能力

缺点：

- 明确是 Claude Code 单源工具
- 官方说明仅支持 macOS/Linux，不支持 Windows
- 更偏 Claude 的替代前端，不是跨助手治理控制台

启发：

- 会话详情页必须做深，不能只停留在搜索结果列表
- 但不能把产品做成某一个 agent 的重皮客户端

## 3.5 `kbwo/ccmanager`

链接：<https://github.com/kbwo/ccmanager>

优点：

- 已支持 Claude、Gemini、Codex、OpenCode 等多个 agent
- 有 worktree、多会话并行、session copying 等非常实用的工程能力
- 证明“多助手 + 多项目 + 多 worktree”是高频需求

缺点：

- 核心是会话管理和工作流调度，不是 transcript 治理
- 不强调真实主题提炼、价值判断、删除前归档
- 对配置/密钥可视化和风险分析支持有限

启发：

- 你的项目应该把“会话上下文迁移”和“项目维度治理”纳入路线图
- 但主轴仍应是治理和提炼，而不是调度器

## 3.6 `Dimension-AI-Technologies/Entropic`

链接：<https://github.com/Dimension-AI-Technologies/Entropic>

优点：

- 是少数明确做成跨平台 Electron GUI 的项目
- 已经把 `~/.claude`、`~/.codex`、`~/.gemini` 融合为 provider-aware 模型
- 有 maintenance tooling，甚至有删除空 session、清理旧 tab 等动作

缺点：

- 支持面仍偏窄，OpenCode 等并非核心
- “维护”动作还较浅，更像 housekeeping，不是面向价值提炼的治理流
- 对 key、relay、敏感配置的集中盘点仍不足

启发：

- 你的方向是对的，市场已经证明“桌面 GUI + 多 agent 诊断”有现实需求
- 但还没有一个项目把“内容理解、配置治理、导出后删除、敏感项审计”真正做全

## 3.7 `daaain/claude-code-log`

链接：<https://github.com/daaain/claude-code-log>

优点：

- 对 Claude transcript 到 Markdown/HTML 的导出非常接近你的“删除前提炼”要求
- Smart summaries、usage、日期过滤、TUI 浏览都做得比较扎实

缺点：

- 单助手
- 更偏导出和查看，不是治理平台

启发：

- Markdown 导出功能不能只导出原文，要有智能摘要和结构化 frontmatter

## 4. 现有开源项目的共同短板

把上面项目放在一起看，行业里已经有三类成熟思路：

- 浏览/搜索类
- 统一工作台/启动器类
- 单助手专用可视化类

但你的需求恰好落在它们的交集之外：

- 需要 Win11 + Linux，而不是只做 macOS
- 需要多助手 + 多配置层 + 多密钥层，而不是只看 transcript
- 需要“价值提炼后再删除”，而不是只有 search 或 archive
- 需要“第三方 relay / 非官方 endpoint / 危险设置”风险识别
- 需要完整审计链路，而不是直接 destructive actions

结论：

目前 GitHub 上有相当多“可借鉴组件式思路”，但还没有一个项目完整覆盖你的目标产品面。

## 5. 最适合吸收的设计信号

建议吸收这些现成经验：

- 从 `coding_agent_session_search` 吸收多 connector 归一化模型
- 从 `agent-sessions` 吸收高性能本地索引、analytics 和 local-first 叙事
- 从 `CodexFlow` 吸收 Windows/WSL 路径处理与项目级组织方式
- 从 `claude-code-viewer` 吸收深度 session 详情页与 schema 校验
- 从 `Entropic` 吸收跨平台 GUI + maintenance tooling
- 从 `claude-code-log` 吸收高质量 Markdown 导出思路

而本项目应该新增的差异化主轴是：

- 治理建议引擎
- 删除前价值提炼
- 配置/密钥/relay 风险总览
- 审计日志和可恢复删除

## 6. 参考来源

- OpenAI Codex Config Basics: <https://developers.openai.com/codex/config-basic>
- OpenAI Codex Config Reference: <https://developers.openai.com/codex/config-reference>
- OpenAI Codex issue showing `rolloutPath`: <https://github.com/openai/codex/issues/5903>
- Anthropic Claude Code settings: <https://docs.anthropic.com/en/docs/claude-code/settings>
- Anthropic Claude Code statusline: <https://docs.anthropic.com/en/docs/claude-code/statusline>
- OpenCode config docs: <https://opencode.ai/docs/config/>
- OpenCode Windows WSL docs: <https://opencode.ai/docs/windows-wsl/>
- `coding_agent_session_search`: <https://github.com/Dicklesworthstone/coding_agent_session_search>
- `agent-sessions`: <https://github.com/jazzyalex/agent-sessions>
- `CodexFlow`: <https://github.com/lulu-sk/CodexFlow>
- `claude-code-viewer`: <https://github.com/d-kimuson/claude-code-viewer>
- `ccmanager`: <https://github.com/kbwo/ccmanager>
- `Entropic`: <https://github.com/Dimension-AI-Technologies/Entropic>
- `claude-code-log`: <https://github.com/daaain/claude-code-log>
