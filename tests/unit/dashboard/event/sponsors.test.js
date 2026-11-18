import { expect } from "@open-wc/testing";

import "/static/js/dashboard/event/sponsors.js";
import { mountLitComponent, useMountedElementsCleanup } from "/tests/unit/test-utils/lit.js";

describe("sponsors-section", () => {
  const sponsors = [
    {
      group_sponsor_id: "sponsor-1",
      name: "Acme Cloud",
      logo_url: "https://example.com/acme.png",
    },
    {
      group_sponsor_id: "sponsor-2",
      name: "Beta Compute",
      logo_url: "https://example.com/beta.png",
    },
  ];

  useMountedElementsCleanup("sponsors-section");

  it("keeps a selected sponsor and its remove control inside the section width", async () => {
    const element = await mountLitComponent("sponsors-section", {
      sponsors,
      selectedSponsors: ["sponsor-1"],
    });

    expect(element.classList.contains("block")).to.equal(true);
    expect(element.classList.contains("max-w-full")).to.equal(true);
    const card = element.querySelector('[aria-label="Remove Acme Cloud"]')?.parentElement;
    expect(card?.className).to.include("flex");
    expect(card?.className).to.include("max-w-full");
    expect(element.querySelector('[aria-label="Remove Acme Cloud"]')?.className).to.include("shrink-0");
  });

  it("normalizes selected sponsor ids into sponsor objects", async () => {
    // Call mount lit component.
    const element = await mountLitComponent("sponsors-section", {
      sponsors,
      selectedSponsors: ["sponsor-2"],
    });

    // Verify normalizes selected sponsor ids into sponsor objects.
    expect(element.selectedSponsors).to.deep.equal([
      {
        group_sponsor_id: "sponsor-2",
        name: "Beta Compute",
        logo_url: "https://example.com/beta.png",
      },
    ]);
  });

  it("filters sponsors and opens the level modal from keyboard selection", async () => {
    // Call mount lit component.
    const element = await mountLitComponent("sponsors-section", {
      sponsors,
    });

    // Call on input change.
    element._onInputChange({
      target: {
        value: "beta",
      },
    });
    await element.updateComplete;

    // Verify filters sponsors and opens the level modal from keyboard selection.
    expect(element.visibleDropdown).to.equal(true);
    expect(element.visibleOptions.map((item) => item.name)).to.deep.equal(["Beta Compute"]);
    expect(element.activeIndex).to.equal(0);

    // Call handle key down.
    element._handleKeydown({
      key: "Enter",
      preventDefault() {},
    });
    await element.updateComplete;

    // Verify filters sponsors and opens the level modal from keyboard selection.
    expect(element.showLevelModal).to.equal(true);
    expect(element.pendingSponsor).to.deep.equal({
      group_sponsor_id: "sponsor-2",
      name: "Beta Compute",
      logo_url: "https://example.com/beta.png",
    });
  });

  it("adds the pending sponsor with its level and renders hidden inputs", async () => {
    // Call mount lit component.
    const element = await mountLitComponent("sponsors-section", {
      sponsors,
    });

    // Call on select.
    element._onSelect(sponsors[0]);
    element.pendingLevel = "Gold";

    // Confirm the pending sponsor level.
    element._confirmAddSponsorLevel();
    await element.updateComplete;

    // Verify adds the pending sponsor with its level and renders hidden inputs.
    expect(element.selectedSponsors).to.deep.equal([
      {
        group_sponsor_id: "sponsor-1",
        name: "Acme Cloud",
        logo_url: "https://example.com/acme.png",
        level: "Gold",
      },
    ]);
    expect(element.showLevelModal).to.equal(false);
    expect(element.querySelector('input[name="sponsors[0][group_sponsor_id]"]').value).to.equal("sponsor-1");
    expect(element.querySelector('input[name="sponsors[0][name]"]').value).to.equal("Acme Cloud");
    expect(element.querySelector('input[name="sponsors[0][logo_url]"]').value).to.equal(
      "https://example.com/acme.png",
    );
    expect(element.querySelector('input[name="sponsors[0][level]"]').value).to.equal("Gold");
  });

  it("opens the add-sponsor dialog on the document body", async () => {
    const element = await mountLitComponent("sponsors-section", { sponsors });

    element.querySelector("button").click();
    await element.updateComplete;

    const modal = document.querySelector("[data-event-sponsor-modal='create']");
    expect(modal).to.exist;
    expect(element.contains(modal)).to.equal(false);
    const logoField = modal.querySelector("image-field");
    expect(logoField.getAttribute("image-kind")).to.equal("logo");
    expect(logoField.getAttribute("target")).to.equal("logo");

    logoField.dispatchEvent(
      new CustomEvent("image-change", {
        bubbles: true,
        detail: { value: "https://example.com/logo.png" },
      }),
    );
    await element.updateComplete;
    expect(element.newSponsorLogoUrl).to.equal("https://example.com/logo.png");
  });

  it("keeps the add-sponsor dialog open when the opening click hits the backdrop", async () => {
    const element = await mountLitComponent("sponsors-section", { sponsors });

    element.querySelector("button").click();
    await element.updateComplete;

    const modal = document.querySelector("[data-event-sponsor-modal='create']");
    const backdrop = modal.querySelector("[data-event-sponsor-backdrop]");
    const panel = backdrop.nextElementSibling;
    expect(panel.className).to.include("max-h-[calc(100dvh-2rem)]");
    expect(panel.className).to.include("overflow-y-auto");
    expect(panel.className).to.include("z-10");

    backdrop.click();
    await element.updateComplete;
    expect(document.querySelector("[data-event-sponsor-modal='create']")).to.exist;

    await new Promise((resolve) => {
      setTimeout(resolve, 0);
    });
    backdrop.click();
    await element.updateComplete;
    expect(element.showCreateModal).to.equal(false);
    expect(document.querySelector("[data-event-sponsor-modal='create']")).to.equal(null);
  });

  it("adds an event-only sponsor with inline hidden fields", async () => {
    const element = await mountLitComponent("sponsors-section", { sponsors });

    element._openCreateModal();
    element.newSponsorName = "Event Partner";
    element.newSponsorLogoUrl = "https://example.com/event-partner.png";
    element.newSponsorWebsiteUrl = "https://example.com/event-partner";
    element.newSponsorLevel = "Community";
    element._confirmCreateSponsor();
    await element.updateComplete;

    expect(element.selectedSponsors).to.deep.equal([
      {
        client_id: "event-sponsor-1",
        name: "Event Partner",
        logo_url: "https://example.com/event-partner.png",
        website_url: "https://example.com/event-partner",
        level: "Community",
      },
    ]);
    expect(element.querySelector('input[name="sponsors[0][group_sponsor_id]"]')).to.equal(null);
    expect(element.querySelector('input[name="sponsors[0][name]"]').value).to.equal("Event Partner");
    expect(element.querySelector('input[name="sponsors[0][logo_url]"]').value).to.equal(
      "https://example.com/event-partner.png",
    );
    expect(element.querySelector('input[name="sponsors[0][website_url]"]').value).to.equal(
      "https://example.com/event-partner",
    );
    expect(element.querySelector('input[name="sponsors[0][level]"]').value).to.equal("Community");
  });

  it("edits a sponsor name, logo, website, and level in place", async () => {
    const element = await mountLitComponent("sponsors-section", { sponsors });

    element._onSelect(sponsors[0]);
    element.pendingLevel = "Gold";
    element._confirmAddSponsorLevel();
    await element.updateComplete;

    element.querySelector('[aria-label="Edit Acme Cloud"]').click();
    await element.updateComplete;

    const modal = document.querySelector("[data-event-sponsor-modal='create']");
    expect(modal.textContent).to.include("Edit sponsor");
    expect(modal.querySelector("#event-sponsor-name").value).to.equal("Acme Cloud");
    expect(modal.querySelector("#event-sponsor-level").value).to.equal("Gold");

    element.newSponsorName = "Acme Platform";
    element.newSponsorLogoUrl = "https://example.com/acme-new.png";
    element.newSponsorWebsiteUrl = "https://example.com/acme";
    element.newSponsorLevel = "Platinum";
    element._confirmCreateSponsor();
    await element.updateComplete;

    expect(element.selectedSponsors).to.deep.equal([
      {
        group_sponsor_id: "sponsor-1",
        name: "Acme Platform",
        logo_url: "https://example.com/acme-new.png",
        website_url: "https://example.com/acme",
        level: "Platinum",
      },
    ]);
    expect(element.querySelector('input[name="sponsors[0][group_sponsor_id]"]').value).to.equal("sponsor-1");
    expect(element.querySelector('input[name="sponsors[0][name]"]').value).to.equal("Acme Platform");
    expect(element.querySelector('input[name="sponsors[0][logo_url]"]').value).to.equal(
      "https://example.com/acme-new.png",
    );
    expect(element.querySelector('input[name="sponsors[0][website_url]"]').value).to.equal(
      "https://example.com/acme",
    );
    expect(element.querySelector('input[name="sponsors[0][level]"]').value).to.equal("Platinum");
    expect(element.showCreateModal).to.equal(false);
  });

  it("blocks event submission when a selected sponsor is missing a level", async () => {
    // Prepare a selected sponsor without a level before submitting.
    const addEventButton = document.createElement("button");
    addEventButton.id = "add-event-button";
    document.body.append(addEventButton);

    // Call mount lit component.
    const element = await mountLitComponent("sponsors-section", {
      sponsors,
      selectedSponsors: [
        {
          group_sponsor_id: "sponsor-1",
          name: "Acme Cloud",
          logo_url: "https://example.com/acme.png",
          level: "",
        },
      ],
    });

    // Prepare submit event for blocking event submission when a selected sponsor.
    const submitEvent = new MouseEvent("click", {
      bubbles: true,
      cancelable: true,
    });
    const dispatchResult = addEventButton.dispatchEvent(submitEvent);
    await element.updateComplete;

    // Submission is blocked when a selected sponsor is missing a level.
    expect(dispatchResult).to.equal(false);
    expect(element.showLevelModal).to.equal(true);
    expect(element.pendingSponsor.group_sponsor_id).to.equal("sponsor-1");
    expect(element.pendingLevel).to.equal("");
  });
});
