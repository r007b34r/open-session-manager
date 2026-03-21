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
- 基础 usage / cost 聚合
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
| `ADP-08` | P1 | todo | `Qwen CLI` 会话与配置适配 | fixture + Rust adapter 测试通过 |
| `ADP-09` | P1 | todo | `Roo Code` 会话与配置适配 | fixture + Rust adapter 测试通过 |
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
| `SRCH-01` | P0 | partial | 大历史索引层 | 10k+ session fixture 下可复用索引，CLI snapshot 不做全量全文扫盘 |
| `SRCH-02` | P0 | todo | BM25 排序 | 搜索结果测试验证 BM25 排名稳定 |
| `SRCH-03` | P1 | todo | semantic search | 引入向量索引或嵌入后，语义检索测试通过 |
| `SRCH-04` | P1 | todo | hybrid ranking | lexical + semantic 混合排序测试通过 |
| `SRCH-05` | P0 | partial | snippet preview 深化 | 命中片段支持 transcript 内定位和高亮测试 |
| `SRCH-06` | P1 | todo | search-as-you-type | 前端交互测试验证防抖、取消和增量刷新 |
| `SRCH-07` | P1 | todo | `list/search/get/view/expand` 统一 CLI | CLI 集成测试覆盖五类命令 |
| `SRCH-08` | P1 | todo | 统一 search API | HTTP 或 Tauri command 测试验证分页、过滤和排序 |
| `SRCH-09` | P0 | todo | 大规模索引缓存与增量更新 | 重复启动基准测试能明显快于全量重建 |
| `SRCH-10` | P0 | done | schema drift 监控 | fixture ledger 与 snapshot golden diff 已接入统一 verify，失败时会返回具体 JSON 路径差异 |

### 4.3 会话控制与恢复

| ID | 优先级 | 状态 | 事项 | 验收指标 |
| --- | --- | --- | --- | --- |
| `SES-01` | P1 | todo | 启动新会话 | UI 或 CLI 可启动受支持助手的新会话，集成测试通过 |
| `SES-02` | P0 | todo | 恢复既有会话 | 对真实会话 ID 执行 resume，结果可观测 |
| `SES-03` | P0 | todo | 继续运行中的会话 | attach 后能继续发送消息并落审计记录 |
| `SES-04` | P0 | todo | attach / detach | 会话控制测试覆盖附着与分离 |
| `SES-05` | P1 | todo | pause / resume | 生命周期状态测试通过 |
| `SES-06` | P1 | todo | session process control | 能读取并展示进程状态、退出码与运行时长 |
| `SES-07` | P0 | todo | one-click resume | 前端点击后恢复目标会话，E2E 通过 |
| `SES-08` | P1 | todo | continue rate-limit 策略 | 恢复节流和冲突保护测试通过 |
| `SES-09` | P0 | partial | 标准化 handoff / resume artifact | 导出与恢复之间形成一对一工件引用，测试覆盖 |

### 4.4 活跃会话与实时监控

| ID | 优先级 | 状态 | 事项 | 验收指标 |
| --- | --- | --- | --- | --- |
| `MON-01` | P0 | todo | active session cockpit | 前端展示活跃会话总览，状态数据可刷新 |
| `MON-02` | P1 | todo | live HUD | 实时面板测试覆盖 token、事件、进度更新 |
| `MON-03` | P0 | todo | busy / waiting / idle 状态识别 | 适配器状态机测试通过 |
| `MON-04` | P1 | todo | live TODO / project monitor | TODO 变化触发 UI 刷新和审计事件 |
| `MON-05` | P1 | todo | SSE / WebSocket 实时事件 | 前后端流式测试通过 |
| `MON-06` | P1 | todo | running session diagnostics | 异常会话诊断卡片和日志导出测试通过 |

### 4.5 工作区、worktree 与并行执行

| ID | 优先级 | 状态 | 事项 | 验收指标 |
| --- | --- | --- | --- | --- |
| `WRK-01` | P0 | todo | Git worktree create / merge / delete / recycle | 工作树生命周期测试覆盖创建、复用、回收 |
| `WRK-02` | P1 | todo | 并行 agent 调度 | 调度状态与资源冲突测试通过 |
| `WRK-03` | P1 | todo | session data copy across worktrees | 同一任务切换工作树时能继承必要上下文 |
| `WRK-04` | P1 | todo | task queue / scheduler / cron | 定时任务测试通过 |
| `WRK-05` | P1 | todo | 容器隔离执行 | Docker/Podman 集成测试通过 |
| `WRK-06` | P1 | todo | iteration workflow | inspect/diff/merge/push 的闭环测试通过 |
| `WRK-07` | P1 | todo | devcontainer / bootstrap hooks | 新项目初始化脚本测试通过 |

### 4.6 Git 与项目视图

| ID | 优先级 | 状态 | 事项 | 验收指标 |
| --- | --- | --- | --- | --- |
| `GIT-01` | P0 | todo | diff viewer | 前端可视化 diff 组件测试通过 |
| `GIT-02` | P1 | todo | commit / push / branch switch | Git 操作链路有保护和回显，集成测试通过 |
| `GIT-03` | P1 | todo | global git status dashboard | 多项目状态面板测试通过 |
| `GIT-04` | P1 | todo | commit history view | 历史视图支持筛选与详情 |
| `GIT-05` | P1 | todo | project grouping | 会话和配置按项目聚合，UI 测试通过 |
| `GIT-06` | P1 | todo | file explorer | 文件树和路径预览测试通过 |
| `GIT-07` | P1 | todo | file preview / editor | 预览与只读编辑模式测试通过 |
| `GIT-08` | P1 | todo | browser preview | 本地 Web 预览链路测试通过 |

### 4.7 配置、provider 与密钥治理

| ID | 优先级 | 状态 | 事项 | 验收指标 |
| --- | --- | --- | --- | --- |
| `CFG-01` | P0 | done | `GitHub Copilot CLI` 安全写回 | 配置编辑、校验、备份、回滚测试通过 |
| `CFG-02` | P0 | done | `Factory Droid` 安全写回 | 配置编辑、校验、备份、回滚测试通过 |
| `CFG-03` | P0 | done | `Gemini CLI` 安全写回 | 写回 `.env/settings.json` 的风险保护测试通过 |
| `CFG-04` | P0 | done | `OpenClaw` 安全写回 | 写回保护和 masked diff 测试通过 |
| `CFG-05` | P0 | todo | 统一 provider presets | 预设导入、应用、回滚测试通过 |
| `CFG-06` | P1 | todo | provider import / export | 导入导出 schema 测试通过 |
| `CFG-07` | P1 | todo | provider health monitor | provider 连通性与健康状态探测测试通过 |
| `CFG-08` | P1 | todo | proxy / failover / circuit breaker | 失败切换和熔断测试通过 |
| `CFG-09` | P1 | todo | request rectifier | 请求修正规则测试通过 |
| `CFG-10` | P2 | todo | tray quick switch | 桌面快捷切换测试通过 |
| `CFG-11` | P1 | todo | auto backup | 配置写回前自动备份测试通过 |
| `CFG-12` | P2 | todo | cloud sync | 同步冲突与脱机恢复测试通过 |
| `CFG-13` | P1 | todo | shared config snippets | 共享片段可导入、复用和审计 |

### 4.8 MCP、skills、prompts

| ID | 优先级 | 状态 | 事项 | 验收指标 |
| --- | --- | --- | --- | --- |
| `MCP-01` | P1 | todo | MCP server 暴露 OSM 会话数据 | `list/search/get` 端到端测试通过 |
| `MCP-02` | P1 | todo | MCP server viewer | UI 可查看 server 列表、状态和配置 |
| `MCP-03` | P1 | todo | unified MCP management | 增删改查与校验测试通过 |
| `MCP-04` | P1 | todo | prompts panel | prompt 浏览、编辑和版本切换测试通过 |
| `MCP-05` | P1 | todo | skills panel / installer | 本地 skills 发现、安装和启停测试通过 |
| `MCP-06` | P2 | todo | skills marketplace | 仓库索引、许可提示和安装流程测试通过 |
| `MCP-07` | P1 | todo | prompt / rule cross-app sync | 多助手规则同步测试通过 |
| `MCP-08` | P0 | todo | 会话知识提炼成技能或规则 | Markdown handoff 可转结构化 rule/skill |
| `MCP-09` | P0 | done | cleanup checklist / session-end hooks | 导出会生成结构化 cleanup checklist，并在项目内存在 `session-end` hook 时执行；软删除前要求 checklist 成功且最近一次 hook 未失败 |

### 4.9 分析与可视化

| ID | 优先级 | 状态 | 事项 | 验收指标 |
| --- | --- | --- | --- | --- |
| `ANA-01` | P0 | todo | pricing lookup | 本地价格表或在线同步测试通过 |
| `ANA-02` | P0 | todo | usage timeline | 时序图和时间聚合测试通过 |
| `ANA-03` | P1 | todo | model / platform breakdown | 多模型、多 provider 聚合测试通过 |
| `ANA-04` | P2 | todo | contribution graph | 周期热力图测试通过 |
| `ANA-05` | P2 | todo | leaderboards / shareable stats | 导出图表和分享数据测试通过 |

### 4.10 自动化与对外接口

| ID | 优先级 | 状态 | 事项 | 验收指标 |
| --- | --- | --- | --- | --- |
| `API-01` | P1 | todo | REST API | 集成测试覆盖分页、筛选、鉴权 |
| `API-02` | P1 | todo | HTTP control API | 远程控制会话和配置链路测试通过 |
| `API-03` | P1 | todo | OpenAPI 文档 | schema 生成与示例校验通过 |
| `API-04` | P1 | todo | agent automation server | 任务触发与回执测试通过 |
| `API-05` | P1 | todo | robot / json mode | CLI/HTTP 可输出稳定 JSON 结构 |
| `API-06` | P1 | todo | `list/search/get/view/expand` 对外接口 | API 与 CLI 能共享同一查询层 |
| `API-07` | P2 | todo | Prometheus metrics | 指标暴露和采集测试通过 |
| `API-08` | P0 | done | health / doctor checks | `doctor` 可发现关键环境问题 |
| `API-09` | P1 | todo | 远程壳层鉴权 | token/JWT 或本地授权测试通过 |

### 4.11 质量、诊断与运维

| ID | 优先级 | 状态 | 事项 | 验收指标 |
| --- | --- | --- | --- | --- |
| `QLT-01` | P0 | done | metadata repair / self-healing | `Claude Code` 缺失 `sessionId` 的历史 JSONL 可按 UUID 文件名自愈，并有回归测试与全量验证 |
| `QLT-02` | P0 | done | unknown session diagnostics | 已知会话根目录下被静默过滤的未知 session-like 文件会进入 `doctor` 诊断输出，而不是直接消失 |
| `QLT-03` | P0 | done | fixture drift ledger | fixtures 已有版本/来源/hash ledger，且统一 verify 会强制检查 drift |
| `QLT-04` | P1 | todo | screenshot automation | UI 回归截图脚本和对比测试通过 |
| `QLT-05` | P1 | todo | config hot reload | 配置变更自动刷新测试通过 |
| `QLT-06` | P2 | todo | desktop auto update | 更新检查与回退测试通过 |
| `QLT-07` | P2 | todo | tray integration | 托盘菜单和状态同步测试通过 |
| `QLT-08` | P1 | todo | 多 profile / 多实例 | profile 切换与隔离测试通过 |

### 4.12 交互与体验

| ID | 优先级 | 状态 | 事项 | 验收指标 |
| --- | --- | --- | --- | --- |
| `UX-01` | P1 | todo | 移动端适配 | Playwright 多视口测试通过 |
| `UX-02` | P1 | todo | 远程访问壳层 | 浏览器跨设备访问测试通过 |
| `UX-03` | P1 | todo | terminal panel | 终端面板交互测试通过 |
| `UX-04` | P0 | todo | richer diff / review flows | 审查链路支持 diff、风险提示和确认动作 |
| `UX-05` | P1 | todo | uploads and previews | 图片、文档和附件预览测试通过 |
| `UX-06` | P1 | todo | richer filters | 高级筛选器测试通过 |
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
- 用这套流程驱动后续 P0 条目，而不是继续口头追踪

## 8. 何时可以说“具备发布条件”

至少要同时满足：

- P0 条目全部完成，状态不再是 `todo/partial`
- Windows 11 真机、WSL 多发行版、Linux 桌面环境验证完成
- 搜索、配置写回、恢复、worktree、doctor 和修复链路具备测试与回归证据
- 发布文档、支持矩阵、致谢页、使用说明与真实功能一致

在这些条件之前，不能对外声称“已经完全吸收全部竞品能力”。
