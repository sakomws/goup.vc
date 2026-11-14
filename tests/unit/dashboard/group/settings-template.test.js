import { expect } from "@open-wc/testing";

describe("dashboard group settings template", () => {
  it("renders persisted UTM sources, campaigns, and mediums", async () => {
    const response = await fetch(
      "/ocg-server/templates/dashboard/group/settings_update.html",
    );
    expect(response.ok).to.equal(true);

    const template = (await response.text()).replace(/\s+/g, " ");
    expect(template).to.include('name="utm_present" value="true"');
    expect(template).to.include(
      'field-name="utm_sources" input-type="text" items="{{ utm.sources|json }}"',
    );
    expect(template).to.include(
      'field-name="utm_campaigns" input-type="text" items="{{ utm.campaigns|json }}"',
    );
    expect(template).to.include(
      'field-name="utm_mediums" input-type="text" items="{{ utm.mediums|json }}"',
    );
    expect(template).to.include('excluded-values="{{ default_utm_mediums|json }}"');
  });
});
