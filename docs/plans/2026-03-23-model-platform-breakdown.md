# Model And Platform Breakdown Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 补齐 `ANA-03`，在总览 Usage 面板里新增模型和 provider/platform 聚合，让用户能看出 token/cost 集中在哪些模型，以及本地配置主要落在哪些 provider。

**Architecture:** 不改 Rust snapshot。直接复用现有 `sessions[].usage.model` 和 `configs[].provider`，在 Web 侧增加一个纯聚合模块，再把 breakdown 卡片挂到 `UsagePanel`。模型 breakdown 保留 token/cost 语义；provider breakdown 展示配置数量、覆盖助手数和代理占比。

**Tech Stack:** React, TypeScript, Vitest

---

### Task 1: 先写失败测试

**Files:**
- Create: `web/src/lib/usage-breakdown.test.ts`
- Modify: `web/src/components/usage-panel.test.tsx`

**Step 1: Write the failing test**

Cover:
- 模型 breakdown 会按 `usage.model` 聚合 sessionCount / totalTokens / cost
- provider breakdown 会按 `configs[].provider` 聚合 configCount / assistantCount / proxyCount
- UsagePanel 会把两个 breakdown 渲染出来

**Step 2: Run test to verify it fails**

Run:
- `npm --prefix web run test -- usage-breakdown.test.ts usage-panel.test.tsx`

**Step 3: Commit**

```bash
git add web/src/lib/usage-breakdown.test.ts web/src/components/usage-panel.test.tsx
git commit -m "test(analytics): cover model and platform breakdown [ANA-03]"
```

### Task 2: 实现 breakdown 聚合与展示

**Files:**
- Create: `web/src/lib/usage-breakdown.ts`
- Modify: `web/src/components/usage-panel.tsx`
- Modify: `web/src/routes/index.tsx`
- Modify: `web/src/lib/i18n.tsx`
- Modify: `web/src/styles.css`

**Step 1: Write minimal implementation**

- 增加模型 / provider breakdown 纯聚合函数
- `UsagePanel` 新增两张 breakdown 卡片
- `OverviewRoute` 传入 `sessions` 与 `configs`
- 补充中英文文案和卡片样式

**Step 2: Run test to verify it passes**

Run:
- `npm --prefix web run test -- usage-breakdown.test.ts usage-panel.test.tsx`

**Step 3: Commit**

```bash
git add web/src/lib/usage-breakdown.ts web/src/components/usage-panel.tsx web/src/routes/index.tsx web/src/lib/i18n.tsx web/src/styles.css
git commit -m "feat(analytics): add model and platform breakdown [ANA-03]"
```

### Task 3: 文档与全量回归

**Files:**
- Modify: `web/src/app.test.tsx`
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`

**Step 1: Add integration coverage**

- `App` 总览测试确认 breakdown 在整页中可见

**Step 2: Run verification**

Run:
- `npm --prefix web run test`

**Step 3: Update docs**

- 把 `ANA-03` 标成已完成
- 在支持矩阵和发布说明里加入 model / provider breakdown

**Step 4: Commit**

```bash
git add web/src/app.test.tsx docs/specs/2026-03-22-autonomous-git-tdd-program.md docs/release/support-matrix.md docs/release/github-release-notes.md
git commit -m "docs(analytics): record model and platform breakdown [ANA-03]"
```
