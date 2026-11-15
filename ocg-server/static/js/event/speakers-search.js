import { initializeOnReadyAndHtmxLoad, markDatasetReady } from "/static/js/common/dom.js";

const TRACK_SEPARATOR = "\u001f";

const tracksFor = (card) => {
  const raw = card.getAttribute("data-speaker-tracks") || "";
  return raw ? raw.split(TRACK_SEPARATOR) : [];
};

const initializeSpeakersSearch = (root = document) => {
  const searchInput =
    root.querySelector?.("#event-speakers-search") || document.getElementById("event-speakers-search");
  const trackSelect =
    root.querySelector?.("#event-speakers-track") || document.getElementById("event-speakers-track");
  if (!markDatasetReady(searchInput || trackSelect, "speakersSearchReady")) {
    return;
  }

  const updateResults = () => {
    const query = searchInput?.value.trim().toLocaleLowerCase() || "";
    const track = trackSelect?.value || "";
    const cards = [...document.querySelectorAll("[data-speaker-card]")];
    let visibleCount = 0;

    cards.forEach((card) => {
      const searchText = String(card.dataset.speakerSearch || "").toLocaleLowerCase();
      const matchesQuery = !query || searchText.includes(query);
      const matchesTrack = !track || tracksFor(card).includes(track);
      const matches = matchesQuery && matchesTrack;
      card.classList.toggle("hidden", !matches);
      if (matches) {
        visibleCount += 1;
      }
    });

    document.querySelectorAll("[data-track-group]").forEach((group) => {
      const name = group.getAttribute("data-track-group") || "";
      group.classList.toggle("hidden", Boolean(track) && name !== track);
    });

    document.querySelectorAll("[data-speakers-empty]").forEach((emptyState) => {
      emptyState.classList.toggle("hidden", visibleCount > 0);
    });
  };

  searchInput?.addEventListener("input", updateResults);
  trackSelect?.addEventListener("change", updateResults);
  updateResults();
};

initializeOnReadyAndHtmxLoad(initializeSpeakersSearch);
