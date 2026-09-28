const form = document.querySelector("[data-event-survey-form]");

form?.addEventListener("submit", () => {
  const answers = [];
  for (const fieldset of form.querySelectorAll("[data-question-id]")) {
    const kind = fieldset.dataset.questionKind;
    const fields = [...fieldset.querySelectorAll("[data-survey-answer]")];
    let value;

    if (kind === "multi-select") {
      value = fields.filter((field) => field.checked).map((field) => field.value);
    } else if (kind === "single-select") {
      value = fields.find((field) => field.checked)?.value;
    } else if (kind === "numeric-scale" || kind === "nps") {
      const raw = fields[0]?.value;
      value = raw === "" || raw === undefined ? undefined : Number.parseInt(raw, 10);
    } else {
      value = fields[0]?.value;
    }

    if (value !== undefined && !(kind === "free-text" && value === "")) {
      answers.push({ question_id: fieldset.dataset.questionId, value });
    }
  }

  form.querySelector("[data-survey-answers]").value = JSON.stringify({ answers });
});
