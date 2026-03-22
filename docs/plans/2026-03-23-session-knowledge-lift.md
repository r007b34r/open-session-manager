# Session Knowledge Lift Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 补齐 `MCP-08`，把会话中已经沉淀出来的摘要、todo、标签和 transcript highlight 转成可直接复用的结构化 rule / skill Markdown，而不是只停留在“导出过手”。

**Architecture:** 不改 Rust 导出链，先在 Web 侧增加一个纯函数知识提炼模块，输入 `SessionDetailRecord`，输出稳定的 rule / skill artifact。`SessionDetail` 新增一个知识提炼卡片，让用户在删前可以直接审查并复制这些结构化内容。

**Tech Stack:** React, TypeScript, Vitest

---

### Task 1: 先写失败测试

**Files:**
- Create: `web/src/lib/knowledge-lift.test.ts`
- Modify: `web/src/components/session-detail.test.tsx`

**Step 1: Write the failing tests**

Cover:
- 会话可生成 rule artifact，包含 source session、summary、open todos 和风险标记
- 会话可生成 skill artifact，包含 trigger、steps 和 resume cue
- SessionDetail 可切换预览 rule / skill

**Step 2: Run tests to verify they fail**

Run:
- `npm --prefix web run test -- src/lib/knowledge-lift.test.ts src/components/session-detail.test.tsx`

**Step 3: Commit**

```bash
git add web/src/lib/knowledge-lift.test.ts web/src/components/session-detail.test.tsx
git commit -m "test(knowledge): cover session rule and skill lift [MCP-08]"
```

### Task 2: 实现知识提炼

**Files:**
- Create: `web/src/lib/knowledge-lift.ts`
- Modify: `web/src/components/session-detail.tsx`
- Modify: `web/src/lib/i18n.tsx`
- Modify: `web/src/styles.css`

**Step 1: Write minimal implementation**

- 生成稳定 Markdown 的 rule / skill artifact
- `SessionDetail` 增加预览切换
- 补中英文文案与只读展示样式

**Step 2: Run tests to verify they pass**

Run:
- `npm --prefix web run test -- src/lib/knowledge-lift.test.ts src/components/session-detail.test.tsx`

**Step 3: Commit**

```bash
git add web/src/lib/knowledge-lift.ts web/src/components/session-detail.tsx web/src/lib/i18n.tsx web/src/styles.css
git commit -m "feat(knowledge): add session rule and skill lift [MCP-08]"
```

### Task 3: 集成回归与文档

**Files:**
- Modify: `web/src/app.test.tsx`
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`

**Step 1: Add integration coverage**

- App 层确认会话详情里能看到知识提炼卡片

**Step 2: Run verification**

Run:
- `npm --prefix web run test`

**Step 3: Update docs**

- 把 `MCP-08` 标成完成
- 支持矩阵和发布说明补上 rule / skill 提炼能力

**Step 4: Commit**

```bash
git add web/src/app.test.tsx docs/specs/2026-03-22-autonomous-git-tdd-program.md docs/release/support-matrix.md docs/release/github-release-notes.md
git commit -m "docs(knowledge): record session rule and skill lift [MCP-08]"
```
