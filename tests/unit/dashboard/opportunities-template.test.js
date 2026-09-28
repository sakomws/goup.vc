import { expect } from "@open-wc/testing";

const loadTemplate = async () => {
  const response = await fetch(
    "/ocg-server/templates/dashboard/opportunities.html",
  );
  expect(response.ok).to.equal(true);
  return response.text();
};

describe("opportunity dashboard template", () => {
  it("keeps preview, activation, publication, and deletion explicit", async () => {
    const template = await loadTemplate();

    expect(template).to.include("Previewing is required before activation");
    expect(template).to.include("/activate");
    expect(template).to.include("/publish");
    expect(template).to.include("/unpublish");
    expect(template).to.include('hx-confirm="Delete this opportunity?"');
    expect(template).to.include(
      "Jobs and CFS stay in their existing workflows",
    );
    expect(template).to.include("opportunity.opens_at");
  });
});
