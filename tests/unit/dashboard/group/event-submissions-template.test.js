import { expect } from "@open-wc/testing";

describe("dashboard event submissions template", () => {
  it("links to the event submissions CSV export", async () => {
    const response = await fetch(
      "/ocg-server/templates/dashboard/group/event_submissions_list.html",
    );
    expect(response.ok).to.equal(true);

    const template = (await response.text()).replace(/\s+/g, " ");
    expect(template).to.include(
      'href="/dashboard/group/events/{{ event_id }}/submissions.csv"',
    );
    expect(template).to.include("Export CSV");
  });
});
