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
    expect(element.querySelector('input[name="sponsors[0][level]"]').value).to.equal("Gold");
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
