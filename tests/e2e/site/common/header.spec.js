import { expect, test } from "@playwright/test";

import { navigateToPath, navigateToSiteHome } from "../../utils.js";

test.describe("site header", () => {
  test("desktop navigation links point to the expected public pages", async ({
    page,
  }) => {
    // Load the public home page before checking desktop navigation links.
    await navigateToSiteHome(page);

    // Find the Main navigation control.
    const navigation = page.getByRole("navigation", {
      name: "Main navigation",
    });

    // Verify desktop navigation links point to the expected public pages.
    await expect(
      navigation.getByRole("link", { name: "Events & Groups" }),
    ).toHaveAttribute("href", /\/explore/);
    await expect(
      navigation.getByRole("link", { name: "Jobs" }),
    ).toHaveAttribute("href", "/jobs");
    await expect(
      navigation.getByRole("link", { name: "Stats" }),
    ).toHaveAttribute("href", "/stats");
    await expect(
      navigation.getByRole("link", { name: "Landscape" }),
    ).toHaveAttribute("href", "/landscape");
    await expect(
      navigation.getByRole("link", { name: "Resources" }),
    ).toHaveAttribute("href", "/wiki");
    await expect(
      navigation.getByRole("link", { name: "Join GOUP" }),
    ).toHaveAttribute("href", /\/log-in/);
    await expect(
      navigation.getByRole("button", { name: "Open search" }),
    ).toBeVisible();
  });

  test("guest call to action points to authentication", async ({ page }) => {
    // Load a public page before checking the guest call to action.
    await navigateToPath(page, "/explore?entity=events");

    const joinLink = page
      .getByRole("navigation", { name: "Main navigation" })
      .getByRole("link", { name: "Join GOUP" });
    await expect(joinLink).toBeVisible();
    await expect(joinLink).toHaveAttribute("href", /\/log-in/);
  });
});
