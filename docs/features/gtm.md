# GTM pipeline

The GTM feature gives alliance and group organizers a nine-stage pipeline for
sponsors, startups, investors, speakers, and organizers. Agents draft work.
Humans approve every stage change.

Path: [/dashboard/alliance?tab=gtm](/dashboard/alliance?tab=gtm ':ignore') and
[/dashboard/group?tab=gtm](/dashboard/group?tab=gtm ':ignore')

## Source files

- Schema: `database/migrations/schema/0115_gtm_leads.sql`
- SQL functions: `database/migrations/functions/dashboard-gtm/`
- Types: `ocg-server/src/types/gtm.rs`
- Database trait: `ocg-server/src/db/gtm.rs`
- Agents: `ocg-server/src/services/gtm.rs`
- Handlers: `ocg-server/src/handlers/dashboard/gtm.rs`, `.../alliance/gtm.rs`, `.../group/gtm.rs`
- Templates: `ocg-server/templates/dashboard/gtm/`

## Pipeline

```text
lead_generation → reachout → get_response → qualification → proposal → negotiation
  → won → delivered → renewal → reachout
  → lost → reachout
```

Humans may move a lead to any stage or delete it from the pipeline. Agents
only suggest the next legal stage and cannot delete leads.

## Sponsor campaign workspace

Sponsor leads include a phase-one campaign workspace:

- alliance-wide or group-specific packages with normalized price, currency,
  billing period, and included deliverables
- multiple sponsor contacts, optionally linked to platform users
- proposals whose package terms are snapshotted when the proposal is created
- assigned follow-up tasks and reminders with open/completed/cancelled state
- proposal-linked or standalone deliverables with delivery state
- standardized lost reasons
- organizer-authored calls, emails, meetings, and notes in the existing
  `gtm_lead_activity` contact history

The lead detail page also exposes estimated value, currency, next action,
renewal date, and lost reason. Winning a sponsor continues to use the existing
`group_sponsor_id` linkage and side-effect flow.

The GTM list includes the due-task queue. `list_due_gtm_tasks` only returns
open tasks and `add_gtm_task` supports a per-lead unique `reminder_key`, so a future
worker can claim/enqueue notifications without producing duplicate reminders.
Notification delivery is intentionally not wired in phase one; organizers can
operate and complete the visible queue now.

## Permissions

Write access uses `alliance.gtm.write` and `group.gtm.write`. Alliance `admin`
and `groups-manager` receive both. Group `admin` receives group GTM write.
Router middleware enforces these permissions.

## Agents

Each run writes a pending `gtm_agent_draft`. Approving a draft applies its
suggested stage and optional won side effects:

- sponsor + group → create `group_sponsor` when checked
- startup / investor → create `landscape_entry` when checked
- organizer → pending group team invite when checked
- speaker → note only

Get Response is organizer-logged in v1: paste a reply, then approve the summary.

## MCP

`goup_search_leads`, `goup_create_lead`, `goup_delete_lead`,
`goup_transition_lead`, `goup_run_gtm_agent`, and `goup_review_gtm_draft`
expose the same HITL loop. Pipeline agents still cannot delete leads.
Mutations still require `MCP_ENABLE_MUTATIONS=true`.
