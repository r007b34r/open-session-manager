# GitHub Copilot CLI Session Control Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 `github-copilot-cli` 补齐既有会话的真实 resume / continue 控制能力，并让 snapshot / HTTP API 正确暴露其可控状态。

**Architecture:** 继续复用现有 `session_control` 动作层，不新增新的控制表结构。`github-copilot-cli` 只补一条新的 controller 分支和命令构造器，统一写入已有 `session_control_state / session_control_events / audit_events`。快照层只新增支持矩阵与命令解析；HTTP 控制 API 复用现有通用路由，不额外加专用端点。

**Tech Stack:** Rust, rusqlite, axum, serde_json

**Status:** 已完成并验证。`SES-02 / SES-03 / SES-07` 现已把 `GitHub Copilot CLI` 纳入真实控制覆盖。

---

### Task 1: 为 Copilot 控制链路写失败测试

**Files:**
- Modify: `src-tauri/src/actions/tests.rs`
- Modify: `src-tauri/tests/http_api.rs`

**Step 1: Write the failing test**

在 `src-tauri/src/actions/tests.rs` 新增测试：
- `resumes_supported_copilot_session_and_records_control_state`
- `continues_attached_copilot_session_and_persists_audit_event_for_copilot`

要求断言：
- `resume_session` 对 `github-copilot-cli` 不再返回 unsupported
- 执行命令包含 `--resume=<sessionId>` 与 `-p`
- `continue_session` 能写入 `last_prompt / last_response / session_continue`

在 `src-tauri/tests/http_api.rs` 新增一个集成测试：
- `serve_command_controls_copilot_session_via_http`

要求断言：
- fixture 里的 `copilot-ses-1` 可以通过 `/resume` 和 `/continue` 控制
- 返回的 session detail 里 `assistant == github-copilot-cli`
- `sessionControl.controller == github-copilot-cli`

**Step 2: Run test to verify it fails**

Run:
- `cargo test --manifest-path src-tauri/Cargo.toml actions::tests::resumes_supported_copilot_session_and_records_control_state --lib -- --exact`
- `cargo test --manifest-path src-tauri/Cargo.toml actions::tests::continues_attached_copilot_session_and_persists_audit_event_for_copilot --lib -- --exact`
- `cargo test --manifest-path src-tauri/Cargo.toml --test http_api serve_command_controls_copilot_session_via_http -- --exact`

Expected: FAIL，因为当前 `session_control` 只支持 `codex / claude-code`。

**Step 3: Write minimal implementation**

先只改最小集合：
- `src-tauri/src/actions/session_control.rs`
- `src-tauri/src/commands/dashboard.rs`

实现：
- `resolve_controller` 支持 `github-copilot-cli`
- 新增 `OPEN_SESSION_MANAGER_COPILOT_COMMAND`
- 新增 `build_copilot_command`
- 命令使用 `copilot --resume=<sessionId> -p <prompt> -s --output-format text`
- dashboard 把 `github-copilot-cli` 视为 `supported`

**Step 4: Run test to verify it passes**

Run: 同上三个命令

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/actions/session_control.rs src-tauri/src/actions/tests.rs src-tauri/src/commands/dashboard.rs src-tauri/tests/http_api.rs docs/plans/2026-03-23-copilot-session-control.md
git commit -m "feat(session): add copilot cli control support [SES-02][SES-03][SES-07]"
```

### Task 2: 校验 snapshot / one-click resume 入口状态

**Files:**
- Modify: `src-tauri/src/commands/dashboard.rs`
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`

**Step 1: Write the failing test**

在 `src-tauri/src/commands/dashboard.rs` 的测试中新增一个用 copilot fixture 的断言：
- `fixture_snapshot_marks_copilot_session_control_supported`

要求断言：
- `copilot-ses-1` 的 `sessionControl.supported == true`
- `controller == github-copilot-cli`
- `command == copilot` 或环境变量注入值

**Step 2: Run test to verify it fails**

Run:
- `cargo test --manifest-path src-tauri/Cargo.toml commands::dashboard::tests::fixture_snapshot_marks_copilot_session_control_supported --lib -- --exact`

Expected: FAIL，因为 dashboard 目前不把 copilot 计入控制支持列表。

**Step 3: Write minimal implementation**

只补 dashboard 支持矩阵与测试需要的 env 注入。

**Step 4: Run test to verify it passes**

Run:
- `cargo test --manifest-path src-tauri/Cargo.toml commands::dashboard::tests::fixture_snapshot_marks_copilot_session_control_supported --lib -- --exact`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/commands/dashboard.rs docs/specs/2026-03-22-autonomous-git-tdd-program.md
git commit -m "docs(spec): record copilot session control coverage [SES-02][SES-03][SES-07]"
```

### Task 3: 做聚焦验证并更新 spec

**Files:**
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`

**Step 1: Run focused verification**

Run:
- `cargo test --manifest-path src-tauri/Cargo.toml actions::tests::resumes_supported_copilot_session_and_records_control_state --lib -- --exact`
- `cargo test --manifest-path src-tauri/Cargo.toml actions::tests::continues_attached_copilot_session_and_persists_audit_event_for_copilot --lib -- --exact`
- `cargo test --manifest-path src-tauri/Cargo.toml commands::dashboard::tests::fixture_snapshot_marks_copilot_session_control_supported --lib -- --exact`
- `cargo test --manifest-path src-tauri/Cargo.toml --test http_api serve_command_controls_copilot_session_via_http -- --exact`
- `cargo test --manifest-path src-tauri/Cargo.toml --lib`
- `cargo test --manifest-path src-tauri/Cargo.toml --test http_api`

**Step 2: Capture Git evidence**

Run:
- `node scripts/git-tdd-checkpoint.mjs --item SES-02 --phase verify --note "copilot resume coverage verified" --command "cargo test --manifest-path src-tauri/Cargo.toml --test http_api"`
- `node scripts/git-tdd-checkpoint.mjs --item SES-03 --phase verify --note "copilot continue coverage verified" --command "cargo test --manifest-path src-tauri/Cargo.toml --lib"`
- `node scripts/git-tdd-checkpoint.mjs --item SES-07 --phase verify --note "copilot one-click resume path verified" --command "cargo test --manifest-path src-tauri/Cargo.toml --test http_api serve_command_controls_copilot_session_via_http -- --exact"`

**Step 3: Update spec**

把 `SES-02 / SES-03 / SES-07` 从“只覆盖 Codex / Claude Code”更新成“已覆盖 Codex / Claude Code / GitHub Copilot CLI”，仍保持 `partial`，因为其他助手尚未接入。

**Step 4: Commit**

```bash
git add docs/specs/2026-03-22-autonomous-git-tdd-program.md
git commit -m "docs(spec): record copilot control verification [SES-02][SES-03][SES-07]"
```

## Verification Log

- `cargo test --manifest-path src-tauri/Cargo.toml actions::tests::resumes_supported_copilot_session_and_records_control_state --lib -- --exact`
- `cargo test --manifest-path src-tauri/Cargo.toml actions::tests::continues_attached_session_and_persists_audit_event_for_copilot --lib -- --exact`
- `cargo test --manifest-path src-tauri/Cargo.toml commands::dashboard::tests::fixture_snapshot_marks_copilot_session_control_supported --lib -- --exact`
- `cargo test --manifest-path src-tauri/Cargo.toml --test http_api serve_command_controls_copilot_session_via_http -- --exact`
- `cargo test --manifest-path src-tauri/Cargo.toml --lib`
- `cargo test --manifest-path src-tauri/Cargo.toml --test http_api`
