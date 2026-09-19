import { expect, test } from "../../../fixtures.js";

import { navigateToPath } from "../../../utils.js";

test.describe("group dashboard GTM view", () => {
  test("loads the pipeline, creates a lead, and approves a draft", async ({
    organizerGroupPage,
  }) => {
    await navigateToPath(organizerGroupPage, "/dashboard/group?tab=gtm");

    const dashboardContent = organizerGroupPage.locator("#dashboard-content");
    await expect(dashboardContent.getByRole("heading", { name: "GTM" })).toBeVisible();

    const leadName = `E2E Sponsor Lead ${Date.now()}`;
    await dashboardContent.locator("#gtm-name").fill(leadName);
    await dashboardContent.locator("#gtm-kind").selectOption("sponsor");
    await dashboardContent.locator("#gtm-org").fill("E2E Corp");

    await Promise.all([
      organizerGroupPage.waitForResponse(
        (response) =>
          response.request().method() === "POST" &&
          response.url().includes("/dashboard/group/gtm/add") &&
          response.status() === 201,
      ),
      dashboardContent.getByRole("button", { name: "Create lead" }).click(),
    ]);

    const leadRow = dashboardContent.locator("tr", { hasText: leadName });
    await expect(leadRow).toBeVisible();
    await expect(leadRow).toContainText("Lead generation");

    await leadRow.getByRole("button", { name: "Open" }).click();
    await expect(
      dashboardContent.getByRole("heading", { name: leadName }),
    ).toBeVisible();

    await dashboardContent.locator("#gtm-agent").selectOption("reachout");
    await Promise.all([
      organizerGroupPage.waitForResponse(
        (response) =>
          response.request().method() === "POST" &&
          response.url().includes("/dashboard/group/gtm/") &&
          response.url().includes("/agents/run") &&
          response.status() === 201,
      ),
      dashboardContent.getByRole("button", { name: "Create draft" }).click(),
    ]);

    await navigateToPath(organizerGroupPage, "/dashboard/group?tab=gtm");
    await dashboardContent
      .locator("tr", { hasText: leadName })
      .getByRole("button", { name: "Open" })
      .click();
    await expect(dashboardContent.getByText("Pending reachout draft")).toBeVisible();

    await Promise.all([
      organizerGroupPage.waitForResponse(
        (response) =>
          response.request().method() === "POST" &&
          response.url().includes("/dashboard/group/gtm/drafts/") &&
          response.url().includes("/review") &&
          response.ok(),
      ),
      dashboardContent.getByRole("button", { name: "Approve" }).click(),
    ]);

    await navigateToPath(organizerGroupPage, "/dashboard/group?tab=gtm");
    await expect(
      dashboardContent.locator("tr", { hasText: leadName }),
    ).toContainText("Reachout");

    const leadRowAfterApprove = dashboardContent.locator("tr", { hasText: leadName });
    await leadRowAfterApprove.getByRole("button", { name: "Delete" }).click();
    await expect(organizerGroupPage.locator(".swal2-popup")).toBeVisible();

    await Promise.all([
      organizerGroupPage.waitForResponse(
        (response) =>
          response.request().method() === "DELETE" &&
          response.url().includes("/dashboard/group/gtm/") &&
          response.ok(),
      ),
      organizerGroupPage.getByRole("button", { name: "Delete" }).click(),
    ]);

    await expect(dashboardContent.locator("tr", { hasText: leadName })).toHaveCount(0);
  });
});
