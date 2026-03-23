import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    window.localStorage.setItem("open-session-manager.enable-demo-data", "1");
  });
  await page.route("**/dashboard-snapshot.json", (route) => route.abort());
});

test("captures the overview shell without layout drift", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 1200 });
  await page.goto("/");

  await expect(
    page.getByRole("heading", { name: /open session manager/i })
  ).toBeVisible();
  await expect(page).toHaveScreenshot("overview-shell.png", {
    animations: "disabled",
    fullPage: true
  });
});

test("captures the session detail workspace without stretched cards", async ({ page }) => {
  await page.setViewportSize({ width: 1320, height: 1400 });
  await page.goto("/#/sessions");

  await page.getByRole("button", { name: /audit anthropic relay settings/i }).click();
  await expect(
    page.getByRole("heading", { name: /audit anthropic relay settings/i })
  ).toBeVisible();

  await expect(page).toHaveScreenshot("session-detail-shell.png", {
    animations: "disabled",
    fullPage: true
  });
});
