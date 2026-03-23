# Git Commit History View Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 完成 `GIT-04`，让 Git 面板支持 commit history 筛选和详情展开，而不是只显示一串最近提交摘要。

**Architecture:** 复用现有 `GitProjectRecord.recentCommits`，只在前端 `GitProjectPanel` 增加筛选输入、本地过滤和展开详情。暂不扩展后端 schema 或追加 git 命令采样。

**Tech Stack:** React + Vitest

---

### Task 1: Git history view red

**Files:**
- Create: `web/src/components/git-project-panel.test.tsx`
- Modify: `web/src/lib/i18n.tsx`

**Step 1: Write the failing test**

补测试断言：
- 可以按 commit summary / author / sha 过滤历史
- 可以展开单条 commit 明细，看到 sha / author / authoredAt

**Step 2: Run test to verify it fails**

Run:
- `npx vitest run src/components/git-project-panel.test.tsx`

Expected: FAIL

### Task 2: Minimal implementation

**Files:**
- Modify: `web/src/components/git-project-panel.tsx`
- Modify: `web/src/lib/i18n.tsx`

**Step 1: Write minimal implementation**

实现：
- history filter input
- filtered commit list
- commit detail toggle

**Step 2: Run test to verify it passes**

Run:
- `npx vitest run src/components/git-project-panel.test.tsx`

Expected: PASS

### Task 3: Spec 与 Git 证据

**Files:**
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`

**Step 1: Record review snapshots**

Run:
- `node scripts/git-review-snapshot.mjs --item GIT-04 --phase verify --command "npx vitest run src/components/git-project-panel.test.tsx"`

**Step 2: Update spec**

- `GIT-04` 改成 `done`

**Step 3: Record checkpoint**

Run:
- `node scripts/git-tdd-checkpoint.mjs --item GIT-04 --phase verify --note "git history view filter and commit detail landed in overview panel"`
