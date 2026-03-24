## 背景

`SES-02 / SES-03 / SES-07` 目前真实控制只覆盖 `Codex / Claude Code / GitHub Copilot CLI`。`OpenCode` 已经有会话解析、导出、审计和 usage，但还没有接入真实恢复与继续执行。

官方 CLI 文档已经提供 `opencode run --session <id> <prompt>` 和 `opencode run --continue <prompt>` 两类非交互入口，因此可以直接复用 OSM 现有 session control 动作层，不需要再单独设计新的 transport。

## 方案

本次只做最小闭环：

- 动作层新增 `OpenCode` 控制器解析和命令拼装
- snapshot / dashboard 把 `OpenCode` 标记为可控助手
- HTTP control API 复用既有路由，验证 `resume` 与 `continue` 都能命中 `OpenCode`
- 用 fake `opencode` 可执行文件做 TDD，不引入真实外部依赖

## 测试入口

- `src-tauri/src/actions/tests.rs`
  - `resumes_supported_opencode_session_and_records_control_state`
  - `continues_attached_session_and_persists_audit_event_for_opencode`
- `src-tauri/src/commands/dashboard.rs`
  - `fixture_snapshot_marks_opencode_session_control_supported`
- `src-tauri/tests/http_api.rs`
  - `serve_command_controls_opencode_session_via_http`

## 验收

- `OpenCode` 会话详情显示 `supported = true`
- `resume` 与 `continue` 都会写回 `session_control_state`、`session_control_events`、`audit_events`
- HTTP API 返回的 session detail 中带有最新 `sessionControl`
- 相关聚焦测试全部通过
