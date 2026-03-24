# Git File Explorer Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 完成 `GIT-06`，让 Git 项目面板展示只读文件树和相对路径预览，帮助用户快速判断项目上下文。

**Architecture:** 复用现有 `gitProjects` snapshot 链路，在 Rust `dashboard` 构建阶段附带轻量文件树摘要。Web 端只消费统一 snapshot，不新增专用文件读取命令；首版严格限制深度和条目数，避免大仓库把 snapshot 撑爆。

**Tech Stack:** Rust `std::fs`、Tauri dashboard snapshot、React、Vitest、Rust unit tests

---

### Task 1: 扩展 Git 项目快照模型

**Files:**
- Modify: `src-tauri/src/commands/dashboard.rs`
- Modify: `web/src/lib/api.ts`

**Step 1: Write the failing test**

- 在 Rust dashboard 测试里断言 `gitProjects[0].workspaceEntries` 至少包含 `README.md` 和 `src`，且不会把 `.git` 暴露到 UI 快照里。

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml local_snapshot_emits_git_project_status_and_recent_commits --lib -- --exact`

**Step 3: Write minimal implementation**

- 为 `GitProjectRecord` 增加 `workspaceEntries` 和 `workspaceTruncated`
- 在 Git project inspection 阶段递归读取 repo root，生成有序的相对路径条目
- 忽略 `.git`，并限制深度与总条目数

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml local_snapshot_emits_git_project_status_and_recent_commits --lib -- --exact`

**Step 5: Commit**

```bash
git add src-tauri/src/commands/dashboard.rs web/src/lib/api.ts
git commit -m "feat(git): add snapshot workspace explorer data [GIT-06]"
```

### Task 2: 渲染 Git 文件树

**Files:**
- Modify: `web/src/components/git-project-panel.tsx`
- Modify: `web/src/components/git-project-panel.test.tsx`
- Modify: `web/src/lib/i18n.tsx`
- Modify: `web/src/styles.css`

**Step 1: Write the failing test**

- 在 `git-project-panel.test.tsx` 断言项目卡片会展示 `Workspace explorer`、目录/文件条目以及截断提示。

**Step 2: Run test to verify it fails**

Run: `npm --prefix web run test -- src/components/git-project-panel.test.tsx`

**Step 3: Write minimal implementation**

- 在 Git 面板里新增只读 explorer 区块
- 用相对路径和层级缩进展示目录/文件
- 对 `workspaceTruncated` 显示固定提示

**Step 4: Run test to verify it passes**

Run: `npm --prefix web run test -- src/components/git-project-panel.test.tsx`

**Step 5: Commit**

```bash
git add web/src/components/git-project-panel.tsx web/src/components/git-project-panel.test.tsx web/src/lib/i18n.tsx web/src/styles.css
git commit -m "feat(git): render workspace explorer in git panel [GIT-06]"
```

### Task 3: 验证与文档收口

**Files:**
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`

**Step 1: Run focused verification**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml local_snapshot_emits_git_project_status_and_recent_commits --lib -- --exact
npm --prefix web run test -- src/components/git-project-panel.test.tsx
```

**Step 2: Run full verification**

Run: `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`

**Step 3: Update spec**

- 把 `GIT-06` 从 `todo` 改成 `done`
- 在验收指标里写明“Git 面板已展示只读文件树与路径预览，并通过 Rust/Web/verify 回归”

**Step 4: Commit**

```bash
git add docs/specs/2026-03-22-autonomous-git-tdd-program.md
git commit -m "docs(git): record workspace explorer delivery [GIT-06]"
```
