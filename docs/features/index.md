# Features overview

**Active contributors:** Sergio Castaño Arteaga, Cintia Sánchez García, Sako Mammadov

GOUP Alliance is a community platform for builders, founders, and open source contributors. This section documents the platform's primary feature areas.

## Feature areas

| Feature | Description |
|---------|-------------|
| [Auth and sessions](auth.md) | OAuth (GitHub, Google, LinkedIn), password auth, axum-login sessions |
| [Events](events.md) | Event creation, RSVP, calendar, discovery, meetings, recording |
| [Groups and alliances](groups-and-alliances.md) | Group and alliance hierarchy, membership, team management |
| [Jobs](jobs.md) | Jobs board, global You.com-powered job discovery |
| [Landscape](landscape.md) | Startup and OSS project directory |
| [Payments](payments.md) | Stripe-backed ticketed events, checkout, refunds |
| [Notifications](notifications.md) | Email notifications, SMTP delivery queue |

## Platform model

The platform is organized around a three-level hierarchy:

```
Alliance  (e.g., GOUP Alliance)
  └── Group  (e.g., AI Builders Baku)
        └── Event  (e.g., Monthly Meetup #12)
```

Users can be members of multiple groups within an alliance. Groups host events and publish jobs to the platform jobs board. The landscape directory is alliance-scoped and lists startups and open source projects.

## Key cross-cutting concerns

- All user-facing pages are server-side rendered with MiniJinja templates in [ocg-server](../services/ocg-server.md).
- HTMX is used for partial page updates; the router exposes `/api/*` JSON endpoints for these.
- Background services (event discovery, job discovery, notifications, payments, meetings, recording) are coordinated via `tokio` tasks with a shared `CancellationToken`.
