# Search Transcript Focus Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 `SRCH-05` 补齐搜索命中片段的 transcript 内定位和高亮，让 Sessions 列表里的搜索结果可以把右侧详情直接对准真实命中的 transcript 条目。

**Architecture:** 扩展 Web 搜索层，为每个命中字段保留结构化 match target，而不是只保留一段字符串 snippet。`SessionsRoute` 根据当前搜索结果把命中的 transcript 索引传给 `SessionDetail`，由详情面板负责高亮匹配项并标记“来自搜索”。只在 Web 侧增加这条定位链路，不触碰现有 Rust snapshot、索引缓存和控制命令。

**Tech Stack:** React, TypeScript, Vitest, Playwright

---

### Task 1: 写失败测试

**Files:**
- Modify: `web/src/lib/session-search.test.ts`
- Modify: `web/src/components/session-detail.test.tsx`
- Modify: `web/src/app.test.tsx`
- Modify: `tests/e2e/open-session-manager.spec.ts`

**Step 1: Write the failing test**

Cover:
- 搜索命中 transcript 时返回结构化 transcript target
- 详情页收到目标后会高亮对应 transcript 条目
- 搜索中的会话被选中后，右侧详情只高亮实际命中的 transcript，而不是整段都无差别渲染
- E2E 验证搜索命中后能在详情区看到“来自搜索”的 transcript 高亮

**Step 2: Run test to verify it fails**

Run:
- `npm --prefix web run test -- session-search.test.ts session-detail.test.tsx app.test.tsx`
- `npm --prefix web run e2e -- --grep "highlights the matched transcript"`

**Step 3: Commit**

```bash
git add web/src/lib/session-search.test.ts web/src/components/session-detail.test.tsx web/src/app.test.tsx tests/e2e/open-session-manager.spec.ts
git commit -m "test(search): cover transcript focus from search results [SRCH-05]"
```

### Task 2: 实现 transcript focus 链路

**Files:**
- Modify: `web/src/lib/session-search.ts`
- Modify: `web/src/routes/sessions.tsx`
- Modify: `web/src/components/session-detail.tsx`
- Modify: `web/src/styles.css`

**Step 1: Write minimal implementation**

- 为搜索结果新增可选 transcript match target
- 让 `SessionsRoute` 把当前选中会话的 transcript match target 传到详情页
- 让 `SessionDetail` 对对应 transcript entry 增加高亮和“搜索命中”标记
- 保持无搜索或非 transcript 命中时的现有行为不变

**Step 2: Run test to verify it passes**

Run:
- `npm --prefix web run test -- session-search.test.ts session-detail.test.tsx app.test.tsx`
- `npm --prefix web run e2e -- --grep "highlights the matched transcript"`

**Step 3: Commit**

```bash
git add web/src/lib/session-search.ts web/src/routes/sessions.tsx web/src/components/session-detail.tsx web/src/styles.css
git commit -m "feat(search): focus matching transcript highlights [SRCH-05]"
```

### Task 3: 文档与验证

**Files:**
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`

**Step 1: Run verification**

Run:
- `npm --prefix web run test`
- `npm --prefix web run e2e`
- `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`

**Step 2: Update docs**

- 把 `SRCH-05` 状态改成真实完成度
- 在支持矩阵和发布说明里明确“搜索片段可定位到 transcript 命中条目”

**Step 3: Commit**

```bash
git add docs/specs/2026-03-22-autonomous-git-tdd-program.md docs/release/support-matrix.md docs/release/github-release-notes.md
git commit -m "docs(search): record transcript focus support [SRCH-05]"
```
