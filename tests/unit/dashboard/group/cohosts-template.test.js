import { expect } from "@open-wc/testing";

const loadTemplate = async (name) => {
  const response = await fetch(
    `/ocg-server/templates/dashboard/group/${name}.html`,
  );

  expect(response.ok).to.equal(true);
  return response.text();
};

const normalizeWhitespace = (value) => value.replace(/\s+/g, " ").trim();

describe("dashboard group co-host templates", () => {
  it("lets an event owner invite and remove co-host groups", async () => {
    const template = normalizeWhitespace(
      await loadTemplate("event_cohosts"),
    );

    expect(template).to.include(
      'hx-post="/dashboard/group/events/{{ event_id }}/cohosts"',
    );
    expect(template).to.include('name="cohost_group_id"');
    expect(template).to.include(
      'hx-put="/dashboard/group/events/{{ event_id }}/cohosts/{{ request.event_cohost_id }}/revoke"',
    );
  });

  it("lets an invited group accept or reject a request", async () => {
    const template = normalizeWhitespace(await loadTemplate("cohosts_list"));

    expect(template).to.include(
      'hx-put="/dashboard/group/cohosts/{{ invitation.event_cohost_id }}/accept"',
    );
    expect(template).to.include(
      'hx-put="/dashboard/group/cohosts/{{ invitation.event_cohost_id }}/reject"',
    );
    expect(template).to.include("{{ invitation.event_url() }}");
  });
});
