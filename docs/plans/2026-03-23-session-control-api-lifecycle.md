# Session Control API Lifecycle Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 OSM 的会话控制补齐 pause/resume 生命周期、进程状态元数据、HTTP 控制 API，以及最小可见的 live HUD。

**Architecture:** 继续沿用现有 `session_control_state / session_control_events / audit_events` 持久化链路，在 SQLite 状态表中补充 pause 与进程元数据字段；Rust 动作层统一读写状态并返回快照可消费的记录；HTTP `serve` 壳层通过新的 `POST` 控制端点复用动作层；Web 详情页和 cockpit 只消费扩展后的控制记录，不自行推导进程细节。

**Tech Stack:** Rust, rusqlite, axum, serde_json, React, Vitest

**Status:** 已完成并验证。`SES-05 / SES-06 / MON-02 / API-02` 已转为可运行状态，验证见文末。

---

### Task 1: 扩展 session control 状态模型

**Files:**
- Modify: `src-tauri/src/storage/schema.sql`
- Modify: `src-tauri/src/storage/sqlite.rs`
- Test: `src-tauri/src/storage/tests.rs`

**Step 1: Write the failing test**

在 `src-tauri/src/storage/tests.rs` 增加一个测试，断言 `session_control_state` 支持读写 `paused / paused_at / process_state / process_id / exit_code / started_at / runtime_seconds / event_count / input_tokens / output_tokens / total_tokens / last_activity_at` 字段。

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml storage::tests::persists_extended_session_control_state -- --exact`

Expected: FAIL，因为 schema 和 row struct 还缺字段。

**Step 3: Write minimal implementation**

补 `schema.sql` 字段、`SessionControlStateRow` 结构体、`load_session_control_state` 与 `upsert_session_control_state` 的 SQL 映射。

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml storage::tests::persists_extended_session_control_state -- --exact`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/storage/schema.sql src-tauri/src/storage/sqlite.rs src-tauri/src/storage/tests.rs
git commit -m "feat(session): extend control state persistence [SES-05][SES-06][MON-02]"
```

### Task 2: 补 pause / resume 与进程状态动作

**Files:**
- Modify: `src-tauri/src/actions/session_control.rs`
- Modify: `src-tauri/src/actions/tests.rs`
- Modify: `src-tauri/src/commands/actions.rs`
- Modify: `src-tauri/src/desktop.rs`

**Step 1: Write the failing test**

在 `src-tauri/src/actions/tests.rs` 增加测试，覆盖：
- `pause_existing_session` 会把状态写成 paused，并写入 `session_pause` 审计事件
- `resume_existing_session` 会清除 paused，并保留/刷新运行状态
- `continue_existing_session` 在 paused 状态下拒绝
- `resume/continue/attach/detach` 会更新 process 元数据、事件数与最近活跃时间

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml actions::tests::pauses_and_resumes_supported_session --lib -- --exact`

Run: `cargo test --manifest-path src-tauri/Cargo.toml actions::tests::refuses_continue_for_paused_session --lib -- --exact`

Expected: FAIL，因为 pause/resume 动作不存在且状态未扩展。

**Step 3: Write minimal implementation**

在动作层新增 `pause_session`，并把 `resume/continue/attach/detach` 统一接入状态更新时间函数。

**Step 4: Run test to verify it passes**

Run: 同上两个测试，再补 `cargo test --manifest-path src-tauri/Cargo.toml actions::tests::updates_process_metadata_during_session_control --lib -- --exact`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/actions/session_control.rs src-tauri/src/actions/tests.rs src-tauri/src/commands/actions.rs src-tauri/src/desktop.rs
git commit -m "feat(session): add pause resume lifecycle controls [SES-05][SES-06]"
```

### Task 3: 让 snapshot 与 live HUD 暴露运行态细节

**Files:**
- Modify: `src-tauri/src/commands/dashboard.rs`
- Modify: `web/src/lib/api.ts`
- Modify: `web/src/components/active-session-cockpit.tsx`
- Modify: `web/src/components/active-session-cockpit.test.tsx`
- Modify: `web/src/components/session-detail.tsx`
- Modify: `web/src/components/session-detail.test.tsx`
- Modify: `web/src/lib/i18n.tsx`

**Step 1: Write the failing test**

新增 Rust 与 Web 测试，要求快照和组件可见：
- `paused / running / idle / detached / unavailable`
- `processId / processState / exitCode / runtimeSeconds / eventCount / token totals / lastActivityAt`

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml commands::dashboard::tests::builds_live_session_hud_fields --lib -- --exact`

Run: `npx vitest run src/components/active-session-cockpit.test.tsx src/components/session-detail.test.tsx src/lib/api.test.ts`

Expected: FAIL，因为当前 record 和 UI 还没有这些字段。

**Step 3: Write minimal implementation**

扩展 `SessionControlRecord`、Web 类型和控件文案，在 cockpit 与详情卡片显示最小 HUD。

**Step 4: Run test to verify it passes**

Run: 同上

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/commands/dashboard.rs web/src/lib/api.ts web/src/components/active-session-cockpit.tsx web/src/components/active-session-cockpit.test.tsx web/src/components/session-detail.tsx web/src/components/session-detail.test.tsx web/src/lib/i18n.tsx
git commit -m "feat(session): surface live control diagnostics in dashboard [MON-02][SES-06]"
```

### Task 4: 增加 HTTP control API 与 OpenAPI

**Files:**
- Modify: `src-tauri/src/api_server.rs`
- Modify: `src-tauri/src/openapi.rs`
- Test: `src-tauri/tests/http_api.rs`

**Step 1: Write the failing test**

在 `src-tauri/tests/http_api.rs` 增加集成测试，覆盖：
- `POST /api/v1/sessions/{sessionId}/resume`
- `POST /api/v1/sessions/{sessionId}/pause`
- `POST /api/v1/sessions/{sessionId}/attach`
- `POST /api/v1/sessions/{sessionId}/detach`
- `POST /api/v1/sessions/{sessionId}/continue`
- 受 token 保护
- OpenAPI 文档包含这些路径和 payload schema

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test http_api serve_command_exposes_session_control_routes -- --exact`

Expected: FAIL，因为端点尚未存在。

**Step 3: Write minimal implementation**

在 `api_server.rs` 中接入 JSON body、动作执行与错误映射，成功后返回更新后的 session detail 或 dashboard snapshot；同步补 OpenAPI paths 和 schema。

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test http_api serve_command_exposes_session_control_routes -- --exact`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/api_server.rs src-tauri/src/openapi.rs src-tauri/tests/http_api.rs
git commit -m "feat(api): add session control HTTP endpoints [API-02]"
```

### Task 5: 相关验证、spec 更新与 Git 证据

**Files:**
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`
- Modify: review snapshots under `.git/worktrees/feat-usability-clarity/osm/reviews/`

**Step 1: Run focused verification**

Run:
- `cargo test --manifest-path src-tauri/Cargo.toml storage::tests::persists_extended_session_control_state -- --exact`
- `cargo test --manifest-path src-tauri/Cargo.toml actions::tests::pauses_and_resumes_supported_session --lib -- --exact`
- `cargo test --manifest-path src-tauri/Cargo.toml actions::tests::refuses_continue_for_paused_session --lib -- --exact`
- `cargo test --manifest-path src-tauri/Cargo.toml actions::tests::updates_process_metadata_during_session_control --lib -- --exact`
- `cargo test --manifest-path src-tauri/Cargo.toml commands::dashboard::tests::builds_live_session_hud_fields --lib -- --exact`
- `cargo test --manifest-path src-tauri/Cargo.toml --test http_api serve_command_exposes_session_control_routes -- --exact`
- `npx vitest run src/lib/api.test.ts src/components/active-session-cockpit.test.tsx src/components/session-detail.test.tsx`

**Step 2: Capture Git evidence**

Run `git-review-snapshot` 和 `git-tdd-checkpoint` 为 `SES-05`、`SES-06`、`MON-02`、`API-02` 记录 `red/green/verify`。

**Step 3: Update spec**

把对应条目从 `todo/partial` 更新为真实状态，并注明验收证据。

**Step 4: Commit**

```bash
git add docs/specs/2026-03-22-autonomous-git-tdd-program.md .git/worktrees/feat-usability-clarity/osm/reviews
git commit -m "docs(spec): record session control lifecycle verification [SES-05][SES-06][MON-02][API-02]"
```

## Verification Log

- `cargo test --manifest-path src-tauri/Cargo.toml --lib`
- `cargo test --manifest-path src-tauri/Cargo.toml --test http_api`
- `npx vitest run src/lib/api.test.ts src/components/active-session-cockpit.test.tsx src/components/session-detail.test.tsx`
