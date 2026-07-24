import { expect } from "@open-wc/testing";

const loadTemplate = async () => {
  const response = await fetch("/ocg-server/templates/dashboard/jobs.html");

  expect(response.ok).to.equal(true);

  return response.text();
};

const normalizeWhitespace = (value) => value.replace(/\s+/g, " ").trim();

describe("jobs dashboard template", () => {
  it("refreshes the dashboard after a job is deleted", async () => {
    const template = normalizeWhitespace(await loadTemplate());

    expect(template).to.include('hx-delete="/dashboard/jobs/{{ job.job_id }}"');
    expect(template).to.include('hx-target="body"');
    expect(template).to.include("data-htmx-response");
    expect(template).to.include('data-success-message="Job deleted."');
    expect(template).to.include(
      'data-error-message="Could not delete this job. Please try again."',
    );
  });
});
