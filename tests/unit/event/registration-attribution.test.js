import { expect } from "@open-wc/testing";
import { registrationAttribution } from "/static/js/event/registration-attribution.js";

describe("event registration attribution", () => {
  it("captures campaign, referral, and referrer fields", () => {
    const result = registrationAttribution(
      "https://example.test/event?source=newsletter&ref=AMB42&utm_medium=email&utm_campaign=launch&utm_content=hero&utm_term=community",
      "https://partner.test/post",
    );

    expect(result).to.deep.equal({
      source: "newsletter",
      referral_code: "AMB42",
      referrer: "https://partner.test/post",
      utm_medium: "email",
      utm_campaign: "launch",
      utm_content: "hero",
      utm_term: "community",
    });
  });

  it("uses utm source and direct defaults", () => {
    expect(registrationAttribution("https://example.test/event?utm_source=linkedin").source).to.equal(
      "linkedin",
    );
    expect(registrationAttribution("https://example.test/event").source).to.equal("direct");
  });
});
