import { expect } from "@open-wc/testing";

const loadTemplate = async (path) => {
  const response = await fetch(path);

  expect(response.ok).to.equal(true);

  return response.text();
};

const normalizeWhitespace = (value) => value.replace(/\s+/g, " ").trim();

describe("discovery dashboard templates", () => {
  it("keeps job candidates visible when no job draft exists", async () => {
    const template = normalizeWhitespace(
      await loadTemplate("/ocg-server/templates/dashboard/jobs/discovery.html"),
    );

    expect(template).to.include("Discovery candidates awaiting review");
    expect(template).to.include("item.job_id.is_some()");
    expect(template).to.include(
      "No job draft was created, so this candidate cannot be published.",
    );
    expect(template).to.include("candidates discovered");
    expect(template).to.include("job drafts created");
  });

  it("keeps group candidates visible when no event draft exists", async () => {
    const template = normalizeWhitespace(
      await loadTemplate(
        "/ocg-server/templates/dashboard/group/integrations.html",
      ),
    );

    expect(template).to.include("Discovery candidates awaiting review");
    expect(template).to.include("item.event_id.is_some()");
    expect(template).to.include(
      "No event draft was created because this group has no default event template.",
    );
    expect(template).to.include("candidates discovered");
    expect(template).to.include("event drafts created");
  });
});
