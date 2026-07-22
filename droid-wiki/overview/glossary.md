# Glossary

Terms used in the codebase and documentation.

---

**OCG** — Open Community Groups. The internal codename for the codebase. Config keys, environment variables (`OCG_*`), and many internal identifiers use this prefix.

**Alliance** — The top-level organisational unit in the platform. An alliance has a slug used as the first path segment in URLs (e.g., `/{alliance}/`). Modelled in `ocg-server/src/types/alliance.rs` and handled in `ocg-server/src/handlers/alliance.rs`.

**Group** — A sub-organisation within an alliance. Groups can have members, events, a store, and optional features such as an accelerator programme. Modelled in `ocg-server/src/types/group.rs`.

**Landscape** — A directory of startups, open-source projects, investors, and accelerators associated with an alliance. Schema introduced in `database/migrations/schema/0059_landscape.sql`. UI at `/landscape`.

**CFS** — Call for Speakers. A feature on events that allows attendees to submit speaker proposals. Schema in `database/migrations/schema/0001_initial.sql` (cfs tables).

**Tern** — The PostgreSQL migration tool used by this project ([github.com/jackc/tern](https://github.com/jackc/tern)). Migration files live in `database/migrations/schema/`. The `migrate.sh` script wraps tern invocations. Config files (`tern.conf`, `tern-tests.conf`, etc.) live outside the repo in `$OCG_CONFIG`.

**MCP** — Model Context Protocol. A JSON-RPC protocol for exposing tools to AI assistants. The `mcp/` directory contains a Node.js MCP server (`mcp/server.mjs`) that wraps `ocg-server` API calls.

**DynDB** — The dynamic trait object type for the database layer, defined in `ocg-server/src/db.rs`. Handlers receive a `DynDB` via axum state extraction rather than a concrete type, allowing mock implementations in tests.

**DynImageStorage** — Dynamic trait object for image storage, defined in `ocg-server/src/services/images.rs`. Two implementations exist: `Db` (images stored in PostgreSQL) and `S3` (images stored on S3-compatible object storage). Selected via `images.provider` in config.

**DynNotificationsManager** — Dynamic trait object for outbound notifications (currently email via SMTP). Defined in `ocg-server/src/services/notifications.rs`.

**DynPaymentsManager** — Dynamic trait object for payment processing. The Stripe implementation lives in `ocg-server/src/services/payments/`.

**DynMeetingsProvider** — Dynamic trait object for meeting creation (Google Meet or Zoom). Logic in `ocg-server/src/services/meetings.rs`.

**Figment** — The configuration library used by `ocg-server`. It merges defaults, an optional YAML file, and `OCG_*` environment variables. See `ocg-server/src/config.rs`, `Config::new()`.

**HTMX** — A JavaScript library used for dynamic page updates. The server detects `hx-request: true` headers to return HTML fragments instead of full pages. Stale-client detection uses the `x-ocg-commit-sha` header (see `router::refresh_stale_clients` in `ocg-server/src/router.rs`).

**Commit SHA header** (`x-ocg-commit-sha`) — A response header set by the server on every response containing the current build's commit SHA (embedded at compile time via `OCG_COMMIT_SHA` env var). HTMX and `ocgFetch` clients send this header back; if it differs from the current binary, the server responds with a page-refresh instruction.

**rust-embed** — A crate used to bundle static assets (JS, CSS, images) from `ocg-server/dist/static/` into the server binary at compile time. The `StaticFile` struct in `ocg-server/src/router.rs` serves these assets at `/static/{file}`.

**Activity tracker** (`ocg-server/src/activity_tracker.rs`) — Records page view events for alliances, groups, and events via `POST /alliances/{id}/views`, `/groups/{id}/views`, `/events/{id}/views`.

**Host pool** — A list of Zoom user accounts (`meetings.zoom.host_pool_users`) used as meeting hosts. The service rotates through the pool to respect per-host concurrency limits (`meetings.zoom.max_simultaneous_meetings_per_host`).

**Payment mode** — Either `test` (Stripe test keys) or `live` (real payments). Set via `payments.mode` in config. Defined in `ocg-server/src/types/payments.rs`.

**pgTAP** — A PostgreSQL extension for writing SQL unit tests. Tests live in `database/tests/`. Run with `just db-tests`.

**justfile** — The project's task runner config (requires [just](https://just.systems/)). Run `just --list` for all available tasks.
