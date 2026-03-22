# Config Snippet Library Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 一次性补齐 `CFG-06` 与 `CFG-13`，让 OSM 的配置编辑器支持 provider snippet 的导入、导出、本地复用和审计记录，而不再只停留在单次表单修改。

**Architecture:** 在 Web 侧引入独立的 snippet schema 与本地存储工具。`ConfigRiskPanel` 负责生成、导入、应用和保存 snippet；`App` 只负责把 snippet 相关动作写入当前 snapshot 的 auditEvents。这样不改现有 Rust 写回链路，也不把 snippet 持久化强耦合到 Tauri runtime。

**Tech Stack:** React, TypeScript, Vitest

---

### Task 1: 写失败测试

**Files:**
- Create: `web/src/lib/config-snippets.test.ts`
- Modify: `web/src/components/config-risk-panel.test.tsx`
- Modify: `web/src/app.test.tsx`

**Step 1: Write the failing test**

Cover:
- snippet schema 可导出为稳定 JSON，并能校验导入 payload
- 配置编辑器可保存 snippet、再次应用 snippet，并把 provider/model/baseUrl 回填到 draft
- 从 JSON 导入 snippet 后可以直接应用
- snippet 保存或导入后会写入审计事件

**Step 2: Run test to verify it fails**

Run:
- `npm --prefix web run test -- config-snippets.test.ts config-risk-panel.test.tsx app.test.tsx`

**Step 3: Commit**

```bash
git add web/src/lib/config-snippets.test.ts web/src/components/config-risk-panel.test.tsx web/src/app.test.tsx
git commit -m "test(config): cover snippet import export and reuse [CFG-06][CFG-13]"
```

### Task 2: 实现 snippet schema、存储和 UI

**Files:**
- Create: `web/src/lib/config-snippets.ts`
- Modify: `web/src/lib/api.ts`
- Modify: `web/src/components/config-risk-panel.tsx`
- Modify: `web/src/lib/i18n.tsx`
- Modify: `web/src/styles.css`
- Modify: `web/src/app.tsx`

**Step 1: Write minimal implementation**

- 为 snippet 导出和导入建立稳定 schema
- 在编辑态新增 snippet 名称、导出 JSON、导入 JSON 和本地 snippet 列表
- 支持保存 snippet 到本地存储并重新应用到任意支持写回的配置 draft
- 将 save/import/export/apply 事件追加到当前 audit 历史

**Step 2: Run test to verify it passes**

Run:
- `npm --prefix web run test -- config-snippets.test.ts config-risk-panel.test.tsx app.test.tsx`

**Step 3: Commit**

```bash
git add web/src/lib/config-snippets.ts web/src/lib/api.ts web/src/components/config-risk-panel.tsx web/src/lib/i18n.tsx web/src/styles.css web/src/app.tsx
git commit -m "feat(config): add reusable snippet library [CFG-06][CFG-13]"
```

### Task 3: 文档与验证

**Files:**
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`

**Step 1: Run verification**

Run:
- `npm --prefix web run test`
- `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`

**Step 2: Update docs**

- 把 `CFG-06` 与 `CFG-13` 更新为真实状态
- 在支持矩阵和版本说明里明确 snippet library 的边界

**Step 3: Commit**

```bash
git add docs/specs/2026-03-22-autonomous-git-tdd-program.md docs/release/support-matrix.md docs/release/github-release-notes.md
git commit -m "docs(config): record snippet library support [CFG-06][CFG-13]"
```
