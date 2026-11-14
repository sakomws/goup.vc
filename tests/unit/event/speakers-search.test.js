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
});
