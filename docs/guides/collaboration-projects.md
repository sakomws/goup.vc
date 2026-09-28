# Collaboration projects

Collaboration projects are group-owned workspaces for sustained work that does
not fit the event or organizer-team models. Project membership is deliberately
separate from `group_team`: group organizers administer projects through
`group.projects.write`, while project owners, contributors, and viewers only
receive access within the project.

## Model

- Projects have a lifecycle (`proposed`, `active`, `paused`, `completed`, or
  `archived`) and visibility (`private`, `members`, or `public`).
- A project may link to an existing landscape entry and/or accelerator cohort.
- Goals and assigned tasks are normalized records rather than embedded JSON.
- Accepted owners and contributors can publish chronological updates.
- Activity is append-only and records important workspace changes.
- Outcomes store baseline, target, achieved values, units, and linked evidence.
- Expert office-hour sessions have capacity-limited, idempotent bookings.

## Workflows

Organizers use **Group dashboard → Projects** to create and manage workspaces.
The `group.projects.write` permission is granted to group admins and inherited
alliance admins/group managers.

Public projects are available at:

`/{alliance}/group/{group}/projects/{project}`

Accepted owners and contributors can post updates themselves. Any accepted
project member can book a future office-hour session with remaining capacity.
These operations authorize against `collaboration_project_member`, never
`group_team`.

## Notifications and summaries

Invitation, booking, and session-reminder deliveries use deterministic keys in
`collaboration_notification_delivery`. Retrying the same operation therefore
does not schedule duplicate delivery. `enqueue_due_collaboration_reminders`
claims only the first reminder for each booking.

`get_collaboration_outcome_summary` returns totals, measured outcomes, targets
met, and evidence links for dashboard and public profile rendering.

## Migration compatibility

Migration `0124_collaboration_projects.sql` intentionally references only
objects introduced before migration 0123. It is valid when 0123 is absent so
the collaboration and opportunity branches can be reviewed and merged
independently.
