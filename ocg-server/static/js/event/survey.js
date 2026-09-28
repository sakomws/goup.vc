const form = document.querySelector("[data-event-survey-form]");

form?.addEventListener("submit", () => {
  const answers = [];
  for (const fieldset of form.querySelectorAll("[data-question-id]")) {
    const kind = fieldset.dataset.questionKind;
    const fields = [...fieldset.querySelectorAll("[data-survey-answer]")];
    let value;

    if (kind === "multi-select") {
      value = fields.filter((field) => field.checked).map((field) => field.value);
    } else if (kind === "single-select" || kind === "numeric-scale" || kind === "nps") {
      value = fields.find((field) => field.checked)?.value;
    } else {
      value = fields[0]?.value;
    }

    if ((kind === "numeric-scale" || kind === "nps") && value !== undefined) {
      value = Number.parseInt(value, 10);
    }

    if (value !== undefined && !(kind === "free-text" && value === "")) {
      answers.push({ question_id: fieldset.dataset.questionId, value });
    }
  }

  form.querySelector("[data-survey-answers]").value = JSON.stringify({ answers });

  const submitButton = form.querySelector("[data-survey-submit]");
  if (submitButton instanceof HTMLButtonElement) {
    submitButton.disabled = true;
    submitButton.textContent = "Submitting…";
    form.setAttribute("aria-busy", "true");
  }
});
