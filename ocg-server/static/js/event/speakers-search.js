import { initializeOnReadyAndHtmxLoad, markDatasetReady } from "/static/js/common/dom.js";

const initializeSpeakersSearch = (root = document) => {
  const searchInput =
    root.querySelector?.("#event-speakers-search") || document.getElementById("event-speakers-search");
  if (!markDatasetReady(searchInput, "speakersSearchReady")) {
    return;
  }

  const updateResults = () => {
    const query = searchInput.value.trim().toLocaleLowerCase();
    const cards = [...document.querySelectorAll("[data-speaker-card]")];
    let visibleCount = 0;

    cards.forEach((card) => {
      const searchText = String(card.dataset.speakerSearch || "").toLocaleLowerCase();
      const matches = !query || searchText.includes(query);
      card.classList.toggle("hidden", !matches);
      if (matches) {
        visibleCount += 1;
      }
    });

    document.querySelectorAll("[data-speakers-empty]").forEach((emptyState) => {
      emptyState.classList.toggle("hidden", visibleCount > 0);
    });
  };

  searchInput.addEventListener("input", updateResults);
  updateResults();
};

initializeOnReadyAndHtmxLoad(initializeSpeakersSearch);
