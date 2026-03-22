# Search-As-You-Type Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 补齐 `SRCH-06`，让 Sessions 页的本地搜索具备真正可验证的 search-as-you-type 行为，而不是每次按键都同步重算。用户在快速输入时应该看到一个短暂的“正在更新”状态；旧查询必须被取消，只应用最后一次输入。

**Architecture:** 只改 Web 侧 `SessionsRoute`。引入 `rawQuery / committedQuery / isSearching` 三段状态，利用定时器做显式 debounce。搜索结果继续复用现有 `searchSessions`；测试从 UI 侧验证防抖、取消和结果刷新，不碰 Rust 索引层。

**Tech Stack:** React, TypeScript, Vitest

---

### Task 1: 先写失败测试

**Files:**
- Create: `web/src/routes/sessions.test.tsx`

**Step 1: Write the failing test**

Cover:
- 输入关键词后，在 debounce 窗口内仍保留旧结果并显示“正在更新”
- 在 debounce 窗口内改成新的关键词时，旧搜索会被取消，只应用最后一次输入

**Step 2: Run test to verify it fails**

Run:
- `npm --prefix web run test -- sessions.test.tsx`

**Step 3: Commit**

```bash
git add web/src/routes/sessions.test.tsx
git commit -m "test(search): cover debounced search-as-you-type [SRCH-06]"
```

### Task 2: 实现防抖搜索

**Files:**
- Modify: `web/src/routes/sessions.tsx`
- Modify: `web/src/lib/i18n.tsx`

**Step 1: Write minimal implementation**

- 引入 `rawQuery / committedQuery / isSearching`
- 在 debounce 窗口内保留旧结果
- 新输入能取消旧定时器
- summary 区显示 pending 文案

**Step 2: Run test to verify it passes**

Run:
- `npm --prefix web run test -- sessions.test.tsx`

**Step 3: Commit**

```bash
git add web/src/routes/sessions.tsx web/src/lib/i18n.tsx
git commit -m "feat(search): add debounced search-as-you-type [SRCH-06]"
```

### Task 3: 文档与验证

**Files:**
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`

**Step 1: Run verification**

Run:
- `npm --prefix web run test`

**Step 2: Update docs**

- 把 `SRCH-06` 更新为真实状态
- 在发布说明里明确搜索已具备防抖与取消行为

**Step 3: Commit**

```bash
git add docs/specs/2026-03-22-autonomous-git-tdd-program.md docs/release/support-matrix.md docs/release/github-release-notes.md
git commit -m "docs(search): record debounced search-as-you-type [SRCH-06]"
```
