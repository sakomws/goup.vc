import { expect, test } from "../../../fixtures.js";

import { navigateToPath } from "../../../utils.js";

test.describe("group collaboration projects", () => {
  test("organizer can create a group-owned project", async ({
    organizerGroupPage,
  }) => {
    await navigateToPath(organizerGroupPage, "/dashboard/group?tab=projects");

    const dashboard = organizerGroupPage.locator("#dashboard-content");
    await expect(
      dashboard.getByRole("heading", { name: "Collaboration projects" }),
    ).toBeVisible();

    const suffix = Date.now();
    await dashboard.getByLabel("Name").fill(`E2E Collaboration ${suffix}`);
    await dashboard.getByLabel("Slug").fill(`e2e-collaboration-${suffix}`);
    await dashboard
      .getByLabel("Summary")
      .fill("A project created by the collaboration workspace E2E test.");
    await dashboard.getByLabel("Lifecycle").selectOption("active");
    await dashboard.getByLabel("Visibility").selectOption("public");

    await Promise.all([
      organizerGroupPage.waitForResponse(
        (response) =>
          response.request().method() === "POST" &&
          response.url().endsWith("/dashboard/group/projects") &&
          response.ok(),
      ),
      dashboard.getByRole("button", { name: "Create project" }).click(),
    ]);

    await expect(
      dashboard.getByText(`E2E Collaboration ${suffix}`),
    ).toBeVisible();
  });
});
