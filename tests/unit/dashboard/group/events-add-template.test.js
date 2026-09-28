import { expect } from "@open-wc/testing";

const loadTemplate = async () => {
  const response = await fetch(
    "/ocg-server/templates/dashboard/group/events_add.html",
  );

  expect(response.ok).to.equal(true);

  return response.text();
};

const normalizeWhitespace = (value) => value.replace(/\s+/g, " ").trim();

describe("dashboard group event add template", () => {
  it("keeps the add event page at full dashboard content height", async () => {
    // Load the event add template before checking page root classes.
    const template = normalizeWhitespace(await loadTemplate());

    // Assert the add event page can fill the group dashboard content area.
    expect(template).to.include('class="space-y-12" data-event-page="add"');
    expect(template).to.include('data-event-page="add"');
    expect(template).to.include(
      '<div id="event-preview-modal-root" class="contents"></div>',
    );
    expect(template).to.include('class="col-span-full min-w-0 space-y-3"');
    expect(template).to.include('class="block min-w-0 max-w-full"');
    expect(template).to.include('class="form-legend mt-3 break-words"');
  });

  it("keeps each event editor section unique and relevant", async () => {
    // Load the event add template before checking section content.
    const template = normalizeWhitespace(await loadTemplate());

    // Assert each tab has exactly one matching content panel.
    const count = (value) => template.split(value).length - 1;
    expect(count('data-content="details"')).to.equal(1);
    expect(count('data-content="date-venue"')).to.equal(1);
    expect(count('data-content="questions"')).to.equal(1);
    expect(count('data-content="cfs"')).to.equal(1);
    expect(count('data-content="sessions"')).to.equal(1);
    expect(count('data-content="hosts-sponsors"')).to.equal(1);

    // Assert the core event identity fields are in Details.
    const detailsFormIndex = template.indexOf('<form id="details-form">');
    const eventNameIndex = template.indexOf('name="name"');
    const dateVenueFormIndex = template.indexOf('<form id="date-venue-form">');
    const startsAtIndex = template.indexOf('name="starts_at"');

    expect(detailsFormIndex).to.be.greaterThan(-1);
    expect(eventNameIndex).to.be.greaterThan(detailsFormIndex);
    expect(eventNameIndex).to.be.lessThan(dateVenueFormIndex);

    // Assert scheduling fields belong to Date & Venue.
    expect(startsAtIndex).to.be.greaterThan(dateVenueFormIndex);
  });

  it("keeps additional information inside the Details panel", async () => {
    const template = normalizeWhitespace(await loadTemplate());

    expect(template).to.include(
      "{# End Event Description -#} </div> </div> {# Additional Information section -#}",
    );
    expect(template).to.not.include(
      "{# End Event Description -#} </div> </div> </div> {# Additional Information section -#}",
    );
  });

  it("shows the event tabs and preview control above the editor", async () => {
    // Load the event add template before checking the top-level controls.
    const template = normalizeWhitespace(await loadTemplate());

    // Assert the compact top navigation exposes preview before editor content.
    expect(template).to.include("event_form::tab_button");
    expect(template).to.include("Hosts & Speakers");
    expect(template).to.include('id="event-preview-button"');
    expect(template).to.include(
      'class="group btn-primary-outline btn-mini inline-flex h-7 min-w-28 items-center justify-center gap-2 whitespace-nowrap disabled:cursor-not-allowed disabled:opacity-50"',
    );
    expect(template.indexOf('id="event-preview-button"')).to.be.lessThan(
      template.indexOf('data-content="details"'),
    );
  });

  it("places the pending changes alert under the top controls", async () => {
    // Load the event add template before checking pending alert placement.
    const template = normalizeWhitespace(await loadTemplate());

    // Assert the save alert follows the title reminder and uses compact actions.
    const previewIndex = template.indexOf('id="event-preview-button"');
    const alertIndex = template.indexOf('id="pending-changes-alert"');

    expect(alertIndex).to.be.greaterThan(previewIndex);
    expect(template).to.not.include("icon-clock");
    expect(template).to.include('id="pending-changes-alert" class="hidden"');
    expect(template).to.include('class="min-w-0 flex-1 break-words text-sm/6"');
    expect(template).to.include(
      'class="btn-primary btn-mini h-7! w-24 text-nowrap ms-auto"',
    );
  });

  it("keeps bottom actions in the main grid column", async () => {
    // Load the event add template before checking grid placement classes.
    const template = normalizeWhitespace(await loadTemplate());

    // Assert root-level actions after the form wrapper stay in the form column.
    expect(template).to.include(
      'class="flex flex-wrap items-center justify-end gap-3 mt-6 px-4 xl:col-start-2 xl:px-0"',
    );
  });

  it("keeps the event form navigation in the shared page scroll", async () => {
    // Load the event add template before checking sidebar scroll behavior.
    const template = normalizeWhitespace(await loadTemplate());

    // Assert the form navigation scrolls with the active event content.
    expect(template).to.not.include('class="sticky top-6"');
    expect(template).to.not.include(
      '<label for="add-event-section-select" class="form-label mb-2 lg:hidden">Section</label>',
    );
    expect(template).to.include('id="add-event-section-select"');
    expect(template).to.include(
      'class="select-primary w-full sm:w-sm xl:hidden"',
    );
    expect(template).to.include(
      'class="hidden flex-col gap-1 font-medium xl:flex"',
    );
    expect(template).to.include(
      'class="col-span-full row-start-2 grid h-full content-start min-h-0 min-w-0 gap-y-8 group-has-[#pending-changes-alert:not(.hidden)]/event-page:row-start-3 xl:grid-cols-[12rem_minmax(0,1fr)] xl:content-stretch xl:gap-x-8 xl:gap-y-0"',
    );
    expect(template).to.include(
      'class="min-w-0 pt-0 xl:row-span-full xl:self-stretch xl:border-r xl:border-stone-900/10 xl:py-0 xl:pr-8"',
    );
    expect(template).to.not.include("lg:border-b-0");
    expect(template).to.include(
      '<div class="min-w-0"> <div class="space-y-12">',
    );
  });

  it("keeps top event tabs at their content width", async () => {
    // Load the event add template before checking tab sizing constraints.
    const template = normalizeWhitespace(await loadTemplate());

    // Prevent horizontal tabs from inheriting the full-width sidebar tab layout.
    expect(template).to.include(
      "[&>li]:w-auto [&>li]:shrink-0 [&>li>div>button]:w-auto [&>li>div>button]:whitespace-nowrap",
    );
    expect(template).to.include('class="min-w-0 overflow-x-auto"');
  });
});
