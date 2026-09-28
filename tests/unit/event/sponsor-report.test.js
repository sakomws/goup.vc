import { expect } from "@open-wc/testing";

describe("event sponsor report template", () => {
  it("uses the generated stylesheet and responsive aggregate report components", async () => {
    const response = await fetch("/ocg-server/templates/event/sponsor_report.html");
    expect(response.ok).to.equal(true);
    const template = await response.text();

    expect(template).to.include('href="/static/css/styles.css"');
    expect(template).not.to.include('href="/static/css/app.css"');
    expect(template).to.include('content="noindex,nofollow"');
    expect(template).to.include("data-sponsor-report-page");
    expect(template).to.include("Sponsor performance");
    expect(template).to.include("Event outcomes");
    expect(template).to.include("Deliverable completion");
    expect(template).to.include("Performance by sponsor");
    expect(template).to.include("{% if !public -%}");
    expect(template).to.include("data-organizer-only-notes");
    expect(template).to.include("Shared reports exclude private organizer notes.");
  });
});
