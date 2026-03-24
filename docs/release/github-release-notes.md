# open Session Manager v0.3.0 Public Preview

这次把 OSM 往前推了一整段，不再只是“能看快照”，而是把搜索缓存、会话控制、analytics 和 worktree 工具链接到了可验证的实现上。

## 本版重点

- 会话支持面从 3 个扩到 9 个：
  - `Codex`
  - `Claude Code`
  - `OpenCode`
  - `Gemini CLI`
  - `GitHub Copilot CLI`
  - `Qwen CLI`
  - `Roo Code`
  - `Factory Droid`
  - `OpenClaw`
- 配置审计扩到 7 个助手，`GitHub Copilot CLI / Factory Droid` 已补上用户级配置治理
- `GitHub Copilot CLI / Factory Droid` 现在会按会话项目路径带出项目级配置覆盖层
- `GitHub Copilot CLI / Factory Droid / Gemini CLI / OpenClaw` 的配置编辑面板现在支持统一 provider presets，可一键恢复到官方端点和常用模型组合
- 配置编辑面板新增共享 snippet library，可把 provider/model/base URL 组合保存到本地、导出为稳定 JSON，并重新导入到任意支持写回的草稿里
- 把一批真实竞品镜像拉到本地并纳入 catalog、研究索引和开源致谢，不再只有零散笔记
- 修掉会直接影响可用性的会话质量问题：
  - `Codex` 不再把 `AGENTS.md`、环境注入块误当真实主题
  - `Claude Code` 不再把纯 `file-history-snapshot` JSONL 当候选会话
- snapshot 现在会把 session index cache 持久化到 SQLite，并按文件变化做增量重建
- Sessions 列表现在会显示会话 ID，多个相近标题不再像同一条
- `Codex / Claude Code / OpenCode / Gemini CLI / GitHub Copilot CLI / Factory Droid / OpenClaw` 现在支持真实 resume / continue，结果会写回会话控制状态和审计历史
- `serve` 现在额外暴露 `/metrics` Prometheus 指标端点，可直接采集 sessions/configs/git projects/doctor findings/audit events
- `serve` 现在还支持本地 automation server：可通过 `POST /api/v1/automation/tasks` 触发 `snapshot.refresh / sessions.search / sessions.resume / sessions.continue`，再通过 `GET /api/v1/automation/tasks/{taskId}` 读取稳定回执
- `serve` 现在支持 `POST /api/v1/auth/local-token`，可为本地壳层发行 loopback-only 的短期签名 Bearer token；受保护路由可接受静态 Bearer 或短期 token
- Web 详情页补上了一键恢复、继续提示、最近控制结果和导出落盘路径提示
- 总览页新增 `active session cockpit`，可直接查看当前可控会话、最近控制响应，并手动刷新运行时状态
- 支持 usage / cost analytics：
  - `Codex`
  - `Claude Code`
  - `OpenCode`
  - `Gemini CLI`
  - `Qwen CLI`
  - `Roo Code`
  - `OpenClaw`
- `Claude Code / Gemini CLI` 在上游日志未直接给出成本时，现在会按本地价格目录估算成本
- usage 面板现在会明确标注 `reported / estimated / unknown` 成本来源，并展示日级 usage timeline
- usage 面板新增 model breakdown 和 provider/platform breakdown，可直接看出 token/cost 集中在哪些模型，以及配置 footprint 分布在哪些 provider
- Sessions 搜索现在会做本地 BM25 风格 lexical 排序，并补上 search-as-you-type 防抖、取消旧查询、命中片段和命中来源标签；transcript 命中会直接定位到右侧详情高亮
- Rust CLI 现在新增统一的 `list / search / get / view / expand` 五类命令，不再只能对着整个 snapshot JSON 自己筛
- 桌面端现在也暴露同名 Tauri command，CLI 和桌面 API 共用同一套 Rust 查询层，不再各算各的
- Sessions 页面新增高级筛选器，可按 assistant / project / risk / export / control 组合收窄会话队列，快速找出该导出、该隔离或可直接恢复的会话
- 配置写回前新增 masked diff 审查流和风险提示，必须显式确认后才会落盘；会话移入隔离区前也新增 cleanup 审查确认
- `DiffViewer` 现在有独立组件测试和空状态提示，不再只是配置审查流里的隐含实现
- 浏览器运行现在也有正式预览链路，`npm --prefix web run browser` 会先 build 再用 `vite preview` 启动，Playwright E2E 不再依赖开发态 dev server
- 会话详情新增 `Knowledge Lift` 卡片，可把当前摘要、待办、风险和关键证据直接提炼成 rule / skill Markdown，再决定是否清理原会话
- 配置写回的自动备份 manifest 现在会进入审计链路，并直接显示在 Audit 页，便于确认回滚落点
- 导出目录设置、导出后路径显示、语言切换、主题切换继续保留
- Markdown 导出补上了 `Session Handoff`，会把 `Next focus / Open tasks / Resume cue` 一起写进去
- Markdown 导出现在还会同步生成结构化 cleanup checklist，并在项目内检测到 `session-end` hook 时执行；软删除前也不再只看 Markdown，而是要求 checklist 已成功落地
- 新增 `git-worktree-manager`，统一处理仓库内 `.worktrees/` 下的 `create / merge / delete / recycle`
- 新增三条本地镜像研究并纳入治理目录：
  - `ChristopherA/claude_code_tools`
  - `ssdeanx/Gemini-CLI-Web`
  - `sugyan/claude-code-webui`

## 这版真正吸收了什么

已经真实落进代码的，不只是“研究过”：

- `jazzyalex/agent-sessions`
  - clean-room 吸收并重写了 `Gemini CLI`、`GitHub Copilot CLI`、`Factory Droid`、`OpenClaw` 适配器能力线
- `daaain/claude-code-log`
  - 延续 Markdown 导出、transcript highlights、todo snapshot 的思路
- `d-kimuson/claude-code-viewer`
  - 延续 viewer 风格详情面板和 todo evidence 呈现思路
- `ChristopherA/claude_code_tools`
  - 吸收 session closure / resume brief、cleanup checklist、session-end hook 这条会话收尾链路，补到 OSM 的导出和软删除守卫
- `farion1231/cc-switch`
  - clean-room 吸收统一 provider/config 治理方向，以及 `Gemini CLI` 与 `OpenClaw` 配置治理中的路径、auth mode、provider/base URL 风险建模与 preset catalog
- `kbwo/ccmanager`
  - clean-room 吸收 repo-local worktree 生命周期管理方向，补成 `git-worktree-manager` CLI
- `endorhq/rover`
  - 吸收 `GitHub Copilot CLI` companion `mcp-config.json` 路径线索，补到 OSM 的 clean-room 配置审计里
- `junhoyeo/tokscale`
  - clean-room 吸收 usage / token / cost 字段模型、本地价格目录估算、成本来源标注和日级 timeline，并补进 `Qwen CLI / Roo Code` 的本地日志解析
- `jazzyalex/agent-sessions`
  - 吸收本地搜索结果呈现、命中来源可视化，以及本地优先索引这条工作台方向
- `yoavf/ai-sessions-mcp`
  - 吸收本地 lexical ranking + snippet 的产品线索，先落到 Web 工作台，并补上 transcript 命中高亮

已经纳入本地镜像、研究索引和致谢体系的还包括：

- `kbwo/ccmanager`
- `farion1231/cc-switch`
- `junhoyeo/tokscale`
- `coder/agentapi`
- `endorhq/rover`
- `kevinelliott/agentpipe`
- `milisp/codexia`
- `siteboon/claudecodeui`
- `smtg-ai/claude-squad`
- `udecode/dotai`
- `autohandai/commander`
- `pchalasani/claude-code-tools`
- `vultuk/claude-code-web`
- `sugyan/claude-code-webui`
- `ssdeanx/Gemini-CLI-Web`

## 关于 `edition = "2024"`

`Cargo.toml` 继续使用 `edition = "2024"`，不是写错年份。

原因很简单：Rust edition 是语言版本，不是当前年份。当前本机 `cargo` 支持的 edition 仍然是：

- `2015`
- `2018`
- `2021`
- `2024`

`2026` 不是有效值，直接改会让仓库配置失效。

## 当前已实现的能力

- 9 个终端代码助手的本地会话发现与解析
- 7 个终端代码助手的配置审计读取与风险预览
- `GitHub Copilot CLI / Factory Droid` 的项目级配置发现
- `GitHub Copilot CLI / Factory Droid / Gemini CLI / OpenClaw` 的安全写回、备份、回滚和审计事件
- `GitHub Copilot CLI / Factory Droid / Gemini CLI / OpenClaw` 的统一 provider presets、套用和恢复检测值
- 配置 snippet library、本地持久化、JSON 导入导出和 save/apply/export/import 审计事件
- SQLite session index cache、增量重建和 index run 统计
- `Codex / Claude Code / OpenCode / Gemini CLI / GitHub Copilot CLI / Factory Droid / OpenClaw` 的真实 resume / continue，以及对应的 session control 状态面板
- `Codex / Claude Code / OpenCode / Gemini CLI / OpenClaw` 的 usage / cost 汇总
- 本地价格目录估算、`reported / estimated / unknown` 成本来源，以及 overview 日级 usage timeline
- 总览里的 model breakdown 与 provider/platform breakdown
- 会话标题、摘要、进度、价值分、风险标记、最后活跃时间
- transcript highlights 与 Claude todo snapshot
- 会话知识提炼：rule / skill Markdown 预览与复用
- Sessions 页加权搜索、search-as-you-type 防抖与取消、命中片段、来源标签和 transcript 命中高亮
- 配置写回 review gate：masked diff、风险提示、确认后再应用
- 会话 cleanup review gate：导出后仍需显式确认才会移入隔离区
- Sessions 页高级筛选：assistant / project / risk / export / control 组合筛选
- `doctor` CLI 与总览环境诊断面板，可显示被跳过的 malformed session 文件
- `Claude Code` 历史 JSONL 如果正文缺失 `sessionId`，但文件名本身是 UUID，OSM 现在会自动恢复会话 ID，而不是一律丢弃
- 已知会话根目录下被适配器静默过滤的未知 session-like 文件，现在也会进入 `doctor` 诊断输出
- Sessions 详情面板改成非 sticky、单列卡片布局，选中交互和高 DPI 窗口下不再出现拉伸失真
- fixtures 现在有版本/来源/hash ledger，`verify.ps1` 会强制跑 drift 检查，fixture 漂移不再靠人工记忆
- fixtures snapshot 现在有规范化 golden 基线；一旦输出漂移，校验会直接报出具体 JSON 路径差异
- `Session Handoff` Markdown 导出
- cleanup checklist / session-end hook 导出链路
- Markdown 导出、软删除、恢复、审计历史
- 中英文切换与跟随系统语言
- 浅色 / 深色 / 跟随系统主题
- Markdown 导出目录设置与导出路径显示
- `node scripts/git-worktree-manager.mjs` 提供 `.worktrees/` 下的 create / recycle / merge / delete
- `npm --prefix web run browser` 提供固定端口的浏览器预览入口，适合不走桌面壳层时直接本地打开
- `cargo run -- list/search/get/view/expand` 提供统一的终端查询入口，既能拿 JSON，也能直接看 Markdown 视图
- CLI 现在支持显式 `--json` 机器模式，输出紧凑稳定 JSON，便于脚本和自动化消费
- 桌面端 Tauri command 同步暴露 `list/search/get/view/expand`，并补上 `assistant` 过滤、`limit/offset` 分页和 `sortBy/descending` 排序，便于后续 Web / HTTP 壳层直接复用
- `cargo run -- serve` 现在会启动本地只读 REST API，暴露 `health/list/search/get/view/expand`，并支持可选 Bearer token
- `/openapi.json` 现在会返回本地 REST API 的 OpenAPI 3.1 文档，方便脚本、测试和后续外壳复用同一份契约
- `cargo run -- mcp` 现在会通过 `stdio` 暴露 `list_sessions/search_sessions/get_session` 三个 MCP tools，便于把本地会话知识接进自动化链路
- Tauri 桌面运行时与浏览器 fallback
- upstream intake pipeline、研究索引与开源致谢

## 当前边界

以下内容不包含在 `v0.3.0 Public Preview` 承诺范围内：

- 语义搜索、hybrid ranking 和更大历史库压测
- 未来新增连接器的真实会话控制、attach / detach、pause / resume 和进程观测
- 建立在 worktree CLI 之上的多项目调度、并行 agent 编排和容器隔离执行
- provider 健康探测和自动切换
- 更深 analytics（model/platform breakdown / contribution graph / shareable stats）和更多助手连接器
- 更完整的异步任务队列、SSE 推送和远程壳层鉴权
- Linux 桌面实机回归
- 发布安装包与签名流程

## 验证结果

本版发布前已通过：

- `cargo test -- --test-threads=1`
- `cargo test --test cli_snapshot`
- `cargo run --manifest-path src-tauri/Cargo.toml -- search --fixtures tests/fixtures --query Claude`
- `node --test tests/git-workflow/git-worktree-manager.test.mjs`
- `npm --prefix web run test`
- `npm --prefix web run e2e`
- `npm --prefix web run build`
- `node --test tests/web-preview/browser-preview.test.mjs`
- `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`

## 开源致谢

完整感谢名单和许可边界已收口到：

- `docs/release/open-source-attribution.md`
- `docs/research/upstreams/index.md`
- `third_party/upstreams/catalog.json`
