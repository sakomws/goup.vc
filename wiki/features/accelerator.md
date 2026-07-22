# Accelerator

The accelerator feature provides group admins with a structured program-management workflow inside the group dashboard. It is scoped to a single group and requires `GroupPermission::EventsWrite`.

## Source files

- Handler: `ocg-server/src/handlers/dashboard/group/accelerator.rs`
- Database trait: `ocg-server/src/db/accelerator.rs`
- Public application route: `/{alliance}/group/{group_id}/accelerator/cohorts/{cohort_id}/apply`
- Landscape type metadata: `ocg-server/src/types/landscape.rs` — a landscape entry can carry the kind `"accelerator"` with dedicated profile fields.

## Data model

An accelerator is organised as:

```
Program
  └── Cohort(s)
        ├── Curriculum weeks
        ├── Applications (with review/accept flow)
        ├── Members (accepted applicants)
        └── Weekly updates (with review flow)
```

Landscape entries of kind `accelerator` may expose additional metadata: application URL, curriculum URL, cohort status (`planned` / `open` / `running` / `completed`), start/end dates, tracks (comma-separated), and a weekly agenda.

## Dashboard operations

The `prepare_page` function executes two parallel queries:

1. `user_has_group_permission` — confirms `EventsWrite` for the current user.
2. `get_group_accelerator_dashboard` — fetches programs, cohorts, applications, members, weeks, and weekly updates in a single call.

Write endpoints and their permission guard:

| Endpoint | Operation |
|---|---|
| `add_program` | Add a new accelerator program |
| `add_cohort` | Add a cohort to a program |
| `add_week` | Add or update a curriculum week |
| `review_application` | Record a review decision on an application |
| `accept_application` | Accept an application and create a cohort member |
| `review_weekly_update` | Record a review decision on a weekly update |

All write handlers call `ensure_can_manage_accelerator` which re-verifies `EventsWrite` before any mutation.

## Public application flow

Prospective cohort members apply via the public route `/{alliance}/group/{group_id}/accelerator/cohorts/{cohort_id}/apply`. Once submitted, the application appears in the dashboard queue for review by a group admin.

## Active contributors

Sako Mammadov, Sergio Castaño Arteaga
