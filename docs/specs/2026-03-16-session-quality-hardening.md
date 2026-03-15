# Session Quality Hardening Spec

**Date:** 2026-03-16
**Owner:** `r007b34r`
**Scope:** `Codex` 标题真实性修复 + `Claude Code` 会话筛选纠偏 + 会话列表可辨识性增强

---

## 1. Problem

当前 OSM 已经能发现更多助手，但真实本地数据质量仍然不够，直接影响可用性：

- `Codex` 会话经常把 `AGENTS.md`、`permissions instructions`、`environment_context` 误识别成真实主题。
- `Claude Code` 会把仅包含 `file-history-snapshot` 的 JSONL 也当成候选会话文件，导致发现阶段出现大量噪声。
- 会话详情和会话列表会因为这些误判而呈现大量重复、超长、低价值文本，用户看起来像“选不中正确会话”。
- 搜索能力虽然存在，但因为标题、摘要、transcript highlights 被元提示污染，无法帮助用户定位真实主题。

## 2. Evidence

本轮依据真实本地样本，而不是假设：

- `C:\Users\Max\.codex\sessions\2026\02\05\...019c2d3f...jsonl`
  - 首条用户消息是 `# AGENTS.md instructions ...`
  - 这不是用户真实任务，但当前实现会直接拿它当标题。
- `C:\Users\Max\.claude\projects\C--Users-Max\...e64bead8....jsonl`
  - 文件内容只有 `file-history-snapshot`
  - 当前实现会进入解析并在 verbose 发现模式下产生噪声警告。
- 真实 `snapshot` 输出显示大量 `codex` 会话标题收敛成同一类 `AGENTS.md instructions ...`

## 3. Goals

本轮必须做到：

1. `Codex` 标题、摘要、transcript highlights 优先反映真实用户任务，而不是启动脚手架。
2. `Claude Code` 发现阶段跳过纯 `file-history-snapshot` 文件，不再把它们当成候选会话。
3. 会话列表对“同标题但不同会话”的区分度提升，用户能更容易定位目标会话。
4. 针对上述根因补上回归测试，避免后续吸收更多助手时再次退化。

## 4. Non-Goals

本轮不承诺：

- 全文搜索/BM25/语义搜索完整上线
- 会话恢复/attach/process control
- provider/MCP/skills 治理全量接入
- Cursor/Cline/Kimi 等更多连接器

## 5. Functional Requirements

### 5.1 Meaningful Goal Extraction

对 `Codex` 和其他使用 transcript 推导标题的助手，必须跳过以下脚手架文本：

- `# AGENTS.md instructions ...`
- `<environment_context> ...`
- `<permissions instructions> ...`
- 纯提示词装配块、技能目录块、环境注入块
- `[Request interrupted by user]`

标题应落到第一个真实用户意图上。

### 5.2 Claude Discovery Filtering

`Claude Code` 发现阶段必须只保留真正含会话语义的 JSONL：

- 至少存在 `sessionId`
- 或存在真实 `user/assistant/system/progress` 会话记录
- 纯 `file-history-snapshot` 文件必须被排除

### 5.3 Detail and Search Quality

当脚手架文本被识别出来时：

- 不应进入 transcript highlights 前列
- 不应成为 title / summary / key artifacts 的主要内容
- 搜索命中应更容易落在真实任务文本

### 5.4 List Disambiguation

Sessions 列表需要增加稳定且易辨识的次级标识，避免多个同标题会话在视觉上完全相同。

## 6. Acceptance Criteria

满足以下条件才可宣称本轮完成：

1. 新增回归测试覆盖：
   - `Codex` 脚手架标题污染
   - `Claude Code` file-history-only 发现误收
2. `cargo test --lib` 通过
3. `cargo test --test cli_snapshot` 通过
4. 真实本地 `snapshot` 中，`Codex` 标题不再被 `AGENTS.md instructions` 主导
5. verbose 发现模式下，不再对纯 `file-history-snapshot` 文件刷大量误告警

