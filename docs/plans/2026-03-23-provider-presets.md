# Provider Presets Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 OSM 补齐 `CFG-05` 统一 provider presets，让支持写回的配置卡片能一键套用官方或常用 provider 模板，并可回退到原始配置值。

**Architecture:** 在 Web 侧建立共享 preset catalog，按 assistant 暴露可用模板，并把“套用 preset”当作对现有写回 draft 的结构化修改。风险、proxy/official 仍由现有 `recordConfigWriteback` 与 normalize 逻辑统一推导，不另建第二套规则。回退分两层：编辑态内可回退到检测到的原始值；保存后仍复用现有 backup manifest + rollback 测试链路。

**Tech Stack:** React, TypeScript, Vitest

---

### Task 1: 写失败测试

**Files:**
- Modify: `web/src/components/config-risk-panel.test.tsx`
- Modify: `web/src/lib/api.test.ts`

**Step 1: Write the failing test**

Cover:
- 编辑支持写回的配置卡片时展示 preset 列表
- 点击 preset 会批量填充 provider / model / baseUrl
- 点击回退按钮会恢复到原始检测值
- 套用 preset 后保存，风险状态仍由现有规则推导

**Step 2: Run test to verify it fails**

Run:
- `npm --prefix web run test -- config-risk-panel.test.tsx api.test.ts`

**Step 3: Commit**

```bash
git add web/src/components/config-risk-panel.test.tsx web/src/lib/api.test.ts
git commit -m "test(config): cover provider preset apply and revert [CFG-05]"
```

### Task 2: 实现 preset catalog 与 UI 接线

**Files:**
- Create: `web/src/lib/provider-presets.ts`
- Modify: `web/src/components/config-risk-panel.tsx`
- Modify: `web/src/lib/i18n.tsx`
- Modify: `web/src/styles.css`

**Step 1: Write minimal implementation**

- 为 `GitHub Copilot CLI / Factory Droid / Gemini CLI / OpenClaw` 建立最小 preset catalog
- 编辑态显示 preset chips 和说明
- 支持“恢复检测值”回到原始配置

**Step 2: Run test to verify it passes**

Run:
- `npm --prefix web run test -- config-risk-panel.test.tsx api.test.ts`

**Step 3: Commit**

```bash
git add web/src/lib/provider-presets.ts web/src/components/config-risk-panel.tsx web/src/lib/i18n.tsx web/src/styles.css
git commit -m "feat(config): add unified provider presets [CFG-05]"
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

- 把 `CFG-05` 状态改为真实完成度
- 在支持矩阵与发布说明中明确 presets 的覆盖范围与边界

**Step 3: Commit**

```bash
git add docs/specs/2026-03-22-autonomous-git-tdd-program.md docs/release/support-matrix.md docs/release/github-release-notes.md
git commit -m "docs(config): record provider presets tranche [CFG-05]"
```
