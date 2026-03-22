# Richer Session Filters Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 补齐 `UX-06`，让 Sessions 页面除了全文搜索之外，还能按助手、项目、风险、导出准备度和控制状态做组合筛选，帮助用户更快判断哪些会话该保留、导出或清理。

**Architecture:** 只改 Web 侧。先把筛选逻辑抽到独立的 `session-filters` 纯函数模块，保证组合规则可单测；再在 `SessionsRoute` 里接入受控筛选表单和新的结果摘要。筛选发生在现有 search-as-you-type 结果之后，不改变 Rust snapshot 或索引层。

**Tech Stack:** React, TypeScript, Vitest

---

### Task 1: 先写失败测试

**Files:**
- Create: `web/src/lib/session-filters.test.ts`
- Modify: `web/src/routes/sessions.test.tsx`

**Step 1: Write the failing test**

Cover:
- 纯筛选逻辑支持 assistant / project / risk / export / control 五类组合过滤
- Sessions 页面切换筛选项后只显示匹配会话
- 切换到 “ready to quarantine” 或 “controllable only” 之类的筛选时，摘要和空状态会同步更新

**Step 2: Run test to verify it fails**

Run:
- `npm --prefix web run test -- session-filters.test.ts sessions.test.tsx`

**Step 3: Commit**

```bash
git add web/src/lib/session-filters.test.ts web/src/routes/sessions.test.tsx
git commit -m "test(filters): cover richer session filters [UX-06]"
```

### Task 2: 实现筛选栏和组合过滤

**Files:**
- Create: `web/src/lib/session-filters.ts`
- Modify: `web/src/routes/sessions.tsx`
- Modify: `web/src/lib/i18n.tsx`
- Modify: `web/src/styles.css`

**Step 1: Write minimal implementation**

- 新增纯函数 `applySessionFilters`
- `SessionsRoute` 增加筛选状态和筛选器 UI
- 搜索结果先走 search-as-you-type，再叠加组合筛选
- 结果摘要显示命中数与筛选状态

**Step 2: Run test to verify it passes**

Run:
- `npm --prefix web run test -- session-filters.test.ts sessions.test.tsx`

**Step 3: Commit**

```bash
git add web/src/lib/session-filters.ts web/src/routes/sessions.tsx web/src/lib/i18n.tsx web/src/styles.css
git commit -m "feat(filters): add richer session filters [UX-06]"
```

### Task 3: 文档和全量回归

**Files:**
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`
- Modify: `web/src/app.test.tsx`

**Step 1: Add integration coverage**

- 在 `App` 里补至少一条筛选交互测试，确保整页接线正常

**Step 2: Run verification**

Run:
- `npm --prefix web run test`

**Step 3: Update docs**

- 把 `UX-06` 更新为真实状态
- 在支持矩阵和发布说明里明确列出新的高级筛选能力

**Step 4: Commit**

```bash
git add web/src/app.test.tsx docs/specs/2026-03-22-autonomous-git-tdd-program.md docs/release/support-matrix.md docs/release/github-release-notes.md
git commit -m "docs(filters): record richer session filters [UX-06]"
```
