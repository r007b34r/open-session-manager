# Search Resume Worktree Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 OSM 补齐可复用会话索引、增量更新、真实会话恢复与继续运行，以及 Git worktree 生命周期工具链。

**Architecture:** 复用现有 SQLite 审计数据库，把索引缓存、会话控制状态和 worktree 操作证据都落进同一套本地持久层。后端先做真实能力，再把 SES-07 的一键恢复和继续提示接入 Web 详情面板。

**Tech Stack:** Rust, rusqlite, Tauri commands, React, Vitest, Playwright, Node test

---

### Task 1: SRCH-01 / SRCH-09 索引缓存模型

**Files:**
- Modify: `src-tauri/src/storage/schema.sql`
- Modify: `src-tauri/src/storage/sqlite.rs`
- Modify: `src-tauri/src/storage/tests.rs`
- Modify: `src-tauri/src/preferences.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`

**Step 1: Write the failing tests**

- `src-tauri/src/storage/tests.rs`
- `src-tauri/src/commands/dashboard.rs`

Cover:
- 第二次 snapshot 对未变化 session 走 cache hit
- 单个 session 文件变化时只重建对应条目
- 运行后持久化最新 index stats

**Step 2: Run test to verify it fails**

Run:
- `cargo test storage::tests::bootstrap_creates_expected_tables -- --exact`
- `cargo test commands::dashboard::tests::local_snapshot_reuses_cached_index_for_unchanged_sessions -- --exact`
- `cargo test commands::dashboard::tests::local_snapshot_reindexes_only_changed_sessions_incrementally -- --exact`

**Step 3: Write minimal implementation**

- 为 SQLite schema 增加 session index cache 与 index run stats 表
- 在 dashboard snapshot 构建中按 `assistant + environment + source_path + size + modified_at` 复用缓存
- 对缺失或变更文件重建，并记录 cache hit/miss/reindex/stale prune

**Step 4: Run test to verify it passes**

Run:
- `cargo test commands::dashboard::tests::local_snapshot_reuses_cached_index_for_unchanged_sessions -- --exact`
- `cargo test commands::dashboard::tests::local_snapshot_reindexes_only_changed_sessions_incrementally -- --exact`
- `cargo test storage::tests -- --test-threads=1`

**Step 5: Commit**

```bash
git add src-tauri/src/storage/schema.sql src-tauri/src/storage/sqlite.rs src-tauri/src/storage/tests.rs src-tauri/src/preferences.rs src-tauri/src/commands/dashboard.rs
git commit -m "feat(search): add persistent incremental session index cache [SRCH-01][SRCH-09]"
```

### Task 2: SES-02 / SES-03 会话恢复与继续运行

**Files:**
- Create: `src-tauri/src/actions/session_control.rs`
- Modify: `src-tauri/src/actions/mod.rs`
- Modify: `src-tauri/src/commands/actions.rs`
- Modify: `src-tauri/src/desktop.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`
- Modify: `src-tauri/src/storage/schema.sql`
- Modify: `src-tauri/src/storage/sqlite.rs`
- Modify: `src-tauri/src/actions/tests.rs`
- Modify: `src-tauri/tests/cli_snapshot.rs`

**Step 1: Write the failing tests**

Cover:
- 对 Codex/Claude 的真实 resume command 组装
- `continue` prompt 会落持久化状态和 audit
- snapshot 能暴露 session control availability 与最近结果

**Step 2: Run test to verify it fails**

Run:
- `cargo test actions::tests::resumes_supported_session_and_records_control_state -- --exact`
- `cargo test actions::tests::continues_attached_session_and_persists_audit_event -- --exact`
- `cargo test --test cli_snapshot snapshot_command_exposes_session_control_state -- --exact`

**Step 3: Write minimal implementation**

- 建立 assistant-specific session control adapter
- 先支持本机已可确认的 `codex` / `claude-code`
- 通过非交互 resume/continue 命令执行真实会话 ID
- 记录控制状态、最近 prompt / response、最近失败原因和审计事件

**Step 4: Run test to verify it passes**

Run:
- `cargo test actions::tests::resumes_supported_session_and_records_control_state -- --exact`
- `cargo test actions::tests::continues_attached_session_and_persists_audit_event -- --exact`
- `cargo test --test cli_snapshot snapshot_command_exposes_session_control_state -- --exact`

**Step 5: Commit**

```bash
git add src-tauri/src/actions/session_control.rs src-tauri/src/actions/mod.rs src-tauri/src/commands/actions.rs src-tauri/src/desktop.rs src-tauri/src/commands/dashboard.rs src-tauri/src/storage/schema.sql src-tauri/src/storage/sqlite.rs src-tauri/src/actions/tests.rs src-tauri/tests/cli_snapshot.rs
git commit -m "feat(session): add resume and continue control plane [SES-02][SES-03]"
```

### Task 3: SES-07 Web 一键恢复与继续提示

**Files:**
- Modify: `web/src/lib/api.ts`
- Modify: `web/src/app.tsx`
- Modify: `web/src/components/session-detail.tsx`
- Modify: `web/src/components/session-detail.test.tsx`
- Modify: `web/src/app.test.tsx`
- Modify: `tests/e2e/open-session-manager.spec.ts`
- Modify: `web/src/lib/i18n.tsx`

**Step 1: Write the failing tests**

Cover:
- 详情面板显示 resume availability 和最近控制结果
- 点击一键恢复会刷新当前详情
- 输入继续提示后会显示最近一次继续运行结果

**Step 2: Run test to verify it fails**

Run:
- `npm --prefix web run test -- session-detail.test.tsx`
- `npm --prefix web run test -- app.test.tsx`

**Step 3: Write minimal implementation**

- 为 session detail 接入 resume button、continue prompt textarea 和 control status card
- 仅在 native command 可用时启用真实操作
- 浏览器纯静态模式明确显示不可用，而不是假装成功

**Step 4: Run test to verify it passes**

Run:
- `npm --prefix web run test -- session-detail.test.tsx`
- `npm --prefix web run test -- app.test.tsx`
- `npm --prefix web run e2e`

**Step 5: Commit**

```bash
git add web/src/lib/api.ts web/src/app.tsx web/src/components/session-detail.tsx web/src/components/session-detail.test.tsx web/src/app.test.tsx tests/e2e/open-session-manager.spec.ts web/src/lib/i18n.tsx
git commit -m "feat(web): add one-click resume controls [SES-07]"
```

### Task 4: WRK-01 Git worktree lifecycle

**Files:**
- Create: `scripts/git-worktree-manager.mjs`
- Create: `tests/git-workflow/git-worktree-manager.test.mjs`
- Modify: `scripts/verify.ps1`
- Modify: `README.md`

**Step 1: Write the failing tests**

Cover:
- create 在仓库内 `.worktrees/` 创建分支工作树
- merge 回主分支并保留明确结果
- delete 安全移除 worktree
- recycle 对已存在 clean worktree 复用，对 stale entry 先 prune 再重建

**Step 2: Run test to verify it fails**

Run:
- `node --test tests/git-workflow/git-worktree-manager.test.mjs`

**Step 3: Write minimal implementation**

- 增加统一 CLI：`create` / `merge` / `delete` / `recycle`
- 默认 worktree 根目录为仓库内 `.worktrees/`
- 用 porcelain 输出给后续前端或自动化复用

**Step 4: Run test to verify it passes**

Run:
- `node --test tests/git-workflow/git-worktree-manager.test.mjs`

**Step 5: Commit**

```bash
git add scripts/git-worktree-manager.mjs tests/git-workflow/git-worktree-manager.test.mjs scripts/verify.ps1 README.md
git commit -m "feat(worktree): add lifecycle manager CLI [WRK-01]"
```

### Task 5: 最终验证与文档同步

**Files:**
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`
- Modify: `docs/release/open-source-attribution.md`

**Step 1: Run full verification**

Run:
- `cargo test -- --test-threads=1`
- `cargo test --test cli_snapshot`
- `npm --prefix web run test`
- `npm --prefix web run e2e`
- `node --test tests/git-workflow/git-worktree-manager.test.mjs`
- `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`

**Step 2: Update spec and release language**

- 把 `SRCH-01` / `SRCH-09` / `SES-02` / `SES-03` / `SES-07` / `WRK-01` 的状态与验收更新到位
- 明确本轮真实吸收自竞品的能力边界

**Step 3: Commit**

```bash
git add docs/specs/2026-03-22-autonomous-git-tdd-program.md docs/release/support-matrix.md docs/release/github-release-notes.md docs/release/open-source-attribution.md README.md
git commit -m "docs: record search session control and worktree tranche"
```
