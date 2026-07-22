# Groups and alliances

**Active contributors:** Sergio Castaño Arteaga, Cintia Sánchez García, Sako Mammadov

## Purpose

The groups and alliances feature defines the organizational hierarchy of the platform. An alliance is the top-level container (e.g., "GOUP Alliance"). Groups are sub-communities within an alliance (e.g., "AI Builders Baku"). Groups host events, post jobs, and manage their own membership and teams.

## Directory layout

```
ocg-server/src/
├── handlers/alliance.rs / alliance/      # alliance public pages
├── handlers/group.rs / group/            # group public pages: about, members, events, …
├── handlers/dashboard/alliance.rs / alliance/  # alliance admin dashboard
├── handlers/dashboard/group.rs / group/  # group admin dashboard (settings, members, teams)
├── db/alliance.rs                        # DB queries for alliance pages
├── db/group.rs                           # DB queries for group pages
├── db/dashboard/alliance.rs             # DB queries for alliance dashboard
├── db/dashboard/group/                   # DB queries for group dashboard
├── templates/alliance.rs                 # MiniJinja template structs for alliance pages
├── templates/group.rs                    # MiniJinja template structs for group pages
├── types/alliance.rs                     # Alliance, AllianceDetails, and related types
└── types/group.rs                        # Group, GroupMember, Team, and related types
```

## Key abstractions

| Abstraction | File | Description |
|-------------|------|-------------|
| `DBAlliance` | `ocg-server/src/db/alliance.rs` | Trait: get_alliance, get_alliance_groups, get_alliance_events, … |
| `DBGroup` | `ocg-server/src/db/group.rs` | Trait: get_group, get_group_members, add_member, remove_member, … |
| `DBDashboard` | `ocg-server/src/db/dashboard.rs` | Composed sub-trait for all dashboard DB operations |

## Platform hierarchy

```
Alliance
  ├── metadata (name, slug, logo, description, social links)
  ├── categories (event categories scoped to the alliance)
  └── Group (one or more)
        ├── metadata (name, slug, logo, description, location)
        ├── members (users with roles: member, admin, owner)
        ├── teams (sub-groups within a group)
        └── events (hosted by the group)
```

Users join groups; group admins and owners manage settings, members, teams, and events through the dashboard. Alliance admins have visibility and control across all groups.

## How it works

### Public site

Alliance and group public pages are rendered by handlers in `ocg-server/src/handlers/alliance.rs` and `ocg-server/src/handlers/group.rs`. Routes follow the pattern `/:alliance_name` and `/:alliance_name/group/:group_name`.

### Dashboard

The group dashboard (accessible at `/dashboard/alliance/:alliance_id/group/:group_id/…`) provides organizer tools:

- **Settings** — edit group profile, location, social links.
- **Members** — invite, approve, remove, and role-manage members.
- **Teams** — create teams and invite team members.
- **Events** — create and manage group events.
- **Jobs** — post and manage job listings.

### Member roles and permissions

Permissions are checked via the `types/permissions.rs` module. Key roles: `Owner`, `Admin`, `Member`. Handlers extract the current user's role from the session and gate actions accordingly.

### Alliance team invitations

When a group team administrator invites a user, the invitation is recorded in the DB and an email is sent via `services/notifications`. The invited user accepts or declines from a link in the email. See [notifications](notifications.md) for the `AllianceTeamInvitation` and `GroupTeamInvitation` templates.

## Integration points

- [Events](events.md) — events belong to a group; group admins manage events via the dashboard.
- [Jobs](jobs.md) — jobs are posted by group organizers.
- [Auth](auth.md) — membership actions require authentication; role checks use the session user.
- [Notifications](notifications.md) — `GroupWelcome`, `GroupCustom`, `AllianceTeamInvitation`, `GroupTeamInvitation` email templates.
- [MCP server](../services/mcp-server.md) — `goup_search_groups`, `goup_search_members`, `goup_search_teams` tools.

## Entry points for modification

- Add a group field: extend the `Group` struct in `ocg-server/src/types/group.rs`, update `DBGroup` in `ocg-server/src/db/group.rs`, and add a migration.
- Add a new dashboard section: create a handler file under `ocg-server/src/handlers/dashboard/group/` and register the route in `ocg-server/src/router/dashboard.rs`.
- Change role logic: update `ocg-server/src/types/permissions.rs`.
