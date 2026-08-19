# GOUP Remote MCP Server

This directory contains a lightweight remote MCP server for GOUP operational
tools. It exposes MCP JSON-RPC over HTTP at `/mcp` and loads tool definitions
from `tools.json`.

## Architecture

- `server.mjs` is the HTTP JSON-RPC server. It handles MCP initialization,
  `tools/list`, `tools/call`, simple health checks, bearer-token auth, and the
  built-in database-backed tool actions.
- `tools.json` is the tool catalog. Static tools render `output.text` templates;
  action tools call JavaScript handlers in `server.mjs`.
- `package.json` declares the Node runtime and local scripts.
- `scripts/setup-mcp-ec2.sh` in the repository root installs the EC2 systemd
  service and writes `~/.config/ocg/mcp.env`.

Static tool output supports `{{ name }}` placeholders from tool arguments. If an
argument is omitted and the corresponding JSON schema property has a `default`,
the default value is rendered.

## Run Locally

```bash
cd mcp
MCP_BEARER_TOKEN='replace-with-a-strong-token' npm start
```

The server listens on `http://127.0.0.1:8787/mcp` by default.

Useful checks:

```bash
curl http://127.0.0.1:8787/health
curl http://127.0.0.1:8787/tools
```

Validate server syntax and `tools.json`:

```bash
cd mcp
npm run check
```

## Run Remotely

Use the EC2 setup helper from the repository root:

```bash
./scripts/setup-mcp-ec2.sh
```

The script generates or reuses a bearer token, writes `~/.config/ocg/mcp.env`,
installs a `goup-mcp` systemd service, starts it, and prints both an NGINX
`/mcp` proxy snippet and a Cursor/client config.

Enable mutation tools only when the MCP endpoint is protected:

```bash
MCP_ENABLE_MUTATIONS=true ./scripts/setup-mcp-ec2.sh
```

Manual background startup is also supported. Set a bearer token before exposing
the server publicly:

```bash
cd ~/goup.vc/mcp
MCP_BEARER_TOKEN='replace-with-a-strong-token' \
nohup npm start > ~/goup-mcp.log 2>&1 &
```

Mutation tools, such as event creation, are disabled by default. Enable them
only on a protected network or behind HTTPS with a bearer token:

```bash
MCP_ENABLE_MUTATIONS=true
```

The event creation tool uses `psql` and reads database connection details from
`DATABASE_URL`, `TERN_CONF`, or `$HOME/.config/ocg/tern.conf`.

## Update an Existing EC2 MCP Service

After MCP code or `tools.json` changes are merged to `main`, update EC2 with:

```bash
cd ~/goup.vc
git pull origin main
cd mcp
npm run check
sudo systemctl restart goup-mcp
sudo systemctl status goup-mcp --no-pager
```

Check the remote endpoint locally on EC2:

```bash
source "$HOME/.config/ocg/mcp.env"
curl -H "Authorization: Bearer $MCP_BEARER_TOKEN" http://127.0.0.1:8787/health
curl -H "Authorization: Bearer $MCP_BEARER_TOKEN" \
  -H "content-type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
  http://127.0.0.1:8787/mcp
```

Authentication is done with an HTTP bearer token. Clients must send:

```text
Authorization: Bearer <token>
```

Put it behind HTTPS, then configure your MCP client with the remote URL:

```json
{
  "mcpServers": {
    "goup-vc": {
      "url": "https://mcp.goup.vc/mcp",
      "headers": {
        "Authorization": "Bearer replace-with-a-strong-token"
      }
    }
  }
}
```

## Add Tools

Add a new entry to `tools.json`:

```json
{
  "name": "goup_example_tool",
  "title": "GOUP Example Tool",
  "description": "Explains what this tool does.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "name": {
        "type": "string",
        "description": "Example input."
      }
    },
    "required": ["name"],
    "additionalProperties": false
  },
  "output": {
    "text": "Hello {{ name }}"
  }
}
```

Restart the MCP server after editing `tools.json`. MCP clients discover tools
through the standard `tools/list` method.

## Included Tools

- `goup_deploy_after_pull`: full EC2 update flow after pulling `origin main`.
  Accepts optional `build_jobs` from `1` to `4`; default is `2`.
- `goup_run_migrations`: run `tern` migrations.
- `goup_release_build_background`: build `ocg-server` in the background.
  Accepts optional `build_jobs` from `1` to `4`; default is `2`.
- `goup_service_status`: inspect systemd logs and local HTTP status.
- `goup_create_event`: create an unpublished draft event through `add_event`.
- `goup_update_event`: update an existing event through `update_event`.
- `goup_create_startup`: add a startup to the landscape through `add_landscape_entry`; published by default.
- `goup_create_github_project`: add a GitHub project to the landscape through `add_landscape_entry`; requires `github_url`.
- `goup_create_startups_bulk`: add many startups to the landscape in one call, sharing `actor_user_id`, `alliance_id`, and a default `published`; each entry is created independently so one failure does not abort the rest, and the result reports per-entry status.
- `goup_search`: search public events, groups, jobs, ecosystem entries, and wiki sources in one call.
- `goup_query_community_analytics`: aggregate public event and confirmed-attendee
  counts for a required time range, optionally narrowed to an alliance and ISO
  country codes. It returns totals, country breakdowns, and top communities;
  it excludes test, deleted, unpublished, canceled, and inactive records, and
  returns no attendee, user, event, or group identifiers. For a continent
  request, pass the relevant ISO 3166-1 alpha-2 country-code set.
- `goup_search_groups`: list or search groups.
- `goup_search_events`: list or search events.
- `goup_search_members`: list or search regular group members.
- `goup_search_teams`: list or search alliance and group team members.
- `goup_search_jobs`: search active published jobs.
- `goup_search_landscape`: search published landscape entries.
- `goup_search_wiki`: list or search wiki feed sources.
- `goup_submit_talk`: create and submit a talk proposal to an open event CFS.

## Landscape link health check

`health-check.mjs` is a standalone job that keeps the landscape from filling up
with dead links. Startups get acquired, shut down, or drop their domain, and
those entries otherwise stay published forever.

Each run walks every published landscape entry with a website, checks whether
the site still resolves, and records a per-entry failure streak in a small JSON
state file. A site that answers at all, including one that blocks bots with 401
or 403, counts as alive. Only hard signals count against an entry: a domain that
no longer resolves, a refused or reset connection, a timeout, or a `404`, `410`,
or `451`. Transient problems such as `5xx` are retried once and, if still
failing, only add to the streak rather than acting immediately.

After `HEALTHCHECK_FAILURE_THRESHOLD` consecutive failing runs (default `3`) an
entry is unpublished through `update_landscape_entry_published`, which is
reversible and writes an audit log. Entries are never deleted, so a human can
review and permanently remove them later. The job is a dry run by default and
only unpublishes when `HEALTHCHECK_APPLY=true` and a valid
`HEALTHCHECK_ACTOR_USER_ID` are set.

Run it once by hand:

```bash
cd mcp
# dry run, prints what it would unpublish
npm run health-check
# apply
HEALTHCHECK_APPLY=true HEALTHCHECK_ACTOR_USER_ID='<uuid>' npm run health-check
```

Environment:

- `DATABASE_URL` or `TERN_CONF`: database connection, same as the MCP server.
- `HEALTHCHECK_ACTOR_USER_ID`: user id recorded as the actor for unpublish audit logs.
- `HEALTHCHECK_APPLY`: `true` to unpublish; otherwise a dry run. Defaults to a dry run.
- `HEALTHCHECK_FAILURE_THRESHOLD`: consecutive failing runs before unpublishing. Defaults to `3`.
- `HEALTHCHECK_STATE_FILE`: streak state file. Defaults to `~/.config/ocg/landscape-health-state.json`.
- `HEALTHCHECK_TIMEOUT_MS`: per-request timeout. Defaults to `15000`.
- `HEALTHCHECK_CONCURRENCY`: parallel checks. Defaults to `12`.

Schedule it on EC2 with the units in `mcp/systemd`. They reuse
`~/.config/ocg/mcp.env`, so add `HEALTHCHECK_APPLY=true` and
`HEALTHCHECK_ACTOR_USER_ID` there once you have reviewed a dry run:

```bash
sudo cp mcp/systemd/goup-landscape-health.service /etc/systemd/system/
sudo cp mcp/systemd/goup-landscape-health.timer /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now goup-landscape-health.timer
# run once now and read the report
sudo systemctl start goup-landscape-health.service
journalctl -u goup-landscape-health.service -n 100 --no-pager
```

Adjust `User` and `WorkingDirectory` in the service file to match the
deployment before copying it.
