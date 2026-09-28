import { expect } from "@open-wc/testing";

const loadTemplate = async () => {
  const response = await fetch(
    "/ocg-server/templates/dashboard/group/events_update.html",
  );

  expect(response.ok).to.equal(true);

  return response.text();
};

const normalizeWhitespace = (value) => value.replace(/\s+/g, " ").trim();

describe("dashboard group event update template", () => {
  it("keeps the update event page at full dashboard content height", async () => {
    // Load the event update template before checking page root classes.
    const template = normalizeWhitespace(await loadTemplate());

    // Assert the update event page can fill the group dashboard content area.
    expect(template).to.include('id="event-update-page"');
    expect(template).to.include('id="event-update-page" class="space-y-12"');
    expect(template).to.include('data-event-page="update"');
    expect(template).to.include('<div id="event-preview-modal-root"></div>');
  });

  it("keeps the existing read-only copy when registration answers lock question editing", async () => {
    // Load the event update template before checking locked question copy.
    const template = normalizeWhitespace(await loadTemplate());

    // Assert the rendered registration question fields.
    expect(template).to.include(
      "{% if event.registration_questions_locked -%}",
    );
    expect(template).to.include(
      "Registration questions are read-only because attendees have submitted answers.",
    );
  });

  it("passes past-event state to online event and session details", async () => {
    // Load the event update template before checking the component contract.
    const template = normalizeWhitespace(await loadTemplate());

    // Assert the online and session details components receive past-event state.
    expect(template).to.include(
      "{% if event.is_past() %}event-past{% endif %}",
    );
  });

  it("shows compact event controls above update content", async () => {
    // Load the event update template before checking the top controls.
    const template = normalizeWhitespace(await loadTemplate());

    // Assert preview, defaults, and public-page controls precede the editor.
    expect(template).to.include('id="event-preview-button"');
    expect(template).to.include('id="set-event-defaults-button"');
    expect(template).to.include('id="event-public-page-link"');
    expect(template).to.include(
      'class="group btn-primary-outline btn-mini inline-flex h-7 min-w-28 items-center justify-center gap-2 whitespace-nowrap disabled:cursor-not-allowed disabled:opacity-50"',
    );
    expect(template.indexOf('id="event-preview-button"')).to.be.lessThan(
      template.indexOf('data-content="details"'),
    );
  });

  it("places the pending changes alert under the top controls", async () => {
    // Load the event update template before checking pending alert placement.
    const template = normalizeWhitespace(await loadTemplate());

    // Assert the save alert follows the title reminder and uses compact actions.
    const previewIndex = template.indexOf('id="event-preview-button"');
    const alertIndex = template.indexOf('id="pending-changes-alert"');

    expect(alertIndex).to.be.greaterThan(previewIndex);
    expect(template).to.include("icon-clock");
    expect(template).to.include('id="pending-changes-alert" class="hidden"');
    expect(template).to.include('class="text-sm/6"');
    expect(template).to.include("btn-primary w-24 text-nowrap ms-auto");
  });

  it("uses the styled confirmation dialog when moving an event", async () => {
    const template = normalizeWhitespace(await loadTemplate());

    expect(template).to.include('id="move-event-form"');
    expect(template).to.include('hx-trigger="confirmed"');
    expect(template).to.not.include("hx-confirm=");
  });

  it("lazy-loads event review tabs from the desktop tab buttons", async () => {
    // Load the event update template before checking lazy tab contracts.
    const template = normalizeWhitespace(await loadTemplate());

    // Assert review tabs fetch their table content only when selected.
    expect(template).to.not.include('id="update-event-section-select"');
    expect(template).to.include("event_form::tab_button");
    expect(template).to.include(
      'hx-get="/dashboard/group/events/{{ event.event_id }}/attendees" hx-trigger="click once" hx-target="#attendees-content"',
    );
    expect(template).to.include(
      'hx-get="/dashboard/group/events/{{ event.event_id }}/invitation-requests" hx-trigger="click once" hx-target="#invitation-requests-content"',
    );
    expect(template).to.include(
      'hx-get="/dashboard/group/events/{{ event.event_id }}/waitlist" hx-trigger="click once" hx-target="#waitlist-content"',
    );
  });

  it("keeps review tabs and bottom actions in the main grid column", async () => {
    // Load the event update template before checking grid placement classes.
    const template = normalizeWhitespace(await loadTemplate());

    // Assert root-level content after the form wrapper stays in the form column.
    expect(template).to.include('data-content="attendees" class="hidden"');
    expect(template).to.include(
      'data-content="invitation-requests" class="hidden"',
    );
    expect(template).to.include('data-content="waitlist" class="hidden"');
    expect(template).to.include(
      'class="flex flex-wrap items-center justify-end gap-3 mt-6"',
    );
  });

  it("keeps the event form navigation in the shared page scroll", async () => {
    // Load the event update template before checking sidebar scroll behavior.
    const template = normalizeWhitespace(await loadTemplate());

    // Assert the form navigation scrolls with the active event content.
    expect(template).to.not.include('class="sticky top-6"');
    expect(template).to.not.include('id="update-event-section-select"');
    expect(template).to.include('<div class="inert-form"');
    expect(template).to.include('<div data-content="details">');
    expect(template).to.not.include("lg:border-b-0");
  });
});
