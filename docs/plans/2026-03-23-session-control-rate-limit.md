# Session Control Rate Limit Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 完成 `SES-08`，为 continue prompt 增加节流与冲突保护，避免用户在 busy 会话或冷却窗口内重复发送提示。

**Architecture:** Rust 控制层在 `continue_session` 进入实际命令执行前统一做 guard 校验；Web 复用同一套 guard 逻辑，在 demo fallback 和详情按钮层都提前阻止无效 continue。继续沿用现有 `session_control_state`，不新增 schema。

**Tech Stack:** Rust + chrono + Tauri command + React + Vitest

---

### Task 1: Rust continue guard

**Files:**
- Modify: `src-tauri/src/actions/session_control.rs`
- Modify: `src-tauri/src/actions/tests.rs`

**Step 1: Write the failing test**

补测试断言：
- busy 会话不能继续发送 prompt
- 最近刚 continue 过的会话，在冷却窗口内不能再次 continue

**Step 2: Run test to verify it fails**

Run:
- `cargo test --manifest-path src-tauri/Cargo.toml actions::tests::refuses_continue_for_busy_session --lib -- --exact`
- `cargo test --manifest-path src-tauri/Cargo.toml actions::tests::throttles_continue_for_recent_prompt --lib -- --exact`

Expected: FAIL

**Step 3: Write minimal implementation**

实现：
- continue 前推导当前控制态是否 busy
- continue 前检查 `last_continued_at` 是否还在冷却窗口
- 返回明确 precondition message

**Step 4: Run test to verify it passes**

Run: same as Step 2

Expected: PASS

### Task 2: Web continue guard

**Files:**
- Modify: `web/src/lib/api.ts`
- Modify: `web/src/lib/api.test.ts`
- Modify: `web/src/components/session-detail.tsx`
- Modify: `web/src/components/session-detail.test.tsx`
- Modify: `web/src/lib/i18n.tsx`

**Step 1: Write the failing test**

补测试断言：
- demo fallback 在 busy 会话或冷却窗口内不会继续发送 prompt
- SessionDetail 会展示 guard hint，并禁用 continue 按钮

**Step 2: Run test to verify it fails**

Run:
- `npx vitest run src/lib/api.test.ts src/components/session-detail.test.tsx`

Expected: FAIL

**Step 3: Write minimal implementation**

实现：
- `getSessionContinueGuard` 统一返回 `busy / throttled / detached / unavailable / ok`
- `recordSessionContinue` 只在 `ok` 时继续
- 详情页根据 guard 展示禁用态和提示文案

**Step 4: Run test to verify it passes**

Run: same as Step 2

Expected: PASS

### Task 3: Spec 与 Git 证据

**Files:**
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`

**Step 1: Record review snapshots**

Run:
- `node scripts/git-review-snapshot.mjs --item SES-08 --phase green --command "cargo test --manifest-path src-tauri/Cargo.toml actions::tests::refuses_continue_for_busy_session --lib -- --exact"`
- `node scripts/git-review-snapshot.mjs --item SES-08 --phase verify --command "npx vitest run src/lib/api.test.ts src/components/session-detail.test.tsx"`

**Step 2: Update spec**

- `SES-08` 改成 `done`

**Step 3: Record checkpoint**

Run:
- `node scripts/git-tdd-checkpoint.mjs --item SES-08 --phase verify --note "continue throttling and busy-session guard landed across Rust and web"`
