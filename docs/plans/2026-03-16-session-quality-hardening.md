# Session Quality Hardening Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 修复会话识别质量根因，让 OSM 对真实本地会话具备可操作的标题、摘要和筛选结果。

**Architecture:** 在不重写领域模型的前提下，新增一层轻量的会话文本清洗规则，分别接入 narrative 提取和 transcript digest。Claude 发现逻辑做轻量预筛选，避免纯快照日志进入解析链路。UI 只做最小必要增强，用更清晰的次级标识帮助用户区分会话。

**Tech Stack:** Rust, Tauri, React, Vitest, cargo test

---

### Task 1: 写出会话质量回归测试

**Files:**
- Modify: `src-tauri/src/adapters/tests.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`

**Step 1: 写 `Claude Code` 发现过滤失败测试**

新增测试场景：
- 目录中同时存在一个真实 Claude 会话文件
- 以及一个只含 `file-history-snapshot` 的 JSONL
- 期望 `discover_session_files()` 只返回真实会话

**Step 2: 写 `Codex` 标题污染失败测试**

新增测试场景：
- 一个 `Codex` rollout 文件先写入 `AGENTS.md instructions`
- 再写入 `<environment_context>`
- 再写入真实用户任务
- 期望 snapshot 标题落到真实任务，且 transcript highlights 首项不是脚手架文本

**Step 3: 运行局部测试，确认先失败**

Run: `C:\Users\Max\.cargo\bin\cargo.exe test --lib adapters::tests commands::dashboard::tests`

Expected:
- 至少新增的两个测试失败

**Step 4: Commit**

```bash
git add src-tauri/src/adapters/tests.rs src-tauri/src/commands/dashboard.rs
git commit -m "test: capture session quality regressions"
```

### Task 2: 实现会话文本清洗与 Claude 预筛选

**Files:**
- Create: `src-tauri/src/session_text.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`
- Modify: `src-tauri/src/transcript/mod.rs`
- Modify: `src-tauri/src/adapters/claude_code.rs`

**Step 1: 新增共享文本规则模块**

实现：
- 空白归一化
- 脚手架文本识别
- 无价值占位文本识别
- 供 narrative 和 transcript 共用

**Step 2: narrative 提取改为跳过脚手架**

对 `Codex` 及相邻助手：
- 第一个用户目标必须取第一个“有意义”的用户文本
- assistant 总结保留最后一条有效文本

**Step 3: transcript digest 跳过脚手架 highlights**

对 `Codex`、`Claude Code`：
- AGENTS/environment/interrupt 文本不进入 highlights 前列

**Step 4: Claude 发现增加轻量预筛选**

在 `discover_session_files()` 中：
- 仅保留真正包含 `sessionId` 或真实会话事件的 JSONL
- 排除纯 `file-history-snapshot`

**Step 5: 运行局部测试，确认转绿**

Run: `C:\Users\Max\.cargo\bin\cargo.exe test --lib adapters::tests commands::dashboard::tests`

Expected:
- 新增测试通过

**Step 6: Commit**

```bash
git add src-tauri/src/session_text.rs src-tauri/src/lib.rs src-tauri/src/commands/dashboard.rs src-tauri/src/transcript/mod.rs src-tauri/src/adapters/claude_code.rs
git commit -m "feat: harden session text quality and claude discovery"
```

### Task 3: 增强 Sessions 列表辨识度

**Files:**
- Modify: `web/src/components/session-table.tsx`
- Modify: `web/src/components/session-table.test.tsx`
- Modify: `web/src/styles.css`

**Step 1: 写失败测试**

新增测试断言：
- 会话行会显示稳定的次级标识
- 至少包含 `sessionId` 片段，便于区分同标题会话

**Step 2: 实现最小 UI 增强**

更新列表行：
- 标题下方显示环境 + 会话 ID
- 保持点击行为不退化

**Step 3: 跑前端单测确认通过**

Run: `npm --prefix web run test`

Expected:
- `session-table` 相关测试通过

**Step 4: Commit**

```bash
git add web/src/components/session-table.tsx web/src/components/session-table.test.tsx web/src/styles.css
git commit -m "feat: improve session list disambiguation"
```

### Task 4: 更新研究与发布口径并做验证

**Files:**
- Modify: `docs/research/2026-03-16-full-competitor-gap-analysis.md`
- Modify: `README.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`

**Step 1: 更新研究结论**

补充本轮确认的根因：
- 能力缺口之外，还存在“会话质量识别债”
- 这会直接影响搜索、清理、转存和 UI 可用性

**Step 2: 更新 README/发布说明**

写清：
- 当前真实支持面
- 已修复的会话质量问题
- 尚未完成的能力边界

**Step 3: 跑完整验证**

Run:
- `C:\Users\Max\.cargo\bin\cargo.exe test --lib`
- `C:\Users\Max\.cargo\bin\cargo.exe test --test cli_snapshot`
- `npm --prefix web run test`
- `npm --prefix web run build`

Expected:
- 全部通过

**Step 4: Commit**

```bash
git add README.md docs/research/2026-03-16-full-competitor-gap-analysis.md docs/release/support-matrix.md docs/release/github-release-notes.md
git commit -m "docs: align release story with session quality hardening"
```

### Task 5: 推送远程分支

**Files:**
- None

**Step 1: 检查工作树**

Run: `git status --short`

Expected:
- 工作树干净

**Step 2: 推送**

Run: `git push origin feat/usability-clarity`

Expected:
- 远程分支包含本轮提交

