# OSM Git 驱动专项总 spec

日期：2026-03-22  
分支：`feat/usability-clarity`  
负责人：`r007b34r`

---

## 1. 这份 spec 解决什么问题

仓库里已经有竞品调研、差距分析和若干阶段性计划，但还缺一份能直接驱动开发的总账。用户批评的点也很明确：

- 不能只写“吸收了什么”，必须真的落成功能
- 不能只靠口头计划，必须把还没做的事拆成可执行指标
- 每一项都要能做 TDD，并且把 Git 的本地审查、标记、提交链路真正用起来

这份 spec 的作用只有两个：

1. 把当前还没做到的事全部收敛成统一 backlog
2. 规定每个 backlog item 的 Git 驱动 TDD 执行方式

## 2. 当前基线

截至 2026-03-22，OSM 已经具备：

- `Codex / Claude Code / OpenCode / Gemini CLI / GitHub Copilot CLI / Factory Droid / OpenClaw` 会话发现与解析
- 上述 7 类助手的配置读取、风险审计和部分项目级配置发现
- transcript digest、todo 提取、Markdown 导出、handoff 导出、软删除、恢复
- 本地加权 lexical 搜索、命中片段和来源标签
- SQLite 会话索引缓存、增量重建和 index run 统计
- `Codex / Claude Code / OpenCode / Gemini CLI / GitHub Copilot CLI / Factory Droid / OpenClaw` 的真实 resume / continue 控制链路，以及 Web 详情页的一键恢复入口
- 基础 usage / cost 聚合
- repo-local `.worktrees/` 生命周期 CLI
- Web / Tauri 桌面 UI、中英双语、跟随系统语言、浅色/深色主题、导出目录偏好

还没有具备的内容，才是这份 spec 的主体。

## 3. 状态标记

- `todo`：还没开始做
- `partial`：已有局部能力，但离竞品级完成度还差关键链路
- `done`：本地功能、测试、验证和 Git 证据链已齐
- `blocked`：依赖真实样本、许可证边界或外部接口

## 4. 全量未完成事项

### 4.1 连接器与助手覆盖

| ID | 优先级 | 状态 | 事项 | 验收指标 |
| --- | --- | --- | --- | --- |
| `ADP-01` | P1 | todo | `Cursor CLI` 会话与配置适配 | fixture + Rust adapter 测试通过，支持矩阵更新 |
| `ADP-02` | P1 | todo | `Cline CLI` 会话与配置适配 | fixture + Rust adapter 测试通过，支持矩阵更新 |
| `ADP-03` | P1 | todo | `Kimi CLI` 会话与配置适配 | fixture + Rust adapter 测试通过，支持矩阵更新 |
| `ADP-04` | P1 | todo | `Aider` 会话与配置适配 | fixture + Rust adapter 测试通过，支持矩阵更新 |
| `ADP-05` | P1 | todo | `Amp` 会话与配置适配 | fixture + Rust adapter 测试通过，支持矩阵更新 |
| `ADP-06` | P2 | todo | `ChatGPT` 本地导出导入 | snapshot 能展示导入来源、主题和时间线 |
| `ADP-07` | P2 | todo | `Pi` 会话与配置适配 | fixture + Rust adapter 测试通过 |
| `ADP-08` | P1 | partial | `Qwen CLI` 会话与配置适配 | 会话发现、摘要、transcript、usage 与 fixture 已落地；配置治理待补 |
| `ADP-09` | P1 | partial | `Roo Code` 会话与配置适配 | 会话发现、摘要、transcript、usage 与 fixture 已落地；配置治理待补 |
| `ADP-10` | P2 | todo | `Kilo` 会话与配置适配 | fixture + Rust adapter 测试通过 |
| `ADP-11` | P2 | todo | `Mux` 会话与配置适配 | fixture + Rust adapter 测试通过 |
| `ADP-12` | P2 | todo | `Synthetic` 会话与配置适配 | fixture + Rust adapter 测试通过 |
| `ADP-13` | P1 | todo | `Amazon Q` 会话与配置适配 | fixture + Rust adapter 测试通过 |
| `ADP-14` | P2 | todo | `Goose` 会话与配置适配 | fixture + Rust adapter 测试通过 |
| `ADP-15` | P2 | todo | `Auggie` 会话与配置适配 | fixture + Rust adapter 测试通过 |
| `ADP-16` | P2 | todo | `Continue` 会话与配置适配 | fixture + Rust adapter 测试通过 |
| `ADP-17` | P2 | todo | `Crush` 会话与配置适配 | fixture + Rust adapter 测试通过 |
| `ADP-18` | P1 | todo | `OpenRouter direct agent` 配置与来源识别 | 配置审计可识别 base URL、provider、token 风险 |

### 4.2 搜索、索引与知识复用

| ID | 优先级 | 状态 | 事项 | 验收指标 |
| --- | --- | --- | --- | --- |
| `SRCH-01` | P0 | done | 大历史索引层 | SQLite 已落地 `session_index_cache / session_index_runs`，重复 snapshot 会复用未变化 session 的索引结果 |
| `SRCH-02` | P0 | done | BM25 排序 | 搜索结果测试已验证 BM25 风格 lexical 排名稳定，标题中的精确短语命中不会再被低信号字段堆分数反超 |
| `SRCH-03` | P1 | todo | semantic search | 引入向量索引或嵌入后，语义检索测试通过 |
| `SRCH-04` | P1 | todo | hybrid ranking | lexical + semantic 混合排序测试通过 |
| `SRCH-05` | P0 | done | snippet preview 深化 | 搜索命中会保留 transcript focus，详情页可高亮对应命中的 transcript 条目 |
| `SRCH-06` | P1 | done | search-as-you-type | 前端交互测试已验证防抖、取消旧查询和 pending 刷新提示 |
| `SRCH-07` | P1 | done | `list/search/get/view/expand` 统一 CLI | Rust CLI 已支持五类命令，`cli_snapshot` 集成测试覆盖 fixture 下的 list/search/get/view/expand 输出 |
| `SRCH-08` | P1 | done | 统一 search API | Tauri command 已支持 `assistant` 过滤、`limit/offset` 分页和 `sortBy/descending` 排序，并有桌面单测覆盖 |
| `SRCH-09` | P0 | done | 大规模索引缓存与增量更新 | 单个 session 文件变化时只重建对应条目，并记录 `cache_hits / cache_misses / reindexed_files / stale_deleted` |
| `SRCH-10` | P0 | done | schema drift 监控 | fixture ledger 与 snapshot golden diff 已接入统一 verify，失败时会返回具体 JSON 路径差异 |

### 4.3 会话控制与恢复

| ID | 优先级 | 状态 | 事项 | 验收指标 |
| --- | --- | --- | --- | --- |
| `SES-01` | P1 | todo | 启动新会话 | UI 或 CLI 可启动受支持助手的新会话，集成测试通过 |
| `SES-02` | P0 | done | 恢复既有会话 | `Codex / Claude Code / OpenCode / Gemini CLI / GitHub Copilot CLI / Factory Droid / OpenClaw` 已支持真实 resume，snapshot、HTTP API 和 UI 可观测最近结果 |
| `SES-03` | P0 | done | 继续运行中的会话 | 上述 7 个助手已支持 continue prompt 并落审计，附着与节流保护已完成 |
| `SES-04` | P0 | done | attach / detach | 会话控制测试覆盖附着与分离 |
| `SES-05` | P1 | done | pause / resume | 生命周期状态、pause guard 与审计测试通过 |
| `SES-06` | P1 | done | session process control | 已读取并展示进程状态、退出码、运行时长、事件数与控制 token |
| `SES-07` | P0 | done | one-click resume | Web 详情页已接恢复按钮、继续提示和最近控制结果；真实执行已覆盖当前 7 个已解析助手 |
| `SES-08` | P1 | done | continue rate-limit 策略 | 恢复节流和冲突保护测试通过 |
| `SES-09` | P0 | done | 标准化 handoff / resume artifact | Markdown 导出会生成 `resume-<session>.json`，软删除 manifest 与恢复审计持续引用该工件，dashboard / Web Audit 已可见 |

### 4.4 活跃会话与实时监控

| ID | 优先级 | 状态 | 事项 | 验收指标 |
| --- | --- | --- | --- | --- |
| `MON-01` | P0 | done | active session cockpit | 总览页已展示可控会话、最近控制结果和刷新动作，并有 Web / E2E 测试覆盖刷新后的状态更新 |
| `MON-02` | P1 | done | live HUD | cockpit 与详情面板已展示运行态、进程字段、事件数与控制 token，Rust/Web 测试通过 |
| `MON-03` | P0 | done | busy / waiting / idle 状态识别 | 适配器状态机测试通过 |
| `MON-04` | P1 | todo | live TODO / project monitor | TODO 变化触发 UI 刷新和审计事件 |
| `MON-05` | P1 | todo | SSE / WebSocket 实时事件 | 前后端流式测试通过 |
| `MON-06` | P1 | todo | running session diagnostics | 异常会话诊断卡片和日志导出测试通过 |

### 4.5 工作区、worktree 与并行执行

| ID | 优先级 | 状态 | 事项 | 验收指标 |
| --- | --- | --- | --- | --- |
| `WRK-01` | P0 | done | Git worktree create / merge / delete / recycle | `git-worktree-manager` CLI 已通过创建、合并、删除、复用、stale prune 回收测试 |
| `WRK-02` | P1 | todo | 并行 agent 调度 | 调度状态与资源冲突测试通过 |
| `WRK-03` | P1 | todo | session data copy across worktrees | 同一任务切换工作树时能继承必要上下文 |
| `WRK-04` | P1 | todo | task queue / scheduler / cron | 定时任务测试通过 |
| `WRK-05` | P1 | todo | 容器隔离执行 | Docker/Podman 集成测试通过 |
| `WRK-06` | P1 | todo | iteration workflow | inspect/diff/merge/push 的闭环测试通过 |
| `WRK-07` | P1 | todo | devcontainer / bootstrap hooks | 新项目初始化脚本测试通过 |

### 4.6 Git 与项目视图

| ID | 优先级 | 状态 | 事项 | 验收指标 |
| --- | --- | --- | --- | --- |
| `GIT-01` | P0 | done | diff viewer | 已有独立前端 diff 组件测试，覆盖字段标签、前后值、风险 badge 和空 diff 提示 |
| `GIT-02` | P1 | done | commit / push / branch switch | Git 操作链路有保护和回显，集成测试通过 |
| `GIT-03` | P1 | done | global git status dashboard | 多项目状态面板测试通过 |
| `GIT-04` | P1 | done | commit history view | 历史视图支持筛选与详情 |
| `GIT-05` | P1 | done | project grouping | Sessions 与 Config 页面已按项目分组展示，并有 Web / App 测试覆盖 |
| `GIT-06` | P1 | done | file explorer | Git 面板已展示只读文件树与相对路径预览，大仓库会给出截断提示，并通过 Rust/Web/visual verify 回归 |
| `GIT-07` | P1 | done | file preview / editor | Git 文件树支持按需只读预览，包含路径逃逸保护与只读 editor 测试 |
| `GIT-08` | P1 | done | browser preview | `web/package.json` 已提供正式 `preview/browser` 脚本，Playwright E2E 统一复用 build 后的 preview 链路 |

### 4.7 配置、provider 与密钥治理

| ID | 优先级 | 状态 | 事项 | 验收指标 |
| --- | --- | --- | --- | --- |
| `CFG-01` | P0 | done | `GitHub Copilot CLI` 安全写回 | 配置编辑、校验、备份、回滚测试通过 |
| `CFG-02` | P0 | done | `Factory Droid` 安全写回 | 配置编辑、校验、备份、回滚测试通过 |
| `CFG-03` | P0 | done | `Gemini CLI` 安全写回 | 写回 `.env/settings.json` 的风险保护测试通过 |
| `CFG-04` | P0 | done | `OpenClaw` 安全写回 | 写回保护和 masked diff 测试通过 |
| `CFG-05` | P0 | done | 统一 provider presets | 支持写回的配置卡片已提供统一 preset catalog、套用和恢复检测值，并有 Web 测试覆盖 |
| `CFG-06` | P1 | done | provider import / export | 配置编辑器已支持稳定 snippet schema 的导出与导入，Web 测试覆盖 JSON schema、导出、导入与审计链路 |
| `CFG-07` | P1 | todo | provider health monitor | provider 连通性与健康状态探测测试通过 |
| `CFG-08` | P1 | todo | proxy / failover / circuit breaker | 失败切换和熔断测试通过 |
| `CFG-09` | P1 | todo | request rectifier | 请求修正规则测试通过 |
| `CFG-10` | P2 | todo | tray quick switch | 桌面快捷切换测试通过 |
| `CFG-11` | P1 | done | auto backup | 配置写回前会自动生成备份 manifest，Rust 审计与 Web Audit 页都能看到该路径 |
| `CFG-12` | P2 | todo | cloud sync | 同步冲突与脱机恢复测试通过 |
| `CFG-13` | P1 | done | shared config snippets | 配置片段可保存到本地 snippet library、重新应用到草稿，并把 save/apply/export/import 写入审计历史 |

### 4.8 MCP、skills、prompts

| ID | 优先级 | 状态 | 事项 | 验收指标 |
| --- | --- | --- | --- | --- |
| `MCP-01` | P1 | done | MCP server 暴露 OSM 会话数据 | `mcp` 子命令已通过 `stdio` 暴露 `list/search/get` tools，并有端到端测试覆盖 |
| `MCP-02` | P1 | done | MCP server viewer | Config 页面已展示 MCP server 列表、状态、传输方式、命令与原始配置片段 |
| `MCP-03` | P1 | todo | unified MCP management | 增删改查与校验测试通过 |
| `MCP-04` | P1 | todo | prompts panel | prompt 浏览、编辑和版本切换测试通过 |
| `MCP-05` | P1 | todo | skills panel / installer | 本地 skills 发现、安装和启停测试通过 |
| `MCP-06` | P2 | todo | skills marketplace | 仓库索引、许可提示和安装流程测试通过 |
| `MCP-07` | P1 | todo | prompt / rule cross-app sync | 多助手规则同步测试通过 |
| `MCP-08` | P0 | done | 会话知识提炼成技能或规则 | 会话详情可把摘要、待办、风险和证据提炼成 rule/skill Markdown，并支持预览切换 |
| `MCP-09` | P0 | done | cleanup checklist / session-end hooks | 导出会生成结构化 cleanup checklist，并在项目内存在 `session-end` hook 时执行；软删除前要求 checklist 成功且最近一次 hook 未失败 |

### 4.9 分析与可视化

| ID | 优先级 | 状态 | 事项 | 验收指标 |
| --- | --- | --- | --- | --- |
| `ANA-01` | P0 | done | pricing lookup | 本地价格目录已落地；支持 `reported / estimated / unknown` 成本来源，并有 Rust / Web / CLI 测试覆盖 |
| `ANA-02` | P0 | done | usage timeline | dashboard snapshot 与 Web overview 已展示日级 usage timeline，并有聚合与 golden 回归测试 |
| `ANA-03` | P1 | done | model / platform breakdown | 总览页已展示 model breakdown 与 provider/platform breakdown，并有 Web 聚合测试覆盖 |
| `ANA-04` | P2 | todo | contribution graph | 周期热力图测试通过 |
| `ANA-05` | P2 | todo | leaderboards / shareable stats | 导出图表和分享数据测试通过 |

### 4.10 自动化与对外接口

| ID | 优先级 | 状态 | 事项 | 验收指标 |
| --- | --- | --- | --- | --- |
| `API-01` | P1 | done | REST API | 本地只读 `serve` 壳层已暴露 `health/list/search/get/view/expand`，并有分页、筛选、Bearer 鉴权集成测试覆盖 |
| `API-02` | P1 | done | HTTP control API | 本地 `serve` 已暴露 `resume/pause/attach/detach/continue` POST 路由，并通过 Bearer 鉴权与 OpenAPI 集成测试 |
| `API-03` | P1 | done | OpenAPI 文档 | `/openapi.json` 已暴露本地 REST API 的 OpenAPI 3.1 文档，`http_api` 已校验核心路由与示例字段 |
| `API-04` | P1 | done | agent automation server | `serve` 已支持 `POST /api/v1/automation/tasks` 与 `GET /api/v1/automation/tasks/{taskId}`，首批覆盖 `snapshot.refresh / sessions.search / sessions.resume / sessions.continue`，并通过回执集成测试 |
| `API-05` | P1 | done | robot / json mode | CLI 已支持显式 `--json` 紧凑输出，HTTP 默认返回稳定 JSON，`cli_snapshot/http_api` 已覆盖 |
| `API-06` | P1 | done | `list/search/get/view/expand` 对外接口 | Tauri command 与 CLI 已共享 `commands::query` 查询层，桌面单测与 CLI 集成测试均已覆盖 |
| `API-07` | P2 | done | Prometheus metrics | `serve` 已暴露 `/metrics` Prometheus 文本指标，并通过鉴权与集成测试覆盖 |
| `API-08` | P0 | done | health / doctor checks | `doctor` 可发现关键环境问题 |
| `API-09` | P1 | done | 远程壳层鉴权 | `serve` 已支持 loopback-only 短期本地签名令牌发行，受保护路由可同时接受静态 Bearer 与短期签名 token，并有过期/集成测试覆盖 |

### 4.11 质量、诊断与运维

| ID | 优先级 | 状态 | 事项 | 验收指标 |
| --- | --- | --- | --- | --- |
| `QLT-01` | P0 | done | metadata repair / self-healing | `Claude Code` 缺失 `sessionId` 的历史 JSONL 可按 UUID 文件名自愈，并有回归测试与全量验证 |
| `QLT-02` | P0 | done | unknown session diagnostics | 已知会话根目录下被静默过滤的未知 session-like 文件会进入 `doctor` 诊断输出，而不是直接消失 |
| `QLT-03` | P0 | done | fixture drift ledger | fixtures 已有版本/来源/hash ledger，且统一 verify 会强制检查 drift |
| `QLT-04` | P1 | done | screenshot automation | `e2e:visual` 已生成并校验总览页与会话详情页截图基线 |
| `QLT-05` | P1 | done | config hot reload | 配置变更自动刷新测试通过 |
| `QLT-06` | P2 | todo | desktop auto update | 更新检查与回退测试通过 |
| `QLT-07` | P2 | todo | tray integration | 托盘菜单和状态同步测试通过 |
| `QLT-08` | P1 | todo | 多 profile / 多实例 | profile 切换与隔离测试通过 |

### 4.12 交互与体验

| ID | 优先级 | 状态 | 事项 | 验收指标 |
| --- | --- | --- | --- | --- |
| `UX-01` | P1 | done | 移动端适配 | Playwright 已增加 `mobile-chrome` 项目，并通过移动视口 E2E 验证 |
| `UX-02` | P1 | todo | 远程访问壳层 | 浏览器跨设备访问测试通过 |
| `UX-03` | P1 | todo | terminal panel | 终端面板交互测试通过 |
| `UX-04` | P0 | done | richer diff / review flows | 配置写回前已支持 masked diff、风险提示和显式确认；会话移入隔离区前也要求 cleanup 审查确认 |
| `UX-05` | P1 | todo | uploads and previews | 图片、文档和附件预览测试通过 |
| `UX-06` | P1 | done | richer filters | Sessions 页面已支持 assistant / project / risk / export / control 组合筛选，并有 Web 测试覆盖 |
| `UX-07` | P0 | done | responsive detail panes | 会话详情已改为非 sticky、单列 detail card 布局，并有 E2E 覆盖高 DPI 与窄窗场景 |

## 5. P0 执行顺序

P0 不是“最想做”，而是会直接决定这个产品像不像一个能发布的软件。

1. `UX-07` 会话详情布局与窗口自适应
2. `CFG-01` 到 `CFG-04` 配置安全写回
3. `MCP-09` cleanup checklist / session-end hooks
4. `SRCH-01`、`SRCH-02`、`SRCH-09`、`SRCH-10` 搜索和索引地基
5. `SES-02`、`SES-03`、`SES-07` 会话恢复主链路
6. `WRK-01` worktree 生命周期
7. `API-08` doctor checks
8. `QLT-01` 到 `QLT-03` 修复与 drift 体系

## 6. Git 驱动 TDD 协议

从这份 spec 开始，所有 backlog item 都必须遵守同一套流程：

1. 先写失败测试，再写生产代码
2. 每个 item 都绑定唯一 ID，例如 `CFG-01`
3. 每个阶段都留下 Git 证据，而不是只靠终端输出

### 6.1 固定阶段

- `plan`：确认 item 的验收范围和测试入口
- `red`：测试先失败
- `green`：最小实现转绿
- `verify`：跑全量相关验证
- `review`：本地审查 diff、风险和文档

### 6.2 Git 证据

每个阶段至少留下以下内容：

- `git status --short --branch`
- `git diff --stat`
- `git log -1 --oneline`
- annotated tag
- git note
- review snapshot

### 6.3 约定

- annotated tag 命名：`osm/tdd/<ITEM>/<PHASE>/<TIMESTAMP>`
- git note ref：`refs/notes/osm-tdd`
- review snapshot 默认目录：`.git/osm/reviews/`
- commit message 必须带 item ID，例如：`feat(config): add safe copilot writeback [CFG-01]`

### 6.4 最低命令模板

```powershell
node scripts/git-review-snapshot.mjs --item CFG-01 --phase red --command "npm --prefix web run test -- config-risk-panel.test.tsx"
node scripts/git-tdd-checkpoint.mjs --item CFG-01 --phase red --note "写回表单测试先失败"

node scripts/git-review-snapshot.mjs --item CFG-01 --phase green --command "npm --prefix web run test -- config-risk-panel.test.tsx"
node scripts/git-tdd-checkpoint.mjs --item CFG-01 --phase green --note "最小写回实现已转绿"

node scripts/git-review-snapshot.mjs --item CFG-01 --phase verify --command "powershell -ExecutionPolicy Bypass -File scripts/verify.ps1"
node scripts/git-tdd-checkpoint.mjs --item CFG-01 --phase verify --note "相关验证通过"
```

## 7. 本轮实现范围

这份 spec 不假装“现在已经全部做完”。本轮真实要交付的是：

- 把全部未完成事项落成统一 backlog
- 把 Git 驱动 TDD 协议写成工具和测试
- `SRCH-01 / SRCH-09` 的 SQLite 索引缓存和增量更新
- `SES-02 / SES-03 / SES-07` 的 7 助手真实会话控制链路与 Web 恢复入口
- `WRK-01` 的 repo-local git worktree lifecycle CLI
- 用这套流程继续驱动后续 P0 条目，而不是继续口头追踪

## 7.1 2026-03-23 已验收增量

- `SES-05`：`pause_session` 已接入动作层、Tauri 命令层和快照序列化，`continue` 在 paused 状态下会被显式拦截。
- `SES-06`：`session_control_state` 已持久化 `paused/process_state/process_id/exit_code/started_at/runtime_seconds/event_count/input_tokens/output_tokens/total_tokens/last_activity_at`。
- `MON-02`：Web `Session Detail` 与 `Active Session Cockpit` 已展示 paused/live HUD 字段，中英文文案已补齐。
- `API-02`：`serve` 已支持 `POST /api/v1/sessions/{sessionId}/resume|pause|attach|detach|continue`，并把结果回读成最新 session detail。
- `SES-02 / SES-03 / SES-07`：`GitHub Copilot CLI / OpenCode` 已接入真实 resume / continue 控制链路，snapshot 与 HTTP API 能把它们识别为可控会话。
- 验证命令：
  - `cargo test --manifest-path src-tauri/Cargo.toml --lib`
  - `cargo test --manifest-path src-tauri/Cargo.toml --test http_api`
  - `npx vitest run src/lib/api.test.ts src/components/active-session-cockpit.test.tsx src/components/session-detail.test.tsx`

## 7.2 2026-03-24 已验收增量

- `SES-02 / SES-03 / SES-07`：`Gemini CLI / Factory Droid / OpenClaw` 已接入真实 resume / continue 控制链路；snapshot、HTTP API 和 Web 详情页已统一视为可控会话。
- `Gemini CLI` 的 resume 走本地 checkpointing / `--resume` 证据链；`Factory Droid` 走 `droid exec -s <sessionId>`；`OpenClaw` 走 `openclaw agent --session-id <sessionId> --message <prompt>`。

## 8. 何时可以说“具备发布条件”

至少要同时满足：

- P0 条目全部完成，状态不再是 `todo/partial`
- Windows 11 真机、WSL 多发行版、Linux 桌面环境验证完成
- 搜索、配置写回、恢复、worktree、doctor 和修复链路具备测试与回归证据
- 发布文档、支持矩阵、致谢页、使用说明与真实功能一致

在这些条件之前，不能对外声称“已经完全吸收全部竞品能力”。
