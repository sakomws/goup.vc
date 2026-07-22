

# Project overview

GOUP Alliance (internal codename **OCG** — Open Community Groups) is a community platform for builders, founders, and open-source contributors. It is live at [https://goup.vc](https://goup.vc).

## What the platform does

The platform lets operators create **alliances** (top-level organisations) that contain **groups** (sub-organisations). Within that structure it provides:

- Group and alliance discovery via `/explore`
- An events calendar with registration, check-in, and payment support
- A jobs board with automated discovery (`/jobs`)
- A landscape directory for startups and OSS ecosystems (`/landscape`)
- Member profiles, mentorship, coffee-meet, and mock-interview tooling
- A wiki feed aggregator (`/wiki`)
- Payments via Stripe (tickets, group store)
- Meetings via Google Meet or Zoom with optional YouTube recording publishing
- Dashboard tooling for alliance admins, group admins, and regular users
- A remote MCP server for operational tooling (`mcp/`)

## Codebase layout

| Path | Purpose |
|------|---------|
| `ocg-server/` | Main Rust web server |
| `ocg-redirector/` | Lightweight HTTP redirect service |
| `mcp/` | Node.js MCP JSON-RPC server |
| `database/migrations/` | PostgreSQL schema managed by tern |
| `tests/e2e/` | Playwright end-to-end tests |
| `tests/unit/` | Web-test-runner unit tests |
| `charts/goup/` | Helm charts for Kubernetes deployment |
| `justfile` | Task runner (see `just --list`) |

## Technology choices

- **Rust / axum** for the HTTP server layer
- **MiniJinja** for server-rendered HTML templates
- **HTMX** for dynamic UI fragments without a full SPA
- **deadpool-postgres** for PostgreSQL connection pooling
- **Figment** for layered configuration (YAML + `OCG_*` env vars)
- **tern** for SQL-first PostgreSQL migrations
- **rust-embed** to bundle static assets into the binary
- **Node.js** for the MCP server (`mcp/server.mjs`)
- **Playwright** for end-to-end tests
