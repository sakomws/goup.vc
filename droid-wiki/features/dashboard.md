# Dashboard

The dashboard feature exposes three distinct role-scoped surfaces: the user dashboard, the group admin dashboard, and the alliance admin dashboard. Each is rendered server-side with MiniJinja templates and driven by HTMX partial replacements.

## Role hierarchy

| Role | Access | Permission check |
|---|---|---|
| Authenticated user | `/dashboard/user` | Session presence only |
| Group admin | `/dashboard/group` | `user_has_selected_group_permission` middleware |
| Alliance admin | `/dashboard/alliance` | `user_has_alliance_dashboard_permission` middleware |

## Permission enforcement

Permissions are injected at the router level in `ocg-server/src/router/dashboard.rs` via axum `middleware::from_fn_with_state` layers that run before each route group. The middleware variants are:

- `auth::user_has_selected_group_permission(GroupPermission::*)` — verifies the acting user holds the named permission in the currently selected group.
- `auth::user_has_selected_alliance_permission(AlliancePermission::*)` — verifies the permission within the selected alliance.
- `auth::user_has_alliance_dashboard_permission` — broad alliance admin gate.

Write operations carry their own stricter permission layer on top of the read-only layer.

## User dashboard

Implemented in `ocg-server/src/handlers/dashboard/user/`. Sections include:

- **Profile** — account settings and profile editing.
- **Mock interviews** — submitted requests, matched pairs, scheduling and feedback.
- **Session proposals** — CFS (call for speakers) submissions, co-speaker invitations, accept/reject flows.
- **Submissions** — resubmit or withdraw submitted CFS entries.

## Group dashboard

Implemented in `ocg-server/src/handlers/dashboard/group/` with sub-modules for each functional area. Permission layers are applied per area:

| Area | Required permission |
|---|---|
| Events (read/write) | `GroupPermission::EventsWrite` |
| Members (write) | `GroupPermission::MembersWrite` |
| Settings (write) | `GroupPermission::SettingsWrite` |
| Sponsors / store (write) | `GroupPermission::SponsorsWrite` |
| Team management | `GroupPermission::TeamWrite` |

Sections include event management, attendee notifications, member join-request approval/rejection, phone-request approval, LinkedIn block-listing, spotlights, intentional-dating introductions, sponsor management, store items, and the accelerator sub-dashboard.

Database queries backing the group dashboard are centralised in `ocg-server/src/db/dashboard/group.rs` (1 979 lines, the largest non-test file in the project).

## Alliance dashboard

Implemented in `ocg-server/src/handlers/dashboard/alliance/`. Permission layers:

| Area | Required permission |
|---|---|
| Read views | `AlliancePermission::Read` |
| Group/landscape mutations | `AlliancePermission::GroupsWrite` |
| Settings mutations | `AlliancePermission::SettingsWrite` |
| Taxonomy mutations | `AlliancePermission::TaxonomyWrite` |

Sections include analytics (with publishable public reports), email template management, event category management, group activation/deactivation, landscape management, partner integrations, regions, settings, and team management. Alliance creation is a platform-level endpoint behind `AlliancePermission` and is accessible from `/dashboard/alliance/create`.

## Common utilities

`ocg-server/src/handlers/dashboard/common.rs` provides `search_user`, a shared endpoint mounted under both group and user dashboard routers for finding users by name or email.

## Active contributors

Cintia Sánchez García, Sako Mammadov, Sergio Castaño Arteaga, Shahriyar Mammadov
