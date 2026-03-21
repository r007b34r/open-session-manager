# Autonomous Git TDD Program Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 把 OSM 的剩余缺口收敛成 Git 驱动的可执行 backlog，并交付本地可用的 TDD 检查点与审查工具。

**Architecture:** 先落文档侧的总 backlog 和执行协议，再用 Node 脚本把 annotated tags、git notes、review 快照和本地验证命令串起来，最后把这套流程接进统一验证脚本。

**Tech Stack:** Markdown, Node.js, `node:test`, Git CLI, PowerShell

---

### Task 1: 写专项 spec 与总执行计划

**Files:**
- Create: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`
- Create: `docs/plans/2026-03-22-autonomous-git-tdd-program.md`

**Step 1: 确认缺口存在**

Run: `rg -n "autonomous-git-tdd-program" docs/specs docs/plans`
Expected: 没有 2026-03-22 的专项 spec 和计划。

**Step 2: 写最小实现**

把全部未完成事项重新编号，并给出：

- backlog ID
- 优先级
- 当前状态
- 验收指标
- Git 驱动 TDD 协议

**Step 3: 验证文档存在**

Run: `Get-Content docs/specs/2026-03-22-autonomous-git-tdd-program.md`
Expected: 能看到完整 backlog 和 Git 流程约定。

**Step 4: 提交**

```bash
git add docs/specs/2026-03-22-autonomous-git-tdd-program.md docs/plans/2026-03-22-autonomous-git-tdd-program.md
git commit -m "docs: add autonomous git tdd program"
```

### Task 2: 先写 Git 工作流测试

**Files:**
- Create: `tests/git-workflow/git-workflow.test.mjs`

**Step 1: 写失败测试**

覆盖三件事：

- review snapshot CLI 能输出稳定 Markdown
- checkpoint CLI 的 dry run 不改 Git 仓库，但会打印计划中的 tag/note/path
- checkpoint CLI 真正执行时会创建 annotated tag、git note 和 review 文件

**Step 2: 运行并确认失败**

Run: `node --test tests/git-workflow/git-workflow.test.mjs`
Expected: FAIL，因为脚本和库还不存在。

**Step 3: 不写生产代码，只确认失败原因**

Expected: 缺少 `scripts/git-review-snapshot.mjs` 或 `scripts/git-tdd-checkpoint.mjs`。

**Step 4: 提交**

```bash
git add tests/git-workflow/git-workflow.test.mjs
git commit -m "test: cover git tdd workflow scripts"
```

### Task 3: 实现 Git 工作流库与两个 CLI

**Files:**
- Create: `scripts/lib/git-workflow.mjs`
- Create: `scripts/git-review-snapshot.mjs`
- Create: `scripts/git-tdd-checkpoint.mjs`

**Step 1: 使用 Task 2 的失败测试**

不新增其它测试入口，先让现有测试变绿。

**Step 2: 写最小实现**

至少支持：

- 解析 `--item`、`--phase`、`--command`、`--note`、`--repo-root`、`--dry-run`
- 采集 `git status --short --branch`、`git diff --stat`、`git log -1 --oneline`
- 写 review snapshot Markdown
- 创建 annotated tag
- 在 `refs/notes/osm-tdd` 下写 git note

**Step 3: 跑测试确认转绿**

Run: `node --test tests/git-workflow/git-workflow.test.mjs`
Expected: PASS。

**Step 4: 提交**

```bash
git add scripts/lib/git-workflow.mjs scripts/git-review-snapshot.mjs scripts/git-tdd-checkpoint.mjs tests/git-workflow/git-workflow.test.mjs
git commit -m "feat: add git driven tdd checkpoints"
```

### Task 4: 把新工具接进统一验证链路

**Files:**
- Modify: `scripts/verify.ps1`

**Step 1: 写失败测试**

没有单独自动测试，失败表现是统一验证脚本尚未覆盖新工具。

**Step 2: 确认缺口存在**

Run: `rg -n "git-workflow|git-review-snapshot|git-tdd-checkpoint" scripts/verify.ps1`
Expected: 没有命中新工具。

**Step 3: 写最小实现**

在 `scripts/verify.ps1` 里加入：

- `node --test tests/git-workflow/git-workflow.test.mjs`
- 新脚本 dry run 检查

**Step 4: 运行验证**

Run: `powershell -ExecutionPolicy Bypass -File scripts/verify.ps1`
Expected: 至少新加的 Node 测试和 dry run 步骤通过。

**Step 5: 提交**

```bash
git add scripts/verify.ps1
git commit -m "test: verify git workflow tooling"
```

### Task 5: 本地 Git 审查与记录

**Files:**
- Review only unless fixes are needed

**Step 1: 记录 red / green / verify**

Run:

```bash
node scripts/git-review-snapshot.mjs --item TOOL-01 --phase green --command "node --test tests/git-workflow/git-workflow.test.mjs"
node scripts/git-tdd-checkpoint.mjs --item TOOL-01 --phase green --note "Git 工作流脚本已转绿"
```

Expected: review 文件、annotated tag 和 git note 都生成成功。

**Step 2: 本地审查**

Run:

```bash
git status --short --branch
git diff --stat
git log --oneline -5
```

Expected: 变更范围清晰、提交粒度可审。

**Step 3: 完整验证**

Run:

```bash
node --test tests/git-workflow/git-workflow.test.mjs
powershell -ExecutionPolicy Bypass -File scripts/verify.ps1
```

Expected: PASS。

**Step 4: 提交**

```bash
git add .
git commit -m "chore: record autonomous git tdd workflow"
```

### Task 6: 用新机制推进首批 P0 条目

**Files:**
- Follow-on work, not part of this foundation patch

**Step 1: 从 `CFG-01` 开始**

先写写回失败测试，再按 `red -> green -> verify -> review` 执行。

**Step 2: 每完成一个 item，都留下 Git 证据**

- annotated tag
- git note
- review snapshot
- item ID in commit message

**Step 3: 更新 spec 状态**

每个条目推进后，必须回写 `docs/specs/2026-03-22-autonomous-git-tdd-program.md` 的状态字段。
