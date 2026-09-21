import { expect } from "@open-wc/testing";

describe("custom domain dashboard template", () => {
  it("supports setup, DNS verification, activation status, and removal", async () => {
    const response = await fetch(
      "/ocg-server/templates/dashboard/group/custom_domain_card.html",
    );
    expect(response.ok).to.equal(true);

    const template = await response.text();
    expect(template).to.include('name="hostname"');
    expect(template).to.include("CNAME or ALIAS/ANAME record");
    expect(template).to.include("TXT record");
    expect(template).to.include('hx-post="{{ action_url }}/verify"');
    expect(template).to.include('hx-delete="{{ action_url }}"');
    expect(template).to.include("provision TLS and activate this hostname");
    expect(template).to.include('href="https://{{ domain.hostname }}/"');
  });

  it("loads custom-domain cards from group and event settings", async () => {
    const [groupResponse, eventResponse] = await Promise.all([
      fetch("/ocg-server/templates/dashboard/group/settings_update.html"),
      fetch("/ocg-server/templates/dashboard/group/events_update.html"),
    ]);
    expect(groupResponse.ok).to.equal(true);
    expect(eventResponse.ok).to.equal(true);

    expect(await groupResponse.text()).to.include(
      'hx-get="/dashboard/group/custom-domain"',
    );
    expect(await eventResponse.text()).to.include(
      'hx-get="/dashboard/group/events/{{ event.event_id }}/custom-domain"',
    );
  });

  it("does not make authenticated status requests from custom hosts", async () => {
    const [groupResponse, eventResponse] = await Promise.all([
      fetch("/ocg-server/templates/group/membership_button.html"),
      fetch("/ocg-server/templates/event/attend_button.html"),
    ]);
    const groupTemplate = await groupResponse.text();
    const eventTemplate = await eventResponse.text();

    expect(groupTemplate).to.include("{% if !custom_domain -%}");
    expect(groupTemplate).to.include('id="membership-checker"');
    expect(eventTemplate).to.include("{% if !custom_domain -%}");
    expect(eventTemplate).to.include(
      'data-attendance-role="attendance-checker"',
    );
  });
});
