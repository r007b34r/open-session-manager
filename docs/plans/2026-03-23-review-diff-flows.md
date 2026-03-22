# Review Diff Flows Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 补齐 `UX-04`，让配置写回和会话清理不再是直接执行动作，而是先进入审查步骤，展示可读 diff、风险提示和明确确认动作。

**Architecture:** Web 侧新增一个轻量 review-flow 聚合模块，把“原始配置 vs 当前草稿”转成字段级 diff 与风险提示；`ConfigRiskPanel` 在真正写回前插入 review gate。`SessionDetail` 在 quarantine 前插入 cleanup review gate，要求显式确认已完成导出并理解风险。

**Tech Stack:** React, TypeScript, Vitest

---

### Task 1: 先写失败测试

**Files:**
- Create: `web/src/lib/review-flow.test.ts`
- Modify: `web/src/components/config-risk-panel.test.tsx`
- Modify: `web/src/components/session-detail.test.tsx`

**Step 1: Write the failing tests**

Cover:
- 配置草稿能生成字段级 diff 和风险提示
- 配置写回需要先进入 review，未确认前不能提交
- session quarantine 需要显式确认后才会触发删除动作

**Step 2: Run tests to verify they fail**

Run:
- `npm --prefix web run test -- src/lib/review-flow.test.ts src/components/config-risk-panel.test.tsx src/components/session-detail.test.tsx`

**Step 3: Commit**

```bash
git add web/src/lib/review-flow.test.ts web/src/components/config-risk-panel.test.tsx web/src/components/session-detail.test.tsx
git commit -m "test(review): cover diff and confirmation flows [UX-04]"
```

### Task 2: 实现 review flow

**Files:**
- Create: `web/src/lib/review-flow.ts`
- Create: `web/src/components/diff-viewer.tsx`
- Modify: `web/src/components/config-risk-panel.tsx`
- Modify: `web/src/components/session-detail.tsx`
- Modify: `web/src/lib/i18n.tsx`
- Modify: `web/src/styles.css`

**Step 1: Write minimal implementation**

- 提供配置 diff / warning 聚合函数
- 新增通用 diff viewer
- 配置写回前显示 review panel
- quarantine 前显示 cleanup review gate
- 补中英文文案与样式

**Step 2: Run tests to verify they pass**

Run:
- `npm --prefix web run test -- src/lib/review-flow.test.ts src/components/config-risk-panel.test.tsx src/components/session-detail.test.tsx`

**Step 3: Commit**

```bash
git add web/src/lib/review-flow.ts web/src/components/diff-viewer.tsx web/src/components/config-risk-panel.tsx web/src/components/session-detail.tsx web/src/lib/i18n.tsx web/src/styles.css
git commit -m "feat(review): add diff and confirmation flows [UX-04]"
```

### Task 3: 集成回归与文档

**Files:**
- Modify: `web/src/app.test.tsx`
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`

**Step 1: Add integration coverage**

- App 层覆盖配置写回 review gate 和 quarantine confirm gate

**Step 2: Run verification**

Run:
- `npm --prefix web run test`

**Step 3: Update docs**

- 把 `UX-04` 标成完成
- 支持矩阵与发布说明补上 review/diff 流

**Step 4: Commit**

```bash
git add web/src/app.test.tsx docs/specs/2026-03-22-autonomous-git-tdd-program.md docs/release/support-matrix.md docs/release/github-release-notes.md
git commit -m "docs(review): record diff and confirmation flows [UX-04]"
```
