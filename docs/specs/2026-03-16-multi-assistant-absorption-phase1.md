# Multi-Assistant Absorption Phase 1 Spec

**Date:** 2026-03-16
**Owner:** `r007b34r`
**Scope:** `Gemini CLI` + `GitHub Copilot CLI` + `Factory Droid`

---

## 1. Problem

OSM 现在真正可用的会话适配器只有 `Codex`、`Claude Code`、`OpenCode`。这与上游竞品相比存在明显缺口，尤其是在用户已经明确要求的多助手覆盖面上。

当前问题不是“缺少描述”，而是“缺少真实功能”：

- 本地扫描无法识别 `Gemini CLI`、`GitHub Copilot CLI`、`Factory Droid`
- dashboard 无法展示这些助手的真实主题、摘要、进度和来源
- README 和发布材料中提到的“吸收上游能力”缺少对应落地
- 支持矩阵与实际功能不一致，影响发布可信度

## 2. Upstream Evidence

本轮吸收基于真实源码和文档，不基于二手总结。

### 主参考项目

- `jazzyalex/agent-sessions`
  - 价值：当前最接近 OSM 目标的本地多助手会话索引器
  - 已验证路径：
    - `AgentSessions/Services/GeminiSessionParser.swift`
    - `AgentSessions/Services/GeminiSessionDiscovery.swift`
    - `AgentSessions/Services/CopilotSessionParser.swift`
    - `AgentSessions/Services/DroidSessionParser.swift`
    - `AgentSessions/Services/DroidSessionDiscovery.swift`
    - `docs/agent-json-tracking.md`
- `kbwo/ccmanager`
  - 价值：证明市场上用户对多助手支持面的预期已经更高
  - 结论：可作为覆盖面基准，但并不是 OSM 的 transcript 治理主实现参考
- `farion1231/cc-switch`
  - 价值：配置治理层的后续吸收对象
  - 结论：本轮先不直接并入会话解析逻辑，后续吸收其 `Gemini / OpenClaw` 配置面板与配置读写策略

### 已确认的真实格式证据

- `Gemini CLI`
  - 根目录：`~/.gemini/tmp/<projectHash>/chats/session-*.json`
  - 回退目录：`~/.gemini/tmp/<projectHash>/session-*.json`
  - 形态：JSON 单文件，可能是对象 + `messages`，也可能是根数组或 `history`
- `GitHub Copilot CLI`
  - 根目录：`~/.copilot/session-state/<sessionId>.jsonl`
  - 形态：JSONL，事件包 `{ type, data, id, timestamp, parentId }`
- `Factory Droid`
  - 会话仓库：`~/.factory/sessions/**/<sessionId>.jsonl`
  - 流式日志：`~/.factory/projects/**/*.jsonl`
  - 形态：两种方言
    - session store：`session_start` + `message.content[]`
    - stream-json：`system|message|tool_call|tool_result|completion|error`

## 3. Product Goals

本轮必须真实达成：

1. OSM 能发现三种新增助手的本地会话文件。
2. OSM 能把这些会话解析成统一 `SessionRecord`。
3. dashboard 能生成标题、摘要、进度、风险与 transcript 摘要，不再报 `UnsupportedAssistant`。
4. fixtures、单测、snapshot CLI 覆盖新增助手。
5. README、支持矩阵、研究索引、发布说明反映真实实现结果。

## 4. Non-Goals

本轮不承诺：

- `OpenClaw` 完整会话支持
- `Gemini` / `Copilot` / `Droid` 的配置审计写回
- 大规模全文检索、语义搜索、索引数据库重构
- 配置 UI 改写和第三方 provider 编辑器

这些内容进入下一阶段吸收，不在本轮混入，避免再次出现“写得很多，功能却没落地”的问题。

## 5. Functional Requirements

### 5.1 Session Discovery

OSM 必须新增以下发现能力：

- `gemini-cli`
  - Windows / Linux：`<home>/.gemini/tmp/**/session-*.json`
  - 优先覆盖 `chats/` 子目录
- `github-copilot-cli`
  - Windows / Linux：`<home>/.copilot/session-state/*.jsonl`
- `factory-droid`
  - Windows / Linux：`<home>/.factory/sessions/**/*.jsonl`
  - Windows / Linux：`<home>/.factory/projects/**/*.jsonl`
  - 对 `projects/**/*.jsonl` 需要做最小识别，避免误收无关 JSONL

WSL 路径模型也必须同步扩展。

### 5.2 Session Parsing

每个新增适配器都必须输出稳定的：

- `session_id`
- `assistant`
- `project_path`
- `source_path`
- `started_at`
- `last_activity_at`
- `message_count`
- `tool_count`
- `raw_format`
- `content_hash`

### 5.3 Dashboard Narrative

每个新增助手都必须能提取：

- 第一条真实用户目标
- 最后一条有效 assistant 输出
- 错误数量线索
- 进而生成：
  - 标题
  - 摘要
  - 进度状态
  - 风险标记
  - transcript highlights

### 5.4 Transcript Digest

本轮 transcript 目标是“最小但真实可用”：

- `Gemini CLI`
  - 支持 user / assistant / toolCalls 文本抽取
- `GitHub Copilot CLI`
  - 支持 user.message / assistant.message / tool.execution_complete
- `Factory Droid`
  - 支持 session store 文本部分和 stream-json 主事件

不追求一次性完整复刻上游全部 event 语义，但必须让 OSM 的清理、导出、详情页和价值判断不再退化为未知会话。

## 6. Architecture Decisions

### 6.1 Keep OSM Domain Stable

不重写领域模型。继续复用：

- `SessionRecord`
- `SessionInsight`
- `TranscriptDigest`
- `DashboardSnapshot`

新增助手通过 adapter 和 narrative/transcript 分支接入，不引入第二套会话模型。

### 6.2 Clean-Room Absorption

吸收方式采用“读上游、重写实现、保留可追溯关系”：

- 不复制 MIT 项目的源码块进 OSM
- 在 catalog 和发布说明中记录吸收来源
- fixture 只做格式级模拟，不直接照搬完整真实会话

### 6.3 TDD-First

本轮必须按测试先行：

1. 先写 adapter fixture tests
2. 跑出 fail
3. 再写实现
4. 再补 dashboard / snapshot tests
5. 最后跑全量验证

## 7. Acceptance Criteria

满足以下条件才可宣称本轮完成：

1. `cargo test --lib` 通过
2. `cargo test --test cli_snapshot` 通过
3. 新增三类 fixture adapter tests 全部通过
4. fixture snapshot 中会话总数从 `3` 提升到 `6`
5. snapshot 输出不再对三类新增助手报 `UnsupportedAssistant`
6. README 与支持矩阵明确写出新增真实支持面
7. 本地 git commit 完成并 push 到远程分支

## 8. Risks

### 8.1 Format Drift

`Gemini` 和 `Droid` 的格式漂移概率高，因此实现必须尽量容忍：

- 多种时间字段
- 多种 root shape
- 蛇形 / 驼峰字段并存
- 非文本消息与空文本消息

### 8.2 Over-collection

`~/.factory/projects/**/*.jsonl` 很容易误收普通日志，因此必须引入最小识别逻辑，不可只凭扩展名收录。

### 8.3 Narrative Gaps

如果 transcript 提取过于简化，会导致详情页只能看到“Indexed transcript from ...”。这会直接削弱 OSM 的清理判断力，因此 dashboard narrative 与 transcript digest 必须和 adapter 一起落地。

## 9. Release Impact

本轮完成后，OSM 的真实会话支持面将从：

- `Codex`
- `Claude Code`
- `OpenCode`

提升为：

- `Codex`
- `Claude Code`
- `OpenCode`
- `Gemini CLI`
- `GitHub Copilot CLI`
- `Factory Droid`

这会是第一次真正把 OSM 推到“多助手会话治理台”的主赛道，而不是只做单点 viewer。
