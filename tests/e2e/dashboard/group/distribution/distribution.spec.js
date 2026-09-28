import { expect, test } from "../../../fixtures.js";
import { navigateToPath } from "../../../utils.js";

test.describe("group distribution dashboard", () => {
  test("organizer sees planning-only distribution controls", async ({
    organizerGroupPage,
  }) => {
    await navigateToPath(
      organizerGroupPage,
      "/dashboard/group?tab=distribution",
    );

    await expect(
      organizerGroupPage.getByRole("heading", { name: "Campaign metrics" }),
    ).toBeVisible();
    await expect(
      organizerGroupPage.getByRole("heading", {
        name: "LinkedIn / X / Instagram calendar",
      }),
    ).toBeVisible();
    await expect(
      organizerGroupPage.getByRole("button", { name: "Add campaign" }),
    ).toBeEnabled();
    await expect(
      organizerGroupPage.getByRole("button", { name: "Mark posted" }),
    ).toHaveCount(0);
  });
});
