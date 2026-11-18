import { expect } from "@open-wc/testing";

import "/static/js/common/cfs-label-selector.js";
import {
  mountLitComponent,
  useMountedElementsCleanup,
} from "/tests/unit/test-utils/lit.js";

describe("cfs-label-selector", () => {
  useMountedElementsCleanup("cfs-label-selector");

  it("normalizes labels and prunes invalid selections", async () => {
    // Render the cfs-label-selector fixture.
    const element = await mountLitComponent("cfs-label-selector", {
      labels: [
        { event_cfs_label_id: 1, name: "Backend", color: "blue" },
        { event_cfs_label_id: 1, name: "Duplicate", color: "red" },
        { event_cfs_label_id: 2, name: "Frontend", color: "green" },
        { event_cfs_label_id: "", name: "Ignored", color: "gray" },
      ],
      selected: ["1", "missing"],
    });

    // Let the component finish rendering.
    await element.updateComplete;

    // The selected event carries the expected payload.
    expect(element.labels).to.deep.equal([
      { event_cfs_label_id: "1", name: "Backend", color: "blue" },
      { event_cfs_label_id: "2", name: "Frontend", color: "green" },
    ]);
    expect(element.selected).to.deep.equal(["1"]);
  });

  it("toggles selections while respecting the maximum selection count", async () => {
    // Render the cfs-label-selector fixture.
    const element = await mountLitComponent("cfs-label-selector", {
      labels: [
        { event_cfs_label_id: "1", name: "Backend", color: "blue" },
        { event_cfs_label_id: "2", name: "Frontend", color: "green" },
      ],
      maxSelected: 1,
    });
    let changeEvents = 0;

    // Track emitted change events while toggling selected labels.
    element.addEventListener("change", () => {
      changeEvents += 1;
    });

    // Toggle labels through the maximum selection boundary.
    await element._toggleSelection("1");
    await element._toggleSelection("2");
    await element._toggleSelection("1");

    // The selected event carries the expected payload.
    expect(element.selected).to.deep.equal([]);
    expect(changeEvents).to.equal(2);
  });

  it("shows submission counts when requested", async () => {
    const element = await mountLitComponent("cfs-label-selector", {
      labels: [
        {
          event_cfs_label_id: "1",
          name: "track / Data and AI",
          color: "pink",
          submissions_count: 7,
        },
      ],
      showCounts: true,
    });

    element._combobox.open();
    await element.updateComplete;

    expect(element.textContent).to.contain("track / Data and AI");
    expect(
      element.querySelector('[title="7 submissions"]')?.textContent.trim(),
    ).to.equal("7");
  });

  it("maps labels to options and keeps the hidden inputs current on change", async () => {
    // Render the selector inside a form.
    document.body.innerHTML = '<form id="labels-form"></form>';
    const form = document.getElementById("labels-form");
    const element = document.createElement("cfs-label-selector");
    element.labels = [
      { event_cfs_label_id: "2", name: "Frontend", color: "green" },
      { event_cfs_label_id: "1", name: "Backend", color: "blue" },
    ];
    form.append(element);
    await element.updateComplete;
    const submittedValues = [];
    element.addEventListener("change", () => {
      submittedValues.push(new FormData(form).getAll("label_ids[]"));
    });

    // Open the dropdown and select a label.
    element.querySelector('input[type="search"]').dispatchEvent(new FocusEvent("focus"));
    await element.updateComplete;
    element.querySelector('[role="option"]').click();
    await element.updateComplete;

    // Verify the options, colored chip and hidden input.
    expect(element.options).to.deep.equal([
      { color: "green", name: "Frontend", value: "2" },
      { color: "blue", name: "Backend", value: "1" },
    ]);
    expect(submittedValues).to.deep.equal([["1"]]);
    expect(element.querySelector('[title="Backend"]').getAttribute("style")).to.include("--label-color:blue");
    expect(element.querySelector('[aria-label="Remove Backend"]')).to.not.equal(null);
  });

  it("keeps the legacy and generic placement markers", async () => {
    // Render the selector with labels.
    const element = await mountLitComponent("cfs-label-selector", {
      labels: [{ event_cfs_label_id: "1", name: "Backend", color: "blue" }],
    });

    // Open the labels dropdown.
    element._combobox.open();
    await element.updateComplete;

    // Verify both marker sets point at the same elements.
    const search = element.querySelector("[data-cfs-label-search]");
    const dropdown = element.querySelector("[data-cfs-label-dropdown]");
    expect(search).to.equal(element.querySelector("[data-multi-select-search]"));
    expect(dropdown).to.equal(element.querySelector("[data-multi-select-dropdown]"));
    expect(dropdown.getAttribute("role")).to.equal("listbox");
  });

  it("supports a mutually exclusive No label filter option with counts", async () => {
    document.body.innerHTML = '<form id="labels-filter-form"></form>';
    const form = document.getElementById("labels-filter-form");
    const element = document.createElement("cfs-label-selector");
    element.setAttribute("include-no-label", "");
    element.setAttribute("show-counts", "");
    element.setAttribute("no-label-count", "4");
    element.labels = [
      { event_cfs_label_id: "1", name: "Alpha", color: "blue", submissions_count: 2 },
      { event_cfs_label_id: "2", name: "Zeta", color: "green", submissions_count: 1 },
    ];
    form.append(element);
    await element.updateComplete;

    // Open dropdown and verify "No label" is pinned at the top with its count, while labels stays clean.
    element.querySelector('input[type="search"]').dispatchEvent(new FocusEvent("focus"));
    await element.updateComplete;
    const options = [...element.querySelectorAll('[role="option"]')];
    expect(options.map((opt) => opt.textContent.replace(/\s+/g, " ").trim())).to.deep.equal([
      "No label 4",
      "Alpha 2",
      "Zeta 1",
    ]);
    expect(element.labels).to.have.length(2);

    // Select a normal label first, then switch to "No label" and verify exclusivity.
    options[1].click();
    await element.updateComplete;
    expect(new FormData(form).getAll("label_ids[]")).to.deep.equal(["1"]);
    expect(new FormData(form).get("no_labels")).to.equal(null);

    options[0].click();
    await element.updateComplete;
    expect(new FormData(form).getAll("label_ids[]")).to.deep.equal([]);
    expect(new FormData(form).get("no_labels")).to.equal("true");
    expect(element.querySelector('[title="No label"]')).to.not.equal(null);

    // Switching back to a real label clears "No label".
    element.querySelectorAll('[role="option"]')[2].click();
    await element.updateComplete;
    expect(new FormData(form).getAll("label_ids[]")).to.deep.equal(["2"]);
    expect(new FormData(form).get("no_labels")).to.equal(null);
  });

  it("initializes from no-label-selected and clears on chip removal", async () => {
    document.body.innerHTML = `
      <form id="labels-init-form">
        <cfs-label-selector
          include-no-label
          no-label-selected
          no-label-count="2"
          labels='[{"event_cfs_label_id":"1","name":"Backend","color":"blue"}]'
        ></cfs-label-selector>
      </form>
    `;
    const form = document.getElementById("labels-init-form");
    const element = form.querySelector("cfs-label-selector");
    await element.updateComplete;

    expect(new FormData(form).get("no_labels")).to.equal("true");
    expect(element.querySelector('[aria-label="Remove No label"]')).to.not.equal(null);

    element.querySelector('[aria-label="Remove No label"]').click();
    await element.updateComplete;
    expect(new FormData(form).get("no_labels")).to.equal(null);
    expect(element.noLabelSelected).to.equal(false);
  });
});
