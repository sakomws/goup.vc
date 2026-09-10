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

```
lead_generation → reachout → get_response → qualification → proposal → negotiation
  → won → delivered → renewal → reachout
  → lost → reachout
```

Humans may move a lead to any stage. Agents only suggest the next legal stage.

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

`goup_search_leads`, `goup_create_lead`, `goup_transition_lead`,
`goup_run_gtm_agent`, and `goup_review_gtm_draft` expose the same HITL loop.
Mutations still require `MCP_ENABLE_MUTATIONS=true`.
