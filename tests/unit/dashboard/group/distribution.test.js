import { expect } from "@open-wc/testing";
import { platformUrls } from "../../../../ocg-server/static/js/dashboard/group/distribution.js";

describe("distribution manual publishing", () => {
  it("uses public compose pages without OAuth APIs", () => {
    expect(platformUrls.linkedin).to.match(/^https:\/\/www\.linkedin\.com\//);
    expect(platformUrls.x).to.equal("https://x.com/compose/post");
    expect(platformUrls.instagram).to.equal("https://www.instagram.com/");
  });
});
