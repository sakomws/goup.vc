import "/static/js/common/users/user-profile-modal-triggers.js";

let affiliationsExpandInitialized = false;

/**
 * Reveals the hidden affiliation chips of a card when its "+N more" button is
 * clicked, then hides the button itself.
 * @param {MouseEvent} event Click event.
 */
const handleAffiliationsMoreClick = (event) => {
  const button = event.target?.closest?.("[data-affiliations-more]");
  if (!button) {
    return;
  }

  const container = button.closest("[data-affiliations]");
  if (!container) {
    return;
  }

  container.querySelectorAll("[data-affiliation-extra]").forEach((chip) => {
    chip.hidden = false;
  });
  button.hidden = true;
};

/**
 * Binds the delegated listener that expands collapsed affiliation chips.
 * @returns {void}
 */
const initAffiliationsExpand = () => {
  if (affiliationsExpandInitialized) {
    return;
  }
  affiliationsExpandInitialized = true;

  document.addEventListener("click", handleAffiliationsMoreClick);
};

initAffiliationsExpand();
