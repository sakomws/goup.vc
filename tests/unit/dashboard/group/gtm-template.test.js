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

  it("renders lead-generation drafts with accessible review controls", async () => {
    const response = await fetch("/ocg-server/templates/dashboard/gtm/list.html");
    const template = await response.text();

    expect(response.ok).to.equal(true);
    expect(template).to.include('id="lead-gen-status" aria-live="polite"');
    expect(template).to.include("Lead suggestions awaiting review");
    expect(template).to.include(
      'hx-post="{{ dashboard_base }}/gtm/drafts/{{ review.draft.gtm_agent_draft_id }}/review"',
    );
    expect(template).to.include('name="status" value="approved"');
    expect(template).to.include('name="status" value="rejected"');
    expect(template).to.include('hx-disabled-elt="this"');
    expect(template).to.include("Draft awaiting review");
  });
});
