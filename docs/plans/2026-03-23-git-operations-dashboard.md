# Git Operations Dashboard Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为总览页的 Git workspace 面板补齐 `commit / push / branch switch` 操作链路、保护规则和结果回显。

**Architecture:** Rust 侧新增 `git_control` action，统一封装 `git add/commit`、`git switch` 和 `git push`，并把动作结果写入审计事件，Web 侧继续复用 dashboard snapshot 和 audit history 做回显。前端通过新的 API helper 驱动原生命令，在浏览器 demo 模式下提供最小可测试模拟，避免 UI 假装成功。

**Tech Stack:** Rust, Tauri commands, SQLite audit events, React, Vitest, cargo test

---

### Task 1: Rust Git action 基座

**Files:**
- Create: `src-tauri/src/actions/git_control.rs`
- Modify: `src-tauri/src/actions/mod.rs`
- Modify: `src-tauri/src/actions/tests.rs`

**Step 1: Write the failing test**

覆盖：
- dirty repo 可提交并生成 `git_commit` 审计
- clean repo 禁止空提交
- branch switch 在 dirty repo 上被保护，clean repo 可切换或新建分支
- push 在存在 bare remote 和上游分支时成功，并写入 `git_push` 审计

**Step 2: Run test to verify it fails**

Run:
- `cargo test actions::tests::commits_git_project_and_records_audit_event -- --exact`
- `cargo test actions::tests::switches_git_branch_with_dirty_worktree_guardrail -- --exact`
- `cargo test actions::tests::pushes_git_project_to_upstream_and_records_audit_event -- --exact`

**Step 3: Write minimal implementation**

- 新增 `GitCommitRequest / GitBranchSwitchRequest / GitPushRequest`
- 封装 git 命令执行和保护规则
- 结果写入 `audit_events`

**Step 4: Run test to verify it passes**

Run:
- `cargo test actions::tests::commits_git_project_and_records_audit_event -- --exact`
- `cargo test actions::tests::switches_git_branch_with_dirty_worktree_guardrail -- --exact`
- `cargo test actions::tests::pushes_git_project_to_upstream_and_records_audit_event -- --exact`

**Step 5: Commit**

```bash
git add src-tauri/src/actions/git_control.rs src-tauri/src/actions/mod.rs src-tauri/src/actions/tests.rs
git commit -m "feat(git): add protected git project actions [GIT-02]"
```

### Task 2: Tauri / desktop 命令暴露

**Files:**
- Modify: `src-tauri/src/commands/actions.rs`
- Modify: `src-tauri/src/desktop.rs`

**Step 1: Write the failing test**

覆盖：
- desktop command 暴露 `commit_git_project / switch_git_project_branch / push_git_project`
- temp repo 集成测试能返回更新后的 snapshot

**Step 2: Run test to verify it fails**

Run:
- `cargo test desktop::tests::desktop_commands_are_async_futures -- --exact`
- `cargo test desktop::tests::git_project_commands_commit_switch_and_push -- --exact`

**Step 3: Write minimal implementation**

- 注册三个 tauri command
- 命令完成后回建最新 dashboard snapshot

**Step 4: Run test to verify it passes**

Run:
- `cargo test desktop::tests::desktop_commands_are_async_futures -- --exact`
- `cargo test desktop::tests::git_project_commands_commit_switch_and_push -- --exact`

**Step 5: Commit**

```bash
git add src-tauri/src/commands/actions.rs src-tauri/src/desktop.rs
git commit -m "feat(desktop): expose git project commands [GIT-02]"
```

### Task 3: Web Git 操作面板

**Files:**
- Modify: `web/src/lib/api.ts`
- Modify: `web/src/lib/api.test.ts`
- Modify: `web/src/components/git-project-panel.tsx`
- Modify: `web/src/app.test.tsx`
- Modify: `web/src/lib/i18n.tsx`

**Step 1: Write the failing test**

覆盖：
- Git 面板显示 commit message、branch switch、push 操作入口
- 提交成功后展示最新 commit summary
- dirty repo 时 branch switch 显示保护信息
- push 成功后显示最近 Git 动作回显

**Step 2: Run test to verify it fails**

Run:
- `npx vitest run src/app.test.tsx -t "在总览里执行 Git 提交并显示最新结果"`
- `npx vitest run src/app.test.tsx -t "在 dirty 仓库里切换分支前给出保护提示"`
- `npx vitest run src/lib/api.test.ts`

**Step 3: Write minimal implementation**

- 增加 `applyGitProjectCommit / applyGitProjectBranchSwitch / applyGitProjectPush`
- demo 模式下最小模拟 git 结果与 audit 回显
- 面板展示最近 Git 动作和错误/成功状态

**Step 4: Run test to verify it passes**

Run:
- `npx vitest run src/app.test.tsx -t "在总览里执行 Git 提交并显示最新结果"`
- `npx vitest run src/app.test.tsx -t "在 dirty 仓库里切换分支前给出保护提示"`
- `npx vitest run src/lib/api.test.ts src/app.test.tsx`

**Step 5: Commit**

```bash
git add web/src/lib/api.ts web/src/lib/api.test.ts web/src/components/git-project-panel.tsx web/src/app.test.tsx web/src/lib/i18n.tsx
git commit -m "feat(web): add actionable git workspace panel [GIT-02]"
```

### Task 4: 验证与状态同步

**Files:**
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`

**Step 1: Run verification**

Run:
- `cargo test actions::tests::commits_git_project_and_records_audit_event -- --exact`
- `cargo test desktop::tests::git_project_commands_commit_switch_and_push -- --exact`
- `npx vitest run src/lib/api.test.ts src/app.test.tsx`

**Step 2: Update spec**

- 将 `GIT-02` 从 `todo` 更新为真实状态

**Step 3: Commit**

```bash
git add docs/specs/2026-03-22-autonomous-git-tdd-program.md
git commit -m "docs: record git operations dashboard status [GIT-02]"
```
