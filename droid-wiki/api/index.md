# API

GOUP Alliance exposes a versioned JSON REST API at `/api/v1/`. It is primarily used by MCP tools and third-party integrations. The full OpenAPI spec is in `docs/openapi.yaml`.

## Authentication

Most endpoints require either a session cookie (obtained via the web login flow) or an API token passed as a bearer token:

```
Authorization: Bearer <token>
```

API tokens are managed at `/settings/tokens` in the dashboard and via the `/api/v1/me/tokens` endpoints.

## Route groups

Routes are defined in `ocg-server/src/router/api.rs`.

### Public endpoints (no auth required)

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/health` | Health check |
| `GET` | `/api/v1/meta/filters` | Available filter options |
| `GET` | `/api/v1/alliances` | List alliances |
| `GET` | `/api/v1/alliances/{alliance}` | Get alliance details |
| `GET` | `/api/v1/alliances/{alliance}/groups` | List groups in an alliance |
| `GET` | `/api/v1/alliances/{alliance}/events` | List events for an alliance |
| `GET` | `/api/v1/groups/{alliance}/{group_slug}` | Get group details |
| `GET` | `/api/v1/events/{alliance}/{group_slug}/{event_slug}` | Get event details |
| `GET` | `/api/v1/jobs` | List jobs |
| `GET` | `/api/v1/jobs/{slug}` | Get job by slug |
| `GET` | `/api/v1/landscape` | List landscape entries |
| `GET` | `/api/v1/search` | Search events, groups, jobs, landscape |

### Authenticated user endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/me` | Current user profile |
| `PATCH` | `/api/v1/me` | Update profile |
| `GET` | `/api/v1/me/tokens` | List API tokens |
| `POST` | `/api/v1/me/tokens` | Create API token |
| `DELETE` | `/api/v1/me/tokens/{token_id}` | Revoke API token |
| `POST` | `/api/v1/groups/{alliance}/{group_id}/join` | Join a group |
| `DELETE` | `/api/v1/groups/{alliance}/{group_id}/join` | Leave a group |
| `GET` | `/api/v1/events/{alliance}/{event_id}/attendance` | Get attendance status |
| `POST` | `/api/v1/events/{alliance}/{event_id}/attend` | RSVP to event |
| `DELETE` | `/api/v1/events/{alliance}/{event_id}/attend` | Cancel RSVP |
| `POST` | `/api/v1/jobs/{job_id}/applications` | Apply to a job |

### Admin endpoints

These require alliance or group admin role.

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/alliances` | Create alliance |
| `PATCH` | `/api/v1/alliances/{alliance}` | Update alliance |
| `POST` | `/api/v1/alliances/{alliance}/groups` | Create group |
| `PATCH` | `/api/v1/alliances/{alliance}/groups/{group_id}` | Update group |
| `POST` | `/api/v1/groups/{group_id}/events` | Create event |
| `PATCH` | `/api/v1/events/{event_id}` | Update event |
| `POST` | `/api/v1/jobs` | Create job |
| `PATCH` | `/api/v1/jobs/id/{job_id}` | Update job |
| `DELETE` | `/api/v1/jobs/id/{job_id}` | Delete job |
| `POST` | `/api/v1/landscape` | Create landscape entry |
| `PATCH` | `/api/v1/landscape/{entry_id}` | Update landscape entry |

## Response format

All responses are JSON. Errors follow a consistent shape:

```json
{
  "error": "human-readable message"
}
```

## MCP tools

The MCP server at `mcp/server.mjs` wraps several of these API endpoints as MCP tools. See [MCP server](../services/mcp-server.md) for the full tool list.

## OpenAPI spec

The full spec is at `docs/openapi.yaml`. Load it into any OpenAPI viewer (Swagger UI, Redoc, etc.) for interactive documentation.
