import { expect } from "@open-wc/testing";

const loadSurveyTemplate = async () => {
  const response = await fetch("/ocg-server/templates/event/survey.html");

  expect(response.ok).to.equal(true);

  return response.text();
};

describe("post-event survey", () => {
  it("loads the generated stylesheet and keeps the responsive visual shell", async () => {
    const [template, fields] = await Promise.all([
      loadSurveyTemplate(),
      fetch("/ocg-server/templates/macros/question_answers.html").then((response) => response.text()),
    ]);

    expect(template).to.include('href="/static/css/styles.css"');
    expect(template).not.to.include('href="/static/css/app.css"');
    expect(template).to.include("public-site-theme");
    expect(template).to.include("rounded-[2rem]");
    expect(fields).to.include("sm:grid-cols-11");
    expect(template).to.include("question_answers::survey_fields");
    expect(template).to.include("data-survey-success");
    expect(template).to.include("data-survey-submitted");
    expect(template).to.include("Back to event");
  });

  it("serializes selected scale controls without changing the payload contract", async () => {
    document.body.innerHTML = `
      <form data-event-survey-form>
        <fieldset data-question-id="nps-question" data-question-kind="nps">
          <input type="radio" value="0" data-survey-answer>
          <input type="radio" value="9" data-survey-answer checked>
        </fieldset>
        <fieldset data-question-id="comment-question" data-question-kind="free-text">
          <textarea data-survey-answer>Useful event</textarea>
        </fieldset>
        <input name="survey_answers" data-survey-answers>
        <button type="submit" data-survey-submit>Submit feedback</button>
      </form>
    `;
    await import(`/static/js/event/survey.js?test=${Date.now()}`);

    const form = document.querySelector("[data-event-survey-form]");
    form.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));

    expect(JSON.parse(form.querySelector("[data-survey-answers]").value)).to.deep.equal({
      answers: [
        { question_id: "nps-question", value: 9 },
        { question_id: "comment-question", value: "Useful event" },
      ],
    });
    expect(form.getAttribute("aria-busy")).to.equal("true");
    expect(form.querySelector("[data-survey-submit]").textContent).to.equal("Submitting…");
  });
});
