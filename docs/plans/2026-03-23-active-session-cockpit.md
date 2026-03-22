# Active Session Cockpit Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 补齐 `MON-01`，在总览页提供真正可用的 active session cockpit，而不是只把恢复按钮埋在详情页里。用户需要在首页看到哪些会话当前可控、是否已附着、最近一次恢复或继续执行的结果，以及是否能主动刷新这一块状态。

**Architecture:** 新增独立的 `ActiveSessionCockpit` 组件，只消费 `DashboardSnapshot.sessions` 里的 `sessionControl` 信息。`App` 负责统一封装 `fetchDashboardSnapshot` 的初始加载与手动刷新，给 `OverviewRoute` 暴露 `onRefreshSnapshot` 与 `isRefreshing`。这样不碰 Rust 的控制链路，只把已有状态采集真正放到总览里可见。

**Tech Stack:** React, TypeScript, Vitest, Playwright

---

### Task 1: 先写失败测试

**Files:**
- Create: `web/src/components/active-session-cockpit.test.tsx`
- Modify: `web/src/app.test.tsx`

**Step 1: Write the failing test**

Cover:
- 总览页 cockpit 只展示有控制状态的会话，并明确显示 attached / ready / unavailable 等状态
- 点击刷新按钮后，`App` 会重新拉取 snapshot，并把 cockpit 中最近控制结果更新到最新值

**Step 2: Run test to verify it fails**

Run:
- `npm --prefix web run test -- active-session-cockpit.test.tsx app.test.tsx`

**Step 3: Commit**

```bash
git add web/src/components/active-session-cockpit.test.tsx web/src/app.test.tsx
git commit -m "test(monitor): cover active session cockpit refresh [MON-01]"
```

### Task 2: 实现 cockpit 与刷新链路

**Files:**
- Create: `web/src/components/active-session-cockpit.tsx`
- Modify: `web/src/routes/index.tsx`
- Modify: `web/src/app.tsx`
- Modify: `web/src/lib/i18n.tsx`
- Modify: `web/src/styles.css`

**Step 1: Write minimal implementation**

- 总览页新增 active session cockpit
- 只展示已有 `sessionControl` 的会话，避免把不可控助手伪装成活跃控制对象
- 支持手动 refresh，并在刷新期间禁用按钮与显示进行中状态

**Step 2: Run test to verify it passes**

Run:
- `npm --prefix web run test -- active-session-cockpit.test.tsx app.test.tsx`

**Step 3: Commit**

```bash
git add web/src/components/active-session-cockpit.tsx web/src/routes/index.tsx web/src/app.tsx web/src/lib/i18n.tsx web/src/styles.css
git commit -m "feat(monitor): add active session cockpit [MON-01]"
```

### Task 3: 验证与文档

**Files:**
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`
- Modify: `tests/e2e/open-session-manager.spec.ts`

**Step 1: Run verification**

Run:
- `npm --prefix web run test`
- `npm --prefix web run e2e -- --grep "active session cockpit"`

**Step 2: Update docs**

- 把 `MON-01` 更新为真实状态
- 在支持矩阵和发布说明里明确 active session cockpit 已落地

**Step 3: Commit**

```bash
git add docs/specs/2026-03-22-autonomous-git-tdd-program.md docs/release/support-matrix.md docs/release/github-release-notes.md tests/e2e/open-session-manager.spec.ts
git commit -m "docs(monitor): record active session cockpit [MON-01]"
```
