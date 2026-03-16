# Session Search Absorption Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add real local session search quality to OSM by replacing the current plain substring filter with weighted lexical matching, contextual snippets, and visible match reasons.

**Architecture:** Keep search offline and client-side for the current snapshot model. Build a dedicated search helper that tokenizes queries, scores matches across structured session fields, extracts the best snippet, and feeds enriched rows into the existing sessions route and table.

**Tech Stack:** TypeScript, React, Vitest

---

### Task 1: Lock Search Behavior With Failing Tests

**Files:**
- Create: `web/src/lib/session-search.test.ts`
- Modify: `web/src/app.test.tsx`

**Step 1: Write the failing test**

Add tests for:

- weighted ranking prefers transcript/title-rich matches
- quoted phrases stay intact
- result snippets and match reasons render in the sessions list

**Step 2: Run test to verify it fails**

Run: `npm --prefix web run test -- web/src/lib/session-search.test.ts web/src/app.test.tsx`
Expected: FAIL because no search helper or UI exists yet.

**Step 3: Write minimal implementation**

Only test code and expectations.

**Step 4: Run test to verify it still fails for the expected reason**

Expected: missing helper exports or missing snippet UI.

**Step 5: Commit**

```bash
git add web/src/lib/session-search.test.ts web/src/app.test.tsx
git commit -m "test: cover weighted session search"
```

### Task 2: Implement Search Helper

**Files:**
- Create: `web/src/lib/session-search.ts`
- Modify: `web/src/lib/api.ts`

**Step 1: Write the failing test**

Use Task 1 tests.

**Step 2: Run test to verify it fails**

Run: `npm --prefix web run test -- web/src/lib/session-search.test.ts`
Expected: FAIL because helper does not exist.

**Step 3: Write minimal implementation**

Add:

- query parser with quote support
- field-weighted lexical scoring
- AND semantics across query terms
- best snippet extraction
- match reason labeling for UI

**Step 4: Run test to verify it passes**

Run: `npm --prefix web run test -- web/src/lib/session-search.test.ts`
Expected: PASS.

**Step 5: Commit**

```bash
git add web/src/lib/session-search.ts web/src/lib/api.ts web/src/lib/session-search.test.ts
git commit -m "feat: add weighted session search model"
```

### Task 3: Surface Search Results In The Sessions UI

**Files:**
- Modify: `web/src/routes/sessions.tsx`
- Modify: `web/src/components/session-table.tsx`
- Modify: `web/src/lib/i18n.tsx`
- Modify: `web/src/styles.css`

**Step 1: Write the failing test**

Use the app test from Task 1.

**Step 2: Run test to verify it fails**

Run: `npm --prefix web run test -- web/src/app.test.tsx`
Expected: FAIL because snippets and match reasons are not rendered.

**Step 3: Write minimal implementation**

Render:

- result count / search state copy
- per-row snippet preview
- per-row match reason badges

**Step 4: Run test to verify it passes**

Run: `npm --prefix web run test -- web/src/app.test.tsx`
Expected: PASS.

**Step 5: Commit**

```bash
git add web/src/routes/sessions.tsx web/src/components/session-table.tsx web/src/lib/i18n.tsx web/src/styles.css web/src/app.test.tsx
git commit -m "feat: add search snippets and match reasons"
```

### Task 4: Full Verification

**Files:**
- Review only unless fixes are needed

**Step 1: Run targeted verification**

Run:

```bash
npm --prefix web run test -- web/src/lib/session-search.test.ts
npm --prefix web run test -- web/src/app.test.tsx
```

Expected: PASS.

**Step 2: Run full web verification**

Run:

```bash
npm --prefix web run test
npm --prefix web run build
```

Expected: PASS.

**Step 3: Record outcome**

Document that OSM now genuinely absorbs the first part of the search capability line:

- structured lexical ranking
- contextual snippets
- visible match sources

while BM25/semantic/MCP/API search remain pending.
