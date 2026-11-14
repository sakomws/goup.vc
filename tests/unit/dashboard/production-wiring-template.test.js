import { expect } from "@open-wc/testing";

const load = async (path) => {
  const response = await fetch(path);
  expect(response.ok).to.equal(true);
  return response.text();
};

describe("dashboard production script wiring", () => {
  it("loads alliance contact behavior", async () => {
    const template = await load("/ocg-server/templates/dashboard/alliance/home.html");
    expect(template).to.include(
      'src="/static/js/dashboard/alliance/contact.js"',
    );
  });

  it("loads user notification preference behavior", async () => {
    const template = await load("/ocg-server/templates/dashboard/user/home.html");
    expect(template).to.include(
      'src="/static/js/dashboard/user/notification-preferences.js"',
    );
    expect(template).to.include(
      'src="/static/js/dashboard/user/notification-group-mutes.js"',
    );
  });
});
