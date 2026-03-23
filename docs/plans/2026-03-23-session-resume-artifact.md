# Session Resume Artifact Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 `SES-09` 增加标准化 `resume artifact`，把导出、隔离、恢复三段链路收成可审计的一对一工件引用。

**Architecture:** Rust 导出动作在 Markdown 与 cleanup checklist 之外额外生成 `resume-<session>.json`，内容承载 handoff 摘要、继续提示和工件路径。软删除 manifest 记录该 artifact 路径，恢复审计继续透传它；dashboard 和 Web Audit 页只做字段透传与展示，不重新推导业务含义。

**Tech Stack:** Rust + rusqlite + serde_json，Tauri snapshot command，React + Vitest

---

### Task 1: Rust 导出链路生成标准化 resume artifact

**Files:**
- Create: `docs/plans/2026-03-23-session-resume-artifact.md`
- Modify: `src-tauri/src/actions/tests.rs`
- Modify: `src-tauri/src/actions/export.rs`

**Step 1: Write the failing test**

在 `src-tauri/src/actions/tests.rs` 增加导出测试，断言：
- `ExportResult` 暴露 `resume_artifact_path`
- `resume-<session>.json` 实际存在
- JSON 内含 `sessionId / assistant / exportPath / checklistPath / resumeCue / nextFocus`

**Step 2: Run test to verify it fails**

Run: `cargo test exports_soft_deletes_restores_and_audits_session --manifest-path src-tauri/Cargo.toml -- --exact`

Expected: FAIL，提示 `resume_artifact_path` 缺失或 artifact 文件不存在。

**Step 3: Write minimal implementation**

在 `export.rs`：
- 定义 `ResumeArtifact`
- 生成 `resume-<session>.json`
- 把路径写入 `ExportResult`
- 在 `export_markdown` 审计 `after_state` 写入 `resume_artifact_path`

**Step 4: Run test to verify it passes**

Run: `cargo test exports_soft_deletes_restores_and_audits_session --manifest-path src-tauri/Cargo.toml -- --exact`

Expected: PASS

**Step 5: Commit**

```bash
git add docs/plans/2026-03-23-session-resume-artifact.md src-tauri/src/actions/tests.rs src-tauri/src/actions/export.rs
git commit -m "feat(session): add resume artifact export chain [SES-09]"
```

### Task 2: 软删除、恢复与 dashboard 审计透传 resume artifact

**Files:**
- Modify: `src-tauri/src/actions/mod.rs`
- Modify: `src-tauri/src/actions/delete.rs`
- Modify: `src-tauri/src/actions/restore.rs`
- Modify: `src-tauri/src/actions/tests.rs`
- Modify: `src-tauri/src/commands/dashboard.rs`
- Modify: `src-tauri/tests/cli_snapshot.rs`

**Step 1: Write the failing test**

补测试断言：
- `QuarantineManifest` 有 `resume_artifact_path`
- `soft_delete` 与 `restore` 审计包含该路径
- `snapshot` 输出的 `auditEvents[*].resumeArtifactPath` 可见

**Step 2: Run test to verify it fails**

Run: `cargo test exports_soft_deletes_restores_and_audits_session --manifest-path src-tauri/Cargo.toml -- --exact`

Run: `cargo test snapshot_command_includes_persisted_audit_history --manifest-path src-tauri/Cargo.toml --test cli_snapshot -- --exact`

Expected: FAIL，提示 manifest 或 snapshot 中缺路径字段。

**Step 3: Write minimal implementation**

实现：
- manifest 增加 `resume_artifact_path`
- soft delete 从最近一次成功导出审计读取该字段
- restore 审计继续回写该字段
- dashboard `AuditEventRecord` 与 `parse_audit_paths` 增加 `resume_artifact_path`

**Step 4: Run test to verify it passes**

Run: 与 Step 2 相同

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/actions/mod.rs src-tauri/src/actions/delete.rs src-tauri/src/actions/restore.rs src-tauri/src/actions/tests.rs src-tauri/src/commands/dashboard.rs src-tauri/tests/cli_snapshot.rs
git commit -m "feat(session): wire resume artifact through quarantine audit [SES-09]"
```

### Task 3: Web Audit 页展示 resume artifact 路径

**Files:**
- Modify: `web/src/lib/api.ts`
- Modify: `web/src/lib/api.test.ts`
- Modify: `web/src/lib/i18n.tsx`
- Modify: `web/src/routes/audit.tsx`
- Modify: `web/src/app.test.tsx`

**Step 1: Write the failing test**

补前端测试断言：
- `AuditEventRecord` 接受 `resumeArtifactPath`
- Audit 页在有该路径时展示标签和具体路径

**Step 2: Run test to verify it fails**

Run: `npx vitest run web/src/lib/api.test.ts web/src/app.test.tsx`

Expected: FAIL，提示新字段未透传或 UI 未渲染。

**Step 3: Write minimal implementation**

实现：
- API 类型与 normalize 逻辑支持 `resumeArtifactPath`
- i18n 增加中英文标签
- Audit 页追加该路径行

**Step 4: Run test to verify it passes**

Run: `npx vitest run web/src/lib/api.test.ts web/src/app.test.tsx`

Expected: PASS

**Step 5: Commit**

```bash
git add web/src/lib/api.ts web/src/lib/api.test.ts web/src/lib/i18n.tsx web/src/routes/audit.tsx web/src/app.test.tsx
git commit -m "feat(web): surface resume artifact audit paths [SES-09]"
```

### Task 4: 更新 spec 与 Git 证据

**Files:**
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`

**Step 1: Run verification snapshots**

Run:
- `node scripts/git-review-snapshot.mjs --item SES-09 --phase red --command "cargo test exports_soft_deletes_restores_and_audits_session --manifest-path src-tauri/Cargo.toml -- --exact"`
- `node scripts/git-review-snapshot.mjs --item SES-09 --phase green --command "cargo test exports_soft_deletes_restores_and_audits_session --manifest-path src-tauri/Cargo.toml -- --exact"`
- `node scripts/git-review-snapshot.mjs --item SES-09 --phase verify --command "cargo test snapshot_command_includes_persisted_audit_history --manifest-path src-tauri/Cargo.toml --test cli_snapshot -- --exact"`

**Step 2: Update spec**

把 `SES-09` 从 `partial` 改成 `done`，验收指标改成已完成描述。

**Step 3: Record checkpoint**

Run:
- `node scripts/git-tdd-checkpoint.mjs --item SES-09 --phase verify --note "resume artifact export, quarantine, restore, dashboard and web audit all verified"`

