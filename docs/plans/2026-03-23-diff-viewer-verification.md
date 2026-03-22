# Diff Viewer Verification Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 把 `GIT-01` 从“只在配置审查流里顺带存在”推进到可独立验收的前端 diff viewer 组件，补上独立测试和空 diff 提示，避免组件在无改动时静默留白。

**Architecture:** 沿用现有 `DiffViewer` 组件，不扩展 Rust 或路由层。先写组件级红测，覆盖字段标签、前后值、风险 badge，以及 `entries=[]` 时的空状态。随后补最小实现和 i18n 文案，最后更新总 spec 与发布文档。

**Tech Stack:** React, TypeScript, Vitest

---

### Task 1: 先写失败测试

**Files:**
- Create: `web/src/components/diff-viewer.test.tsx`

**Step 1: Write the failing tests**

Cover:
- `DiffViewer` 会展示字段标签、前后值和风险严重级别
- `entries=[]` 时展示明确空状态，而不是留空白块

**Step 2: Run tests to verify they fail**

Run:
- `npm --prefix web run test -- src/components/diff-viewer.test.tsx`

**Step 3: Commit**

```bash
git add web/src/components/diff-viewer.test.tsx
git commit -m "test(diff): cover standalone diff viewer states [GIT-01]"
```

### Task 2: 实现最小改动

**Files:**
- Modify: `web/src/components/diff-viewer.tsx`
- Modify: `web/src/lib/i18n.tsx`

**Step 1: Write minimal implementation**

- `DiffViewer` 在无条目时显示空状态
- 补中英文文案

**Step 2: Run tests to verify they pass**

Run:
- `npm --prefix web run test -- src/components/diff-viewer.test.tsx`

**Step 3: Commit**

```bash
git add web/src/components/diff-viewer.tsx web/src/lib/i18n.tsx
git commit -m "feat(diff): add standalone diff viewer empty state [GIT-01]"
```

### Task 3: 文档与 verify

**Files:**
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`

**Step 1: Run verification**

Run:
- `npm --prefix web run test`

**Step 2: Update docs**

- 把 `GIT-01` 标成完成
- 支持矩阵与发布说明同步 diff viewer 的独立验收状态

**Step 3: Commit**

```bash
git add docs/specs/2026-03-22-autonomous-git-tdd-program.md docs/release/support-matrix.md docs/release/github-release-notes.md
git commit -m "docs(diff): record standalone diff viewer coverage [GIT-01]"
```
