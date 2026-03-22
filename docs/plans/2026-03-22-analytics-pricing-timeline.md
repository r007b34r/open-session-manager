# Analytics Pricing Timeline Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 OSM 补齐 `ANA-01` pricing lookup 与 `ANA-02` usage timeline，让成本展示具备可验证来源，并把用量趋势落到快照与 Web 面板。

**Architecture:** 后端先在 usage 聚合层引入本地价格目录，对已有上游明确上报成本的会话保留 `reported`，对缺失成本但能从本地价格表推导的会话标记为 `estimated`，其余保持 `unknown`。随后基于 `lastActivityAt` 构建日级 usage timeline，把 totals、assistant breakdown 和 timeline 一起暴露到 dashboard snapshot，再在 Web overview 面板展示来源说明和趋势图。

**Tech Stack:** Rust, serde, chrono, Tauri snapshot command, React, Vitest

---

### Task 1: ANA-01 后端价格目录与成本来源

**Files:**
- Modify: `src-tauri/src/usage.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Write the failing tests**

Cover:
- 当上游会话没有显式 `cost` 但模型命中本地价格表时，能估算 `costUsd`
- 已上报成本的会话保持 `reported`，不会被价格表覆盖
- 未命中价格目录的会话保持 `unknown`

**Step 2: Run test to verify it fails**

Run:
- `cargo test usage::tests::estimates_cost_from_local_price_catalog_when_upstream_cost_is_missing -- --exact`
- `cargo test usage::tests::preserves_reported_cost_source_when_session_already_has_cost -- --exact`
- `cargo test usage::tests::keeps_unknown_cost_source_when_no_price_catalog_entry_exists -- --exact`

**Step 3: Write minimal implementation**

- 为 `SessionUsageRecord` / aggregate record 增加成本来源元数据
- 为已支持模型建立最小本地价格目录
- 仅在能明确解析模型且 token 字段齐全时估算成本

**Step 4: Run test to verify it passes**

Run:
- `cargo test usage::tests::estimates_cost_from_local_price_catalog_when_upstream_cost_is_missing -- --exact`
- `cargo test usage::tests::preserves_reported_cost_source_when_session_already_has_cost -- --exact`
- `cargo test usage::tests::keeps_unknown_cost_source_when_no_price_catalog_entry_exists -- --exact`

**Step 5: Commit**

```bash
git add src-tauri/src/usage.rs src-tauri/src/lib.rs
git commit -m "feat(analytics): add local pricing lookup and cost provenance [ANA-01]"
```

### Task 2: ANA-02 后端 usage timeline 快照

**Files:**
- Modify: `src-tauri/src/usage.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`
- Modify: `src-tauri/tests/cli_snapshot.rs`

**Step 1: Write the failing tests**

Cover:
- snapshot 输出日级 usage timeline
- 同一天多会话会正确聚合 tokens、sessions 和 cost
- 当某天存在 unknown cost 时，该天 aggregate cost 保持 unknown，而不是伪造 0

**Step 2: Run test to verify it fails**

Run:
- `cargo test usage::tests::builds_daily_usage_timeline_with_cost_provenance -- --exact`
- `cargo test --test cli_snapshot snapshot_command_emits_usage_timeline_and_cost_provenance -- --exact`

**Step 3: Write minimal implementation**

- 新增 usage timeline record 与构建函数
- dashboard snapshot 暴露 `usageTimeline`
- fixture snapshot 和 CLI 输出接入新字段

**Step 4: Run test to verify it passes**

Run:
- `cargo test usage::tests::builds_daily_usage_timeline_with_cost_provenance -- --exact`
- `cargo test --test cli_snapshot snapshot_command_emits_usage_timeline_and_cost_provenance -- --exact`

**Step 5: Commit**

```bash
git add src-tauri/src/usage.rs src-tauri/src/commands/dashboard.rs src-tauri/tests/cli_snapshot.rs
git commit -m "feat(analytics): add usage timeline snapshot payload [ANA-02]"
```

### Task 3: Web usage 面板展示来源与趋势

**Files:**
- Modify: `web/src/lib/api.ts`
- Modify: `web/src/components/usage-panel.tsx`
- Modify: `web/src/components/usage-panel.test.tsx`
- Modify: `web/src/lib/i18n.tsx`

**Step 1: Write the failing tests**

Cover:
- usage 面板展示 reported / estimated / unknown 成本来源
- timeline 在 overview 中显示日级趋势，不会把 unknown 显示成 `$0.00`
- API normalization 能接受新 snapshot 字段并在缺失时降级为空 timeline

**Step 2: Run test to verify it fails**

Run:
- `npm --prefix web run test -- usage-panel.test.tsx`
- `npm --prefix web run test -- api.test.ts`

**Step 3: Write minimal implementation**

- 扩展前端 snapshot 类型和 normalize 逻辑
- usage 面板新增 cost provenance 与 timeline 视图
- 中英文文案补齐

**Step 4: Run test to verify it passes**

Run:
- `npm --prefix web run test -- usage-panel.test.tsx`
- `npm --prefix web run test -- api.test.ts`

**Step 5: Commit**

```bash
git add web/src/lib/api.ts web/src/components/usage-panel.tsx web/src/components/usage-panel.test.tsx web/src/lib/i18n.tsx
git commit -m "feat(web): show cost provenance and usage timeline [ANA-01][ANA-02]"
```

### Task 4: 验证与 spec 同步

**Files:**
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`

**Step 1: Run verification**

Run:
- `cargo test usage::tests -- --test-threads=1`
- `cargo test --test cli_snapshot`
- `npm --prefix web run test -- usage-panel.test.tsx`
- `npm --prefix web run test -- api.test.ts`
- `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`

**Step 2: Update docs**

- 把 `ANA-01` / `ANA-02` 状态改成真实完成度
- 在支持矩阵和发布说明里明确“本地价格目录估算”和“usage timeline”的边界

**Step 3: Commit**

```bash
git add docs/specs/2026-03-22-autonomous-git-tdd-program.md docs/release/support-matrix.md docs/release/github-release-notes.md
git commit -m "docs: record analytics pricing and timeline tranche [ANA-01][ANA-02]"
```
