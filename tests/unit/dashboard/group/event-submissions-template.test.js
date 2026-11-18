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
    expect(template).to.include("data-submission-email-open");
    expect(template).to.include('data-audience="approved"');
    expect(template).to.include('data-audience="rejected"');
    expect(template).to.include('data-audience="all"');
    expect(template).to.include('name="ts_query"');
    expect(template).to.include('placeholder="Search speaker or proposal"');
    expect(template).to.include("include-no-label");
    expect(template).to.include('no-label-count="{{ unlabeled_submissions_count }}"');
    expect(template).to.include("no-label-selected");
    expect(template).to.include("Approval emailed");
    expect(template).to.include('data-step="listed"');
    expect(template).to.include('data-step="profile"');
    expect(template).to.include('data-step="slides"');
    expect(template).to.include('data-step="schedule"');
  });
});
