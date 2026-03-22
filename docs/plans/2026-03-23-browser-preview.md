# Browser Preview Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 补齐 `GIT-08`，把 OSM 的浏览器运行链路从“只适合开发态 dev server”推进到“可直接 build 后 preview”的正式脚本，并让 Playwright E2E 复用这条链路。

**Architecture:** 不改业务代码，只收敛 Web 运行脚本和验证入口。先写一个 Node 级红测，锁定 `web/package.json` 必须提供正式 `preview/browser` 脚本，且 `web/playwright.config.ts` 的 `webServer` 必须走 preview 而不是 dev。随后补最小脚本实现和文档说明，最后把总 spec 与发布说明同步成完成态。

**Tech Stack:** Node.js, Vite, Playwright

---

### Task 1: 先写失败测试

**Files:**
- Create: `tests/web-preview/browser-preview.test.mjs`

**Step 1: Write the failing tests**

Cover:
- `web/package.json` 提供正式 `preview` 脚本
- `web/package.json` 提供面向用户的 `browser` 启动脚本
- `web/playwright.config.ts` 的 `webServer.command` 走 `npm run browser`

**Step 2: Run tests to verify they fail**

Run:
- `node --test tests/web-preview/browser-preview.test.mjs`

**Step 3: Commit**

```bash
git add tests/web-preview/browser-preview.test.mjs
git commit -m "test(web): cover browser preview launch chain [GIT-08]"
```

### Task 2: 实现正式 preview 链路

**Files:**
- Modify: `web/package.json`
- Modify: `web/playwright.config.ts`
- Modify: `README.md`

**Step 1: Write minimal implementation**

- 增加 `preview` 脚本
- 增加 `browser` 脚本，固定 `build + vite preview --host 127.0.0.1 --port 4173 --strictPort`
- Playwright `webServer` 改走 `npm run browser`
- README 补浏览器预览启动命令

**Step 2: Run tests to verify they pass**

Run:
- `node --test tests/web-preview/browser-preview.test.mjs`
- `npm --prefix web run build`

**Step 3: Commit**

```bash
git add web/package.json web/playwright.config.ts README.md
git commit -m "feat(web): add browser preview launch chain [GIT-08]"
```

### Task 3: 文档与 verify

**Files:**
- Modify: `docs/specs/2026-03-22-autonomous-git-tdd-program.md`
- Modify: `docs/release/support-matrix.md`
- Modify: `docs/release/github-release-notes.md`
- Modify: `scripts/verify.ps1`

**Step 1: Run verification**

Run:
- `node --test tests/web-preview/browser-preview.test.mjs`
- `npm --prefix web run e2e`

**Step 2: Update docs**

- 把 `GIT-08` 标成完成
- 支持矩阵、发布说明与 verify 链路同步 browser preview

**Step 3: Commit**

```bash
git add docs/specs/2026-03-22-autonomous-git-tdd-program.md docs/release/support-matrix.md docs/release/github-release-notes.md scripts/verify.ps1
git commit -m "docs(web): record browser preview coverage [GIT-08]"
```
