import { expect, test } from "@playwright/test";

import { navigateToPath } from "../../utils.js";

test.describe("opportunity board", () => {
  test("renders unified filters and privacy-safe exports", async ({ page }) => {
    await navigateToPath(page, "/opportunities");

    await expect(page.getByRole("heading", { name: "Find ways to build together" })).toBeVisible();
    await expect(page.locator("#opportunity-query")).toBeVisible();
    await expect(page.locator("#opportunity-kind")).toBeVisible();
    await expect(page.getByRole("link", { name: "Export CSV" })).toHaveAttribute(
      "href",
      /\/opportunities\.csv/,
    );
  });
});
