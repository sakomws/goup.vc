# Opportunity board

The opportunity board at `/opportunities` provides one searchable public view
of grants, funding, partnerships, research calls, published jobs, and open
calls for speakers.

## Source ownership

- Grants, funding, partnerships, and research calls are native opportunity
  records. Their creator manages draft, published, and deleted states at
  `/dashboard/opportunities`.
- Jobs are live projections of published, non-expired job records. Job posting,
  applications, expiration, and moderation remain in the jobs workflow.
- Event and standing group calls for speakers are live projections while their
  existing CFS windows are open. Proposals and reviews remain in the CFS
  workflow.

The projection design avoids duplicated applications, proposals, moderation
state, and lifecycle jobs.

## Privacy and permissions

Public visitors only receive published, open, public records. Signed-in users
may see member-only records. Native records can only be changed, published,
unpublished, or deleted by their creator. CSV export and MCP search apply the
same public visibility rules and never include applicant, proposal, reviewer,
or saved-search owner data.

## Saved searches and digests

Signed-in users can save the current filters as a daily or weekly search. A new
search is inactive until the user reviews its preview and explicitly activates
it. Users can delete searches at any time.

The notification worker records each scheduled digest run before enqueueing.
The `(saved_search_id, scheduled_for)` key makes retries idempotent. Digests are
optional notifications: verified users who disable optional notifications do
not receive them. Only matches created since the previous run are included.

## Integrations

- Global search presents an Opportunities section.
- The home feed uses the same unified board query.
- `/opportunities.csv` exports up to 100 visible filtered rows.
- MCP tool `goup_search_opportunities` accepts the same query, kind, location,
  remote, limit, and offset filters.
