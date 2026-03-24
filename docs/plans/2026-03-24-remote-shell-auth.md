# Remote Shell Auth Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 `serve` 增加可测试的短期远程壳层鉴权能力，让本地浏览器或壳层可以先领取短期访问令牌，再使用该令牌访问受保护 API。

**Architecture:** 在现有静态 Bearer token 之外，新增一个 loopback-only 的本地令牌发行端点。服务启动时持有进程内签名 secret，发行短期 signed token；`authorize` 同时接受静态 token 或有效的短期 signed token。保持实现单进程、无外部存储、无异步队列。

**Tech Stack:** Rust, Axum, Chrono, SHA-256, OpenAPI, `cargo test`

---

### Task 1: 为本地令牌发行写失败测试

**Files:**
- Modify: `src-tauri/tests/http_api.rs`
- Test: `src-tauri/tests/http_api.rs`

**Step 1: Write the failing test**

- 新增 `serve_command_issues_local_shell_token_and_accepts_it`
- 新增 `serve_command_rejects_expired_local_shell_token`
- 新增 OpenAPI 断言，检查 auth route 和 schema 暴露

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test http_api serve_command_issues_local_shell_token_and_accepts_it`

Expected: FAIL，因为 `/api/v1/auth/local-token` 还不存在。

**Step 3: Write minimal implementation**

- 在 `api_server` 增加 auth route、request/response schema、loopback 检查和 signed token 校验。

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test http_api serve_command_issues_local_shell_token_and_accepts_it`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/tests/http_api.rs src-tauri/src/api_server.rs src-tauri/src/openapi.rs
git commit -m "feat(api): add local shell auth tokens [API-09]"
```

### Task 2: 把令牌鉴权接进现有 Bearer 入口

**Files:**
- Modify: `src-tauri/src/api_server.rs`
- Test: `src-tauri/tests/http_api.rs`

**Step 1: Write the failing test**

- 让 `/api/v1/sessions`、`/metrics`、automation routes 都能接受 local auth token

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test http_api serve_command_rejects_expired_local_shell_token`

Expected: FAIL，因为过期校验和签名校验尚未实现。

**Step 3: Write minimal implementation**

- 统一 `authorize`，先接受静态 token，再接受 signed token。

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test http_api serve_command_rejects_expired_local_shell_token`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/api_server.rs src-tauri/tests/http_api.rs
git commit -m "feat(api): validate signed shell tokens [API-09]"
```

### Task 3: 文档与发布契约同步

**Files:**
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`
- Modify: `src-tauri/src/openapi.rs`

**Step 1: Write the failing test**

- 用 `http_api` OpenAPI 断言暴露 auth route

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test http_api serve_command_exposes_health_and_session_routes`

Expected: FAIL，如果 OpenAPI 没同步。

**Step 3: Write minimal implementation**

- 更新 OpenAPI 和发布文档，使功能矩阵与真实行为一致。

**Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test http_api`

Expected: PASS

**Step 5: Commit**

```bash
git add docs/specs/2026-03-22-autonomous-git-tdd-program.md docs/release/support-matrix.md docs/release/github-release-notes.md src-tauri/src/openapi.rs
git commit -m "docs(api): document remote shell auth [API-09]"
```
