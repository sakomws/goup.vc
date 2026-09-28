import { expect } from "@open-wc/testing";

const loadTemplate = async () => {
  const response = await fetch(
    "/ocg-server/templates/dashboard/user/invitations_list.html",
  );
  expect(response.ok).to.equal(true);
  return response.text();
};

const normalizeWhitespace = (value) => value.replace(/\s+/g, " ").trim();

describe("dashboard user invitations template", () => {
  it("renders cross-group co-host details and personal actions", async () => {
    const template = normalizeWhitespace(await loadTemplate());

    expect(template).to.include('title = "Co-host Invitations"');
    expect(template).to.include("invitation.primary_group_name");
    expect(template).to.include("invitation.cohost_group_name");
    expect(template).to.include("invitation.message");
    expect(template).to.include(
      'hx-put="/dashboard/user/invitations/cohost/{{ invitation.event_cohost_id }}/accept"',
    );
    expect(template).to.include(
      'hx-put="/dashboard/user/invitations/cohost/{{ invitation.event_cohost_id }}/reject"',
    );
    expect(template).to.include('hx-trigger="confirmed"');
  });
});
