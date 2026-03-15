import { expect, test } from "@playwright/test";

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
