# Multi-Assistant Session Control Expansion Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 把 `Gemini CLI / Factory Droid / OpenClaw` 接入 OSM 的真实会话控制链路，补齐 `SES-02 / SES-03 / SES-07` 在动作层、快照层和 HTTP API 的剩余缺口。

**Architecture:** 继续复用现有 `session_control` 动作层和 `dashboard` 快照模型，不引入新的控制 transport。对每个助手只增加最小的 controller 解析、命令拼装和 fake CLI 测试；不改动已有 REST 路由结构。`Gemini CLI` 的 `--resume` 支持基于官方 checkpointing 能力与上游实现证据做保守接入；`Factory Droid` 与 `OpenClaw` 优先按官方 CLI 入口接线。

**Tech Stack:** Rust, cargo test, fake CLI executables, SQLite audit persistence

---

### Task 1: 为剩余三类助手写失败测试

**Files:**
- Modify: `src-tauri/src/actions/tests.rs`
- Modify: `src-tauri/tests/http_api.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`

**Step 1: Write the failing tests**

- 在 `actions/tests.rs` 新增 6 个测试：
  - `resumes_supported_gemini_session_and_records_control_state`
  - `continues_attached_session_and_persists_audit_event_for_gemini`
  - `resumes_supported_factory_droid_session_and_records_control_state`
  - `continues_attached_session_and_persists_audit_event_for_factory_droid`
  - `resumes_supported_openclaw_session_and_records_control_state`
  - `continues_attached_session_and_persists_audit_event_for_openclaw`
- 在 `http_api.rs` 新增 3 个测试：
  - `serve_command_controls_gemini_session_via_http`
  - `serve_command_controls_factory_droid_session_via_http`
  - `serve_command_controls_openclaw_session_via_http`
- 在 `dashboard.rs` 新增 3 个测试：
  - `fixture_snapshot_marks_gemini_session_control_supported`
  - `fixture_snapshot_marks_factory_droid_session_control_supported`
  - `fixture_snapshot_marks_openclaw_session_control_supported`

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml resumes_supported_gemini_session_and_records_control_state -- --exact
cargo test --manifest-path src-tauri/Cargo.toml --test http_api serve_command_controls_gemini_session_via_http -- --exact
cargo test --manifest-path src-tauri/Cargo.toml fixture_snapshot_marks_gemini_session_control_supported --lib -- --exact
```

Expected: FAIL，错误原因为 `session control is not supported` 或 snapshot `supported` 仍为 `false`。

**Step 3: Record red checkpoint**

Run:

```powershell
node scripts/git-review-snapshot.mjs --item SES-02 --phase red --command "cargo test --manifest-path src-tauri/Cargo.toml resumes_supported_gemini_session_and_records_control_state -- --exact"
node scripts/git-tdd-checkpoint.mjs --item SES-02 --phase red --note "remaining assistant controllers fail before implementation"
node scripts/git-tdd-checkpoint.mjs --item SES-03 --phase red --note "remaining assistant continue prompts fail before implementation"
node scripts/git-tdd-checkpoint.mjs --item SES-07 --phase red --note "snapshot still marks remaining assistants unsupported"
```

### Task 2: 实现动作层和快照层最小支持

**Files:**
- Modify: `src-tauri/src/actions/session_control.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`

**Step 1: Write minimal implementation**

- `resolve_controller` 新增：
  - `gemini-cli`
  - `factory-droid`
  - `openclaw`
- 新增命令拼装：
  - `gemini --resume <sessionId> <prompt>`
  - `droid exec -s <sessionId> <prompt>`
  - `openclaw agent --session-id <sessionId> --message <prompt> --json`
- `build_session_control_record` 把上述三类助手标记为 `supported = true`
- 新增对应环境变量：
  - `OPEN_SESSION_MANAGER_GEMINI_COMMAND`
  - `OPEN_SESSION_MANAGER_DROID_COMMAND`
  - `OPEN_SESSION_MANAGER_OPENCLAW_COMMAND`

**Step 2: Run tests to verify they pass**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml resumes_supported_gemini_session_and_records_control_state -- --exact
cargo test --manifest-path src-tauri/Cargo.toml continues_attached_session_and_persists_audit_event_for_factory_droid -- --exact
cargo test --manifest-path src-tauri/Cargo.toml fixture_snapshot_marks_openclaw_session_control_supported --lib -- --exact
```

Expected: PASS。

**Step 3: Record green checkpoint**

Run:

```powershell
node scripts/git-review-snapshot.mjs --item SES-02 --phase green --command "cargo test --manifest-path src-tauri/Cargo.toml resumes_supported_gemini_session_and_records_control_state -- --exact"
node scripts/git-tdd-checkpoint.mjs --item SES-02 --phase green --note "resume support expanded to gemini factory openclaw"
node scripts/git-tdd-checkpoint.mjs --item SES-03 --phase green --note "continue support expanded to gemini factory openclaw"
node scripts/git-tdd-checkpoint.mjs --item SES-07 --phase green --note "snapshot now marks all parsed assistants controllable"
```

### Task 3: 补 HTTP API 覆盖并更新 spec

**Files:**
- Modify: `src-tauri/tests/http_api.rs`
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`

**Step 1: Run focused HTTP/API verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --test http_api serve_command_controls_gemini_session_via_http -- --exact
cargo test --manifest-path src-tauri/Cargo.toml --test http_api serve_command_controls_factory_droid_session_via_http -- --exact
cargo test --manifest-path src-tauri/Cargo.toml --test http_api serve_command_controls_openclaw_session_via_http -- --exact
```

Expected: PASS。

**Step 2: Update docs/spec status**

- `SES-02 / SES-03 / SES-07` 从 `partial` 改为 `done`
- 支持矩阵和 release notes 补充 7 个助手的真实控制覆盖
- 对 `Gemini CLI` 标注“基于 checkpointing / resume CLI 证据接入”

**Step 3: Run verify and commit**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib
cargo test --manifest-path src-tauri/Cargo.toml --test http_api
node scripts/git-review-snapshot.mjs --item SES-07 --phase verify --command "cargo test --manifest-path src-tauri/Cargo.toml --test http_api"
node scripts/git-tdd-checkpoint.mjs --item SES-02 --phase verify --note "all supported assistants now expose real resume control"
node scripts/git-tdd-checkpoint.mjs --item SES-03 --phase verify --note "all supported assistants now expose real continue control"
node scripts/git-tdd-checkpoint.mjs --item SES-07 --phase verify --note "web/http snapshot now surfaces one-click control across all supported assistants"
git add src-tauri/src/actions/session_control.rs src-tauri/src/actions/tests.rs src-tauri/src/commands/dashboard.rs src-tauri/tests/http_api.rs docs/specs/2026-03-22-autonomous-git-tdd-program.md docs/release/support-matrix.md docs/release/github-release-notes.md docs/plans/2026-03-24-multi-assistant-session-control-expansion.md
git commit -m "feat(session): expand multi-assistant session control [SES-02][SES-03][SES-07]"
```
