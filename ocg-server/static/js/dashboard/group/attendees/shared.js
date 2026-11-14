import { closestElementWithinRoot, getElementById, isElementHidden } from "/static/js/common/dom.js";
import { isEscapeEvent } from "/static/js/common/keyboard.js";
import { toggleModalVisibility } from "/static/js/common/modals/modal-lifecycle.js";

export const attendeesRootSelector = "#attendees-content";

export const resolveAttendeesRoot = (root = document) => {
  if (root instanceof Element && root.matches(attendeesRootSelector)) {
    return root;
  }
  if (root instanceof Element) {
    return root.closest(attendeesRootSelector) || getElementById(root, "attendees-content") || root;
  }
  return getElementById(root, "attendees-content") || root.body || root;
};

export const setScopedModalVisibility = (root, targetModalId, visible, focusOrigin = null) => {
  const modal = getElementById(root, targetModalId);
  if (!modal) return;

  const isHidden = isElementHidden(modal);
  if ((visible && isHidden) || (!visible && !isHidden)) {
    toggleModalVisibility(targetModalId, focusOrigin);
  }
};

export const closeScopedModalFromEvent = (event, root, closeSelector, closeModal) => {
  if (!closestElementWithinRoot(event.target, closeSelector, root)) {
    return false;
  }
  event.stopPropagation();
  closeModal(root);
  return true;
};

export const bindScopedModalEscape = (root, closeModal) => {
  root.addEventListener("keydown", (event) => {
    if (isEscapeEvent(event)) {
      closeModal(root);
    }
  });
};

export const applyExternalRefundReviewCopy = (form, trigger, noteId, root, externalSuccessMessage = "") => {
  const externalNote = getElementById(root, noteId);
  const isExternal = trigger.dataset.refundExternal === "true";
  if (externalNote instanceof HTMLElement) {
    externalNote.hidden = !isExternal;
    externalNote.classList.toggle("hidden", !isExternal);
  }

  if (!("defaultSuccessMessage" in form.dataset)) {
    form.dataset.defaultSuccessMessage = form.dataset.successMessage || "";
  }
  form.dataset.successMessage =
    isExternal && externalSuccessMessage ? externalSuccessMessage : form.dataset.defaultSuccessMessage;
};
