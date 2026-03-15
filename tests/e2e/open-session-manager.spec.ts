import { expect, test } from "@playwright/test";

test("indexes fixtures, exports, soft-deletes, and shows risky masked config entries", async ({
  page
}) => {
  await page.goto("/");

  await expect(
    page.getByRole("heading", { name: /open session manager/i })
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: /refactor wsl collector handshake/i })
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
