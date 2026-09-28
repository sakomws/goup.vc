import { expect } from "@open-wc/testing";

describe("GTM dashboard templates", () => {
  it("gives confirmed delete actions stable element IDs", async () => {
    const [listResponse, detailResponse] = await Promise.all([
      fetch("/ocg-server/templates/dashboard/gtm/list.html"),
      fetch("/ocg-server/templates/dashboard/gtm/detail.html"),
    ]);

    expect(listResponse.ok).to.equal(true);
    expect(detailResponse.ok).to.equal(true);

    const expectedId = 'id="delete-gtm-lead-{{ lead.gtm_lead_id }}"';
    expect(await listResponse.text()).to.include(expectedId);
    expect(await detailResponse.text()).to.include(expectedId);
  });
});
