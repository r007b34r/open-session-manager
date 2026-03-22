import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    window.localStorage.setItem("open-session-manager.enable-demo-data", "1");
  });
  await page.route("**/dashboard-snapshot.json", (route) => route.abort());
});

test("indexes fixtures, exports, soft-deletes, and shows risky masked config entries", async ({
  page
}) => {
  await page.goto("/");

  await expect(
    page.getByRole("heading", { name: /open session manager/i })
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: /refactor wsl collector handshake/i })
  ).toBeVisible();

  await page.getByRole("button", { name: /export markdown/i }).click();
  await page.getByRole("button", { name: /move to quarantine/i }).click();
  await page
    .getByRole("checkbox", {
      name: /i exported the valuable parts and want to continue/i
    })
    .click();
  await page.getByRole("button", { name: /confirm move to quarantine/i }).click();

  await page
    .getByRole("navigation", { name: /primary/i })
    .getByRole("link", { name: "Audit", exact: true })
    .click();
  await expect(
    page.getByText(/exported markdown digest for refactor wsl collector handshake/i)
  ).toBeVisible();
  await expect(
    page.getByText(/moved refactor wsl collector handshake into the quarantine queue/i)
  ).toBeVisible();

  await page
    .getByRole("navigation", { name: /primary/i })
    .getByRole("link", { name: "Configs", exact: true })
    .click();
  await expect(page.getByText("***6789")).toBeVisible();
  await expect(page.getByText("https://relay.cch.example/v1")).toBeVisible();
});

test("keeps the session workspace aligned when filtering on a narrower viewport", async ({
  page
}) => {
  await page.setViewportSize({ width: 980, height: 900 });
  await page.goto("/#/sessions");

  await page.getByRole("button", { name: /audit anthropic relay settings/i }).click();
  await expect(
    page.getByRole("heading", { name: /audit anthropic relay settings/i })
  ).toBeVisible();

  await page.getByRole("searchbox", { name: /search sessions/i }).fill("definitely-no-match");

  await expect(
    page.getByRole("heading", { name: /select a session/i })
  ).toBeVisible();
});

test("selects a session when the user clicks the row instead of the title button", async ({
  page
}) => {
  await page.goto("/#/sessions");

  await page.locator(".data-table tbody tr").nth(1).click();

  await expect(
    page.getByRole("heading", { name: /audit anthropic relay settings/i })
  ).toBeVisible();
});

test("keeps the session detail panel non-sticky and single-column to avoid stretched cards", async ({
  page
}) => {
  await page.setViewportSize({ width: 1320, height: 920 });
  await page.goto("/#/sessions");

  await page.getByRole("button", { name: /audit anthropic relay settings/i }).click();
  await expect(
    page.getByRole("heading", { name: /audit anthropic relay settings/i })
  ).toBeVisible();

  const detailLayout = await page.locator(".detail-panel").evaluate((element) => {
    const style = window.getComputedStyle(element);
    return {
      position: style.position,
      overflowY: style.overflowY
    };
  });
  const detailCardColumns = await page.locator(".detail-card-grid").evaluate((element) => {
    const style = window.getComputedStyle(element);
    return style.gridTemplateColumns
      .split(" ")
      .map((value) => value.trim())
      .filter(Boolean).length;
  });

  expect(detailLayout.position).toBe("static");
  expect(detailLayout.overflowY).toBe("visible");
  expect(detailCardColumns).toBe(1);
});

test("shows the markdown export location and lets the user override the export folder", async ({
  page
}) => {
  await page.goto("/#/sessions");

  await expect(
    page.getByLabel(/markdown export folder/i)
  ).toHaveValue(/OpenSessionManager[\\/]+exports/i);

  await page.getByLabel(/markdown export folder/i).fill("D:/OSM/exports");
  await page.getByRole("button", { name: /save export folder/i }).click();
  await page.getByRole("button", { name: /export markdown/i }).click();

  await expect(page.getByLabel(/markdown export folder/i)).toHaveValue("D:/OSM/exports");
  await expect(page.getByText(/d:\/osm\/exports\/session-ses-001\.md/i)).toBeVisible();
});

test("supports system light-dark switching from the shell controls", async ({ page }) => {
  await page.goto("/");

  await page.getByRole("button", { name: "Dark", exact: true }).click();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");

  await page.getByRole("button", { name: "Light", exact: true }).click();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
});

test("supports one-click resume and continue actions in demo mode", async ({ page }) => {
  await page.goto("/#/sessions");

  await page.getByRole("button", { name: /resume session/i }).click();
  await expect(page.getByText(/ready from demo resume/i)).toBeVisible();

  await page.getByLabel(/continue prompt/i).fill("Continue with the next verification step.");
  await page.getByRole("button", { name: /continue session/i }).click();

  await expect(
    page.getByText(/ready from demo continue: continue with the next verification step\./i)
  ).toBeVisible();
});

test("keeps the overview page scroll position stable when selecting an embedded session", async ({
  page
}) => {
  await page.setViewportSize({ width: 1320, height: 920 });
  await page.goto("/");

  const targetButton = page.getByRole("button", {
    name: /audit anthropic relay settings/i
  });
  await targetButton.scrollIntoViewIfNeeded();
  const beforeScrollY = await page.evaluate(() => window.scrollY);

  await targetButton.click();

  await expect(
    page.getByRole("heading", { name: /audit anthropic relay settings/i }).last()
  ).toBeVisible();
  await expect(page).toHaveURL(/\/$/);

  const afterScrollY = await page.evaluate(() => window.scrollY);
  expect(Math.abs(afterScrollY - beforeScrollY)).toBeLessThan(32);
});

test("highlights the matched transcript when a search result is selected", async ({
  page
}) => {
  await page.goto("/#/sessions");

  await page.getByRole("searchbox", { name: /search sessions/i }).fill(
    "anthropic_base_url override"
  );
  await page.getByRole("button", { name: /audit anthropic relay settings/i }).click();

  await expect(page.getByText(/search hit/i)).toBeVisible();
  const matchedEntry = page.locator(".detail-transcript-entry.is-search-match");
  await expect(matchedEntry).toContainText(/mapped anthropic_base_url override/i);
  await expect(matchedEntry.locator("mark")).toHaveCount(2);
  await expect(matchedEntry.locator("mark").nth(1)).toHaveText(/override/i);
});

test("shows the active session cockpit and refreshes runtime status", async ({
  page
}) => {
  await page.unroute("**/dashboard-snapshot.json");
  await page.addInitScript(() => {
    window.localStorage.removeItem("open-session-manager.enable-demo-data");
  });

  let requestCount = 0;
  await page.route("**/dashboard-snapshot.json", async (route) => {
    requestCount += 1;
    const snapshot =
      requestCount === 1
        ? buildRuntimeSnapshot("READY from initial snapshot", false)
        : buildRuntimeSnapshot("READY from refreshed snapshot", true);

    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(snapshot)
    });
  });

  await page.goto("/");

  const cockpit = page.locator(".cockpit-panel");

  await expect(page.getByRole("heading", { name: /active session cockpit/i })).toBeVisible();
  await expect(cockpit.getByText(/ready from initial snapshot/i)).toBeVisible();

  await cockpit.getByRole("button", { name: /refresh cockpit/i }).click();

  await expect(cockpit.getByText(/ready from refreshed snapshot/i)).toBeVisible();
  await expect(cockpit.getByText(/^attached$/i)).toBeVisible();
});

function buildRuntimeSnapshot(lastResponse: string, attached: boolean) {
  return {
    metrics: [],
    sessions: [
      {
        sessionId: "ses-mon-001",
        title: "Resume Codex rollout",
        assistant: "Codex",
        progressState: "completed",
        progressPercent: 100,
        lastActivityAt: "2026-03-23T01:00:00.000Z",
        environment: "Windows 11",
        valueScore: 88,
        summary: "Runtime snapshot for the active cockpit.",
        projectPath: "C:/Projects/osm",
        sourcePath: "C:/Users/Max/.codex/sessions/demo.jsonl",
        tags: [],
        riskFlags: [],
        keyArtifacts: [],
        transcriptHighlights: [],
        todoItems: [],
        sessionControl: {
          supported: true,
          available: true,
          controller: "codex",
          command: "codex",
          attached,
          lastResponse
        }
      }
    ],
    configs: [],
    doctorFindings: [],
    auditEvents: [],
    usageOverview: {
      totals: {
        sessionsWithUsage: 0,
        inputTokens: 0,
        outputTokens: 0,
        cacheReadTokens: 0,
        cacheWriteTokens: 0,
        reasoningTokens: 0,
        totalTokens: 0,
        costSource: "unknown"
      },
      assistants: []
    },
    usageTimeline: [],
    runtime: {
      auditDbPath: "C:/Users/Max/AppData/Local/OpenSessionManager/audit/audit.db",
      exportRoot: "C:/Users/Max/Documents/OpenSessionManager/exports",
      defaultExportRoot: "C:/Users/Max/Documents/OpenSessionManager/exports",
      exportRootSource: "default",
      quarantineRoot: "C:/Users/Max/AppData/Local/OpenSessionManager/quarantine",
      preferencesPath:
        "C:/Users/Max/AppData/Local/OpenSessionManager/preferences.json"
    }
  };
}
