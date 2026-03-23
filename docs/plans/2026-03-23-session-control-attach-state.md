# Session Control Attach State Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 完成 `SES-04` 的 attach / detach，并把 `MON-03` 的 busy / waiting / idle 状态识别接到 snapshot 和 Web 控制面板。

**Architecture:** 不改外部助手协议，继续围绕本地 `session_control_state` 和 audit event 建立控制层。Rust 新增 attach / detach 动作与运行态推导；desktop/Tauri 暴露命令；Web 用同一套 `sessionControl` 字段展示状态并控制按钮。

**Tech Stack:** Rust + rusqlite + Tauri command + React + Vitest

---

### Task 1: Rust attach / detach 动作与状态推导

**Files:**
- Modify: `src-tauri/src/actions/session_control.rs`
- Modify: `src-tauri/src/actions/tests.rs`
- Modify: `src-tauri/src/storage/sqlite.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`

**Step 1: Write the failing test**

补测试断言：
- attach 会把已支持会话标记为 `attached = true`
- detach 会把已附着会话标记为 `attached = false`
- dashboard snapshot 会把会话运行态识别成 `busy / waiting / idle / detached`

**Step 2: Run test to verify it fails**

Run:
- `cargo test --manifest-path src-tauri/Cargo.toml actions::tests::attaches_and_detaches_supported_session --lib -- --exact`
- `cargo test --manifest-path src-tauri/Cargo.toml dashboard::tests::builds_session_control_runtime_states --lib -- --exact`

Expected: FAIL

**Step 3: Write minimal implementation**

实现：
- `attach_session` / `detach_session`
- continue 必须在 attached 状态下执行
- snapshot `SessionControlRecord` 暴露 `runtimeState`

**Step 4: Run test to verify it passes**

Run: same as Step 2

Expected: PASS

### Task 2: Desktop / Web 接 attach / detach 与运行态展示

**Files:**
- Modify: `src-tauri/src/commands/actions.rs`
- Modify: `src-tauri/src/desktop.rs`
- Modify: `web/src/lib/api.ts`
- Modify: `web/src/lib/api.test.ts`
- Modify: `web/src/components/session-detail.tsx`
- Modify: `web/src/components/session-detail.test.tsx`
- Modify: `web/src/components/active-session-cockpit.tsx`
- Modify: `web/src/components/active-session-cockpit.test.tsx`
- Modify: `web/src/routes/sessions.tsx`
- Modify: `web/src/app.tsx`
- Modify: `web/src/app.test.tsx`
- Modify: `web/src/lib/i18n.tsx`

**Step 1: Write the failing test**

补测试断言：
- SessionDetail 有 attach / detach 按钮
- detached 会话不能 continue
- cockpit 状态 badge 能显示 `busy / waiting / idle`

**Step 2: Run test to verify it fails**

Run: `npx vitest run src/components/session-detail.test.tsx src/components/active-session-cockpit.test.tsx src/app.test.tsx src/lib/api.test.ts`

Expected: FAIL

**Step 3: Write minimal implementation**

实现：
- 新增 `applySessionAttach / applySessionDetach`
- SessionDetail 接 attach / detach 回调
- cockpit / detail 状态文案接运行态

**Step 4: Run test to verify it passes**

Run: same as Step 2

Expected: PASS

### Task 3: 更新 spec 与 Git 证据

**Files:**
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`

**Step 1: Record review snapshots**

Run:
- `node scripts/git-review-snapshot.mjs --item SES-04 --phase green --command "cargo test --manifest-path src-tauri/Cargo.toml actions::tests::attaches_and_detaches_supported_session --lib -- --exact"`
- `node scripts/git-review-snapshot.mjs --item MON-03 --phase verify --command "npx vitest run src/components/session-detail.test.tsx src/components/active-session-cockpit.test.tsx src/app.test.tsx src/lib/api.test.ts"`

**Step 2: Update spec**

- `SES-04` 改成 `done`
- `MON-03` 改成 `done`

**Step 3: Record checkpoint**

Run:
- `node scripts/git-tdd-checkpoint.mjs --item SES-04 --phase verify --note "attach and detach landed across Rust, desktop, and web"`
- `node scripts/git-tdd-checkpoint.mjs --item MON-03 --phase verify --note "busy waiting idle runtime state landed across snapshot and cockpit"`

