import { expect } from "@open-wc/testing";

import { waitForMicrotask } from "/tests/unit/test-utils/async.js";
import { resetDom } from "/tests/unit/test-utils/dom.js";

describe("event speakers search", () => {
  afterEach(() => {
    resetDom();
  });

  it("filters speaker cards by name and shows an empty state", async () => {
    document.body.innerHTML = `
      <input id="event-speakers-search" type="search" />
      <div data-speaker-card data-speaker-search="maryam ahmadova">Maryam Ahmadova</div>
      <div data-speaker-card data-speaker-search="rashad amirjanov">Rashad Amirjanov</div>
      <p class="hidden" data-speakers-empty>No speakers match your search.</p>
    `;

    await import(`/static/js/event/speakers-search.js?test=${Date.now()}`);
    await waitForMicrotask();

    const input = document.getElementById("event-speakers-search");
    const cards = [...document.querySelectorAll("[data-speaker-card]")];
    const emptyState = document.querySelector("[data-speakers-empty]");

    input.value = "Maryam";
    input.dispatchEvent(new Event("input", { bubbles: true }));
    expect(cards[0].classList.contains("hidden")).to.equal(false);
    expect(cards[1].classList.contains("hidden")).to.equal(true);
    expect(emptyState.classList.contains("hidden")).to.equal(true);

    input.value = "nobody";
    input.dispatchEvent(new Event("input", { bubbles: true }));
    expect(cards.every((card) => card.classList.contains("hidden"))).to.equal(
      true,
    );
    expect(emptyState.classList.contains("hidden")).to.equal(false);
  });

  it("filters speaker cards and talk groups by track", async () => {
    document.body.innerHTML = `
      <select id="event-speakers-track">
        <option value="">All tracks</option>
        <option value="AI">AI</option>
        <option value="Security">Security</option>
      </select>
      <input id="event-speakers-search" type="search" />
      <div data-track-group="AI">AI talks</div>
      <div data-track-group="Security">Security talks</div>
      <div data-speaker-card data-speaker-search="maryam ahmadova" data-speaker-tracks="AI">Maryam Ahmadova</div>
      <div data-speaker-card data-speaker-search="rashad amirjanov" data-speaker-tracks="${["Security", "AI"].join("\u001f")}">Rashad Amirjanov</div>
      <div data-speaker-card data-speaker-search="legacy host">Legacy Host</div>
      <p class="hidden" data-speakers-empty>No speakers match.</p>
    `;

    await import(`/static/js/event/speakers-search.js?test=${Date.now()}`);
    await waitForMicrotask();

    const trackSelect = document.getElementById("event-speakers-track");
    const searchInput = document.getElementById("event-speakers-search");
    const cards = [...document.querySelectorAll("[data-speaker-card]")];
    const groups = [...document.querySelectorAll("[data-track-group]")];
    const emptyState = document.querySelector("[data-speakers-empty]");

    trackSelect.value = "AI";
    trackSelect.dispatchEvent(new Event("change", { bubbles: true }));
    expect(cards[0].classList.contains("hidden")).to.equal(false);
    expect(cards[1].classList.contains("hidden")).to.equal(false);
    expect(cards[2].classList.contains("hidden")).to.equal(true);
    expect(groups[0].classList.contains("hidden")).to.equal(false);
    expect(groups[1].classList.contains("hidden")).to.equal(true);
    expect(emptyState.classList.contains("hidden")).to.equal(true);

    searchInput.value = "maryam";
    searchInput.dispatchEvent(new Event("input", { bubbles: true }));
    expect(cards[0].classList.contains("hidden")).to.equal(false);
    expect(cards[1].classList.contains("hidden")).to.equal(true);

    trackSelect.value = "Security";
    trackSelect.dispatchEvent(new Event("change", { bubbles: true }));
    expect(cards.every((card) => card.classList.contains("hidden"))).to.equal(true);
    expect(emptyState.classList.contains("hidden")).to.equal(false);

    searchInput.value = "";
    trackSelect.value = "";
    trackSelect.dispatchEvent(new Event("change", { bubbles: true }));
    expect(cards.some((card) => card.classList.contains("hidden"))).to.equal(false);
    expect(groups.some((group) => group.classList.contains("hidden"))).to.equal(false);
  });
});
