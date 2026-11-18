import { handleHtmxResponse } from "/static/js/common/alerts.js";
import {
  closestElement,
  getElementById,
  initializeOnReadyAndHtmxLoad,
  markDatasetReady,
} from "/static/js/common/dom.js";
import { parseJsonAttribute } from "/static/js/common/utils.js";
import "/static/js/dashboard/event/cfs/submissions.js";

const MODAL_ELEMENT_ID = "review-submission-modal";
const OPEN_ACTION = "open-cfs-submission-modal";
const DATA_KEY = "cfsSubmissionModalReady";
const SUBMISSIONS_CONTENT_ID = "submissions-content";
const SUBMISSIONS_FILTERS_FORM_ID = "submissions-filters-form";
const SUBMISSIONS_FILTER_ID = "submissions-label-filter";
const SUBMISSIONS_FILTERS_SORT_ID = "submissions-sort";
const SUBMISSIONS_SEARCH_ID = "submissions-search";
const SUBMISSIONS_SEARCH_CLEAR_ID = "submissions-search-clear";
const SUBMISSIONS_FILTERS_BOUND_KEY = "submissionsFiltersBound";
const SUBMISSION_EMAIL_MODAL_ID = "submission-email-modal";
const SUBMISSION_EMAIL_FORM_ID = "submission-email-form";
const SUBMISSION_EMAIL_AUDIENCE_ID = "submission-email-audience";
const SUBMISSION_EMAIL_SUMMARY_ID = "submission-email-summary";
const SUBMISSION_EMAIL_COPY = {
  all: "This email goes to speakers and co-speakers on every submission except withdrawn ones, when they accept organizer email.",
  approved: "This email goes to speakers and co-speakers on approved submissions who accept organizer email.",
  rejected: "This email goes to speakers and co-speakers on rejected submissions who accept organizer email.",
};
const PREP_EMAIL_SUCCESS = {
  listed: "Listed email sent.",
  profile: "Photo and bio email sent.",
  slides: "Slides email sent.",
  schedule: "Schedule email sent.",
};
const PREP_EMAIL_ERROR = "The speaker prep email could not be sent. Please try again.";
const SUBMISSION_EMAIL_SUCCESS = {
  all: "Email sent to speakers.",
  approved: "Email sent to approved speakers.",
  rejected: "Email sent to rejected speakers.",
};
const SUBMISSION_EMAIL_ERROR = "Something went wrong while trying to send the email. Please try again later.";

/**
 * Gets the CFS review modal custom element from the current document.
 * @returns {Element|null} Review modal element when present.
 */
const getReviewSubmissionModal = () => getElementById(document, MODAL_ELEMENT_ID);

/**
 * Opens the CFS review modal from a submission action button payload.
 * @param {HTMLElement} button - Button carrying serialized submission data.
 * @returns {void}
 */
const openSubmissionModal = (button) => {
  const payload = button.dataset.submission;
  if (!payload) {
    return;
  }
  const modal = getReviewSubmissionModal();
  if (!modal || typeof modal.open !== "function") {
    return;
  }
  const submission = parseJsonAttribute(payload, null);
  if (!submission || typeof submission !== "object" || Array.isArray(submission)) {
    console.error("Invalid submission payload");
    return;
  }
  const descriptionHtmlPayload = button.dataset.proposalDescriptionHtml;
  if (descriptionHtmlPayload && submission?.session_proposal) {
    const descriptionHtml = parseJsonAttribute(descriptionHtmlPayload, "");
    if (typeof descriptionHtml === "string") {
      submission.session_proposal.description_html = descriptionHtml;
    }
  }
  modal.open(submission);
};

/**
 * Returns the speaker email dialog.
 * @returns {HTMLElement|null} Dialog element when present.
 */
const getSubmissionEmailModal = () => getElementById(document, SUBMISSION_EMAIL_MODAL_ID);

/**
 * Shows or hides the speaker email dialog.
 * @param {boolean} open Whether the dialog should be visible.
 * @returns {void}
 */
const setSubmissionEmailModalOpen = (open) => {
  const modal = getSubmissionEmailModal();
  if (!(modal instanceof HTMLElement)) {
    return;
  }

  modal.classList.toggle("hidden", !open);
  modal.classList.toggle("flex", open);
  modal.setAttribute("aria-hidden", open ? "false" : "true");
  if (open) {
    modal.dataset.previousBodyOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return;
  }

  document.body.style.overflow = modal.dataset.previousBodyOverflow || "";
  delete modal.dataset.previousBodyOverflow;
};

/**
 * Opens the speaker email dialog for the selected audience.
 * @param {HTMLElement} button Button carrying the audience.
 * @returns {void}
 */
const openSubmissionEmailModal = (button) => {
  if (button.hasAttribute("disabled")) {
    return;
  }

  const audience = button.dataset.audience || "";
  const copy = SUBMISSION_EMAIL_COPY[audience];
  const audienceInput = getElementById(document, SUBMISSION_EMAIL_AUDIENCE_ID);
  const summary = getElementById(document, SUBMISSION_EMAIL_SUMMARY_ID);
  if (!copy || !(audienceInput instanceof HTMLInputElement) || !(summary instanceof HTMLElement)) {
    return;
  }

  audienceInput.value = audience;
  summary.textContent = copy;
  setSubmissionEmailModalOpen(true);
};

/**
 * Closes the speaker email dialog when it is open.
 * @returns {void}
 */
const closeSubmissionEmailModal = () => {
  const modal = getSubmissionEmailModal();
  if (!(modal instanceof HTMLElement) || modal.classList.contains("hidden")) {
    return;
  }

  setSubmissionEmailModalOpen(false);
};

/**
 * Binds document-level CFS review handlers once for swapped submission lists.
 * @returns {void}
 */
const bindCfsSubmissionGlobalHandlers = () => {
  if (!markDatasetReady(document.documentElement, DATA_KEY)) {
    return;
  }

  document.addEventListener("htmx:afterSwap", (event) => {
    const target = event?.detail?.target || event?.detail?.elt;
    if (!(target instanceof Element) || target.id !== SUBMISSIONS_CONTENT_ID) {
      return;
    }

    initializeSubmissionFilters(target);

    const modal = getReviewSubmissionModal();
    if (!modal || typeof modal.syncLabelsFromFilter !== "function") {
      return;
    }

    modal.syncLabelsFromFilter();
  });

  document.addEventListener("click", (event) => {
    const emailButton = closestElement(event.target, "[data-submission-email-open]");
    if (emailButton instanceof HTMLElement) {
      openSubmissionEmailModal(emailButton);
      return;
    }

    if (
      closestElement(
        event.target,
        "#close-submission-email-modal, #cancel-submission-email, #overlay-submission-email-modal",
      )
    ) {
      closeSubmissionEmailModal();
      return;
    }

    const button = closestElement(event.target, `[data-action="${OPEN_ACTION}"]`);
    if (button) {
      openSubmissionModal(button);
    }
  });

  document.addEventListener("htmx:afterRequest", (event) => {
    const prepButton = closestElement(event.target, "[data-prep-email]");
    if (prepButton instanceof HTMLElement) {
      const step = prepButton.dataset.step || "";
      handleHtmxResponse({
        errorMessage: event.detail?.xhr?.responseText || PREP_EMAIL_ERROR,
        successMessage: PREP_EMAIL_SUCCESS[step] || "Speaker prep email sent.",
        xhr: event.detail?.xhr,
      });
      return;
    }

    const form = event.target;
    if (!(form instanceof HTMLFormElement) || form.id !== SUBMISSION_EMAIL_FORM_ID) {
      return;
    }

    const audienceInput = form.elements.namedItem("audience");
    const audience = audienceInput instanceof HTMLInputElement ? audienceInput.value : "";
    const ok = handleHtmxResponse({
      errorMessage: event.detail?.xhr?.responseText || SUBMISSION_EMAIL_ERROR,
      successMessage: SUBMISSION_EMAIL_SUCCESS[audience] || SUBMISSION_EMAIL_SUCCESS.all,
      xhr: event.detail?.xhr,
    });
    if (!ok) {
      return;
    }

    form.reset();
    closeSubmissionEmailModal();
  });

  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape") {
      closeSubmissionEmailModal();
    }
  });
};

/**
 * Initializes auto-submit behavior for the submissions filters form.
 * @param {Document|Element} root - Root element to search from.
 * @returns {void}
 */
export const initializeSubmissionFilters = (root = document) => {
  const form = getElementById(root, SUBMISSIONS_FILTERS_FORM_ID);
  if (!markDatasetReady(form, SUBMISSIONS_FILTERS_BOUND_KEY)) {
    return;
  }

  const sort = getElementById(root, SUBMISSIONS_FILTERS_SORT_ID);
  const labelFilter = getElementById(root, SUBMISSIONS_FILTER_ID);
  const search = getElementById(root, SUBMISSIONS_SEARCH_ID);
  const clearSearch = getElementById(root, SUBMISSIONS_SEARCH_CLEAR_ID);
  const submitFilters = () => {
    window.requestAnimationFrame(() => form.requestSubmit());
  };

  sort?.addEventListener("change", submitFilters);
  labelFilter?.addEventListener("change", submitFilters);
  search?.addEventListener("change", submitFilters);
  clearSearch?.addEventListener("click", () => {
    if (search instanceof HTMLInputElement) {
      search.value = "";
    }
    submitFilters();
  });
};

/**
 * Initializes CFS submission filters and shared review modal handlers.
 * @param {Document|Element} root - Root element to search from.
 * @returns {void}
 */
const initializeCfsSubmissions = (root = document) => {
  bindCfsSubmissionGlobalHandlers();
  initializeSubmissionFilters(root);
};

initializeOnReadyAndHtmxLoad(initializeCfsSubmissions);
