# Config Backup Manifest Visibility Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 补齐 `CFG-11` 的可见性缺口。当前配置写回前已经会自动生成备份 manifest，但路径只留在 Rust 内部，用户无法从审计界面确认备份落点。本轮要把 `manifest_path` 进入审计链路，并在 Web 审计页展示出来。

**Architecture:** 保持现有备份目录结构不变。Rust 写回动作把 `manifest_path` 注入 `after_state`，dashboard snapshot 继续复用现有 `parse_audit_paths` 提取。浏览器 fallback 的 `recordConfigWriteback` 补同样字段。Web 审计页新增路径展示区，优先显示 `manifest / output / quarantine` 三类受管路径。

**Tech Stack:** Rust, React, TypeScript, Vitest

---

### Task 1: 先写失败测试

**Files:**
- Modify: `src-tauri/src/actions/tests.rs`
- Modify: `web/src/app.test.tsx`

**Step 1: Write the failing tests**

Cover:
- Rust `config_writeback` 审计事件的 `after_state` 包含 `manifest_path`
- Web 审计页在配置写回后能看到备份 manifest 路径

**Step 2: Run tests to verify they fail**

Run:
- `cargo test writes_back_config_audit_event_with_backup_manifest_path --manifest-path src-tauri/Cargo.toml`
- `npm --prefix web run test -- src/app.test.tsx`

**Step 3: Commit**

```bash
git add src-tauri/src/actions/tests.rs web/src/app.test.tsx
git commit -m "test(config): cover backup manifest visibility [CFG-11]"
```

### Task 2: 实现 manifest 路径透传与展示

**Files:**
- Modify: `src-tauri/src/actions/config_writeback.rs`
- Modify: `web/src/lib/api.ts`
- Modify: `web/src/routes/audit.tsx`
- Modify: `web/src/lib/i18n.tsx`

**Step 1: Write minimal implementation**

- `config_writeback` / `config_rollback` 审计事件携带 `manifest_path`
- 浏览器 fallback 的写回事件也补 `manifestPath`
- 审计页展示受管路径列表

**Step 2: Run tests to verify they pass**

Run:
- `cargo test writes_back_config_audit_event_with_backup_manifest_path --manifest-path src-tauri/Cargo.toml`
- `npm --prefix web run test -- src/app.test.tsx`

**Step 3: Commit**

```bash
git add src-tauri/src/actions/config_writeback.rs web/src/lib/api.ts web/src/routes/audit.tsx web/src/lib/i18n.tsx
git commit -m "feat(config): expose backup manifest path in audit trail [CFG-11]"
```

### Task 3: 文档与 verify

**Files:**
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`

**Step 1: Run verification**

Run:
- `cargo test writes_back_config_audit_event_with_backup_manifest_path --manifest-path src-tauri/Cargo.toml`
- `npm --prefix web run test`

**Step 2: Update docs**

- 把 `CFG-11` 标成完成
- 支持矩阵与发布说明明确“自动备份 manifest 路径可在审计页查看”

**Step 3: Commit**

```bash
git add docs/specs/2026-03-22-autonomous-git-tdd-program.md docs/release/support-matrix.md docs/release/github-release-notes.md
git commit -m "docs(config): record backup manifest visibility [CFG-11]"
```
