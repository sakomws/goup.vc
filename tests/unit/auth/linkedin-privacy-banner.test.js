import { expect } from "@open-wc/testing";

const loadTemplate = async (path) => {
  const response = await fetch(path);

  expect(response.ok).to.equal(true);

  return response.text();
};

const normalizeWhitespace = (value) => value.replace(/\s+/g, " ").trim();

const bannerCopy = [
  'id="linkedin-privacy-banner"',
  "Your LinkedIn profile stays private.",
  "GOUP does not take or use information from your LinkedIn profile.",
  "LinkedIn is only used so you can join or sign in.",
];

describe("linkedin privacy banner", () => {
  it("reassures first-time LinkedIn join and sign-in visitors", async () => {
    // Load the shared banner before checking the first-time LinkedIn copy.
    const template = normalizeWhitespace(
      await loadTemplate(
        "/ocg-server/templates/auth/linkedin_privacy_banner.html",
      ),
    );

    // Verify the banner tells visitors that LinkedIn profile data is not used.
    for (const snippet of bannerCopy) {
      expect(template).to.include(snippet);
    }
  });

  it("shows the banner on login and sign-up when LinkedIn is enabled", async () => {
    // Load both auth templates before checking banner placement.
    const logIn = await loadTemplate("/ocg-server/templates/auth/log_in.html");
    const signUp = await loadTemplate(
      "/ocg-server/templates/auth/sign_up.html",
    );

    // Verify first-time join and sign-in pages include the privacy banner.
    expect(logIn).to.include('{% if login.linkedin -%}');
    expect(logIn).to.include(
      '{% include "auth/linkedin_privacy_banner.html" -%}',
    );
    expect(signUp).to.include('{% if login.linkedin -%}');
    expect(signUp).to.include(
      '{% include "auth/linkedin_privacy_banner.html" -%}',
    );
    expect(logIn).not.to.include(
      "We use your verified email, name, and photo",
    );
    expect(signUp).not.to.include(
      "We use your verified email, name, and photo",
    );
  });
});
