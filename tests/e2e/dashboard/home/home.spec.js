import { expect, test } from "../../fixtures.js";

import { navigateToPath } from "../../utils.js";

const DASHBOARD_ROUTES = ["/dashboard/alliance", "/dashboard/group", "/dashboard/user"];

test.describe("dashboard home", () => {
  for (const route of DASHBOARD_ROUTES) {
    test(`requires login for ${route}`, async ({ page }) => {
      // Open the protected dashboard route as a guest.
      await navigateToPath(page, route);

      // Verify requires login for route.
      await expect(page).toHaveURL(/\/log-in/);
      await expect(page.getByRole("heading", { name: "Welcome back." })).toBeVisible();
    });
  }

  test.describe("mobile experience @mobile", () => {
    test("alliance dashboard opens on mobile", async ({ adminAlliancePage }) => {
      // Load the alliance dashboard on a mobile viewport.
      await navigateToPath(adminAlliancePage, "/dashboard/alliance?tab=groups");

      // Verify the dashboard content is available and the menu can be opened.
      await expect(adminAlliancePage.locator("#dashboard-main-content")).toBeVisible();
      await expect(adminAlliancePage.locator("#open-dashboard-menu")).toBeVisible();
    });

    test("group dashboard opens on mobile", async ({ organizerGroupPage }) => {
      // Load the group dashboard on a mobile viewport.
      await navigateToPath(organizerGroupPage, "/dashboard/group?tab=events");

      // Verify the dashboard content is available and the menu can be opened.
      await expect(organizerGroupPage.locator("#dashboard-main-content")).toBeVisible();
      await expect(organizerGroupPage.locator("#open-dashboard-menu")).toBeVisible();
    });

    test("user dashboard opens on mobile", async ({ member1Page }) => {
      // Load the user dashboard on a mobile viewport.
      await navigateToPath(member1Page, "/dashboard/user?tab=events");

      // Verify the dashboard content is available and the menu can be opened.
      await expect(member1Page.locator("#dashboard-main-content")).toBeVisible();
      await expect(member1Page.locator("#open-dashboard-menu")).toBeVisible();
    });
  });
});
