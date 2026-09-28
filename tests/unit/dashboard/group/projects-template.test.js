import { expect } from "@open-wc/testing";

describe("collaboration project templates", () => {
  it("exposes organizer workflows without leaking member controls", async () => {
    const [dashboardResponse, publicResponse] = await Promise.all([
      fetch("/ocg-server/templates/dashboard/group/projects.html"),
      fetch("/ocg-server/templates/group/project.html"),
    ]);

    expect(dashboardResponse.ok).to.equal(true);
    expect(publicResponse.ok).to.equal(true);

    const dashboard = await dashboardResponse.text();
    const profile = await publicResponse.text();

    expect(dashboard).to.include('hx-post="/dashboard/group/projects"');
    expect(dashboard).to.include("/projects/office-hours/");
    expect(profile).not.to.include('hx-post="/projects/');
  });
});
