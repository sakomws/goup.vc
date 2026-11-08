# What GOUP's First Automated Security Audit Found

This post is for GOUP maintainers, contributors, and security researchers. Its purpose is to publish
the results of the November 8, 2026 automated audit clearly enough to reproduce and challenge them.
The conclusion comes first: **no vulnerabilities were confirmed** by the source-only audit, but it
identified **two authorization leads that require validation**. Neither lead has a severity because
neither was executed against a database or deployment.

## The result in one minute

The audit found that GOUP's primary browser path has several strong controls, including route-level
permission middleware and database authorization checks for sensitive operations such as moving
events and assigning custom domains.

It also found two places where that authorization model may not be consistently enforced:

1. `/api/v1` mutation handlers appear to treat a bearer token's scope as the complete authorization
   decision, without also checking the caller's permission for the target resource.
2. Many functions written in PostgreSQL's procedural language, PL/pgSQL, trust a caller-provided
   actor ID. When mutations are enabled, the Model Context Protocol (MCP) server passes a tool
   caller's selected actor ID to a subset of those functions. Other unchecked functions would rely
   on authorization by any caller that reaches them without the browser middleware.

These are source-grounded leads, not demonstrated exploits. The audit could not determine deployment
controls such as ingress restrictions, MCP token distribution, or database role grants. It also
could not execute GOUP code. Those facts could prevent or materially change the impact.

## Audit facts

- **Repository:** [`sakomws/goup.vc`](https://github.com/sakomws/goup.vc)
- **Audited commit:** [`f8fb052c8f795dc0fb1e3532f159e357727dbbc7`](https://github.com/sakomws/goup.vc/commit/f8fb052c8f795dc0fb1e3532f159e357727dbbc7)
- **Workflow run:** [Slack security audit #42467120909](https://github.com/sakomws/goup.vc/actions/runs/42467120909)
- **Profile:** `quick`
- **Scope:** the entire repository
- **Budget:** eight agent invocations
- **Execution policy:** source and local sandbox only
- **Execution performed:** none; no application, database, fixture, build, or live system ran
- **Coverage recorded:** eight covered units, two candidate units, and one deferred unit
- **Audit procedure:** Cloudflare's
  [`security-audit-skill`](https://github.com/cloudflare/security-audit-skill/tree/c1c8a8c1471069fb0e188eeaff69b8e8db6564a8),
  pinned to commit `c1c8a8c1471069fb0e188eeaff69b8e8db6564a8`

This was a partial quick-profile review, not an exhaustive security assessment. Public site and
dashboard rendering, meetings, notifications, go-to-market agents, static JavaScript, continuous
integration workflows, Helm configuration, and deployment scripts were among the deferred areas.

## Lead 1: API scopes may not enforce resource authorization

- **Status:** needs validation
- **Severity:** unassigned
- **Boundary:** bearer-token scope to alliance, group, event, job, and landscape resources

### What the source shows

The API router exposes mutation routes for alliances, groups, events, jobs, and landscape entries:

1. [`ocg-server/src/router/api.rs`](https://github.com/sakomws/goup.vc/blob/f8fb052c8f795dc0fb1e3532f159e357727dbbc7/ocg-server/src/router/api.rs#L12-L72)
   maps the `/api/v1` endpoints to mutation handlers.
2. [`ApiUser::require_scope`](https://github.com/sakomws/goup.vc/blob/f8fb052c8f795dc0fb1e3532f159e357727dbbc7/ocg-server/src/handlers/api/auth.rs#L22-L29)
   checks whether the token has the requested scope or `admin:platform`. The check does not receive
   a target alliance, group, event, job, or landscape entry.
3. The
   [`update_alliance`](https://github.com/sakomws/goup.vc/blob/f8fb052c8f795dc0fb1e3532f159e357727dbbc7/ocg-server/src/handlers/api/admin.rs#L42-L53)
   and
   [`create_event`](https://github.com/sakomws/goup.vc/blob/f8fb052c8f795dc0fb1e3532f159e357727dbbc7/ocg-server/src/handlers/api/admin.rs#L82-L100)
   handlers check a scope and then call the database with caller-selected target identifiers. No
   per-resource authorization check is visible in these handler paths.
4. The token creation handler accepts one to sixteen caller-selected scopes. The
   [`create`](https://github.com/sakomws/goup.vc/blob/f8fb052c8f795dc0fb1e3532f159e357727dbbc7/ocg-server/src/handlers/api/tokens.rs#L32-L49)
   handler and
   [`CreateApiTokenInput`](https://github.com/sakomws/goup.vc/blob/f8fb052c8f795dc0fb1e3532f159e357727dbbc7/ocg-server/src/handlers/api/tokens.rs#L76-L81)
   do not visibly limit requested scopes to the user's effective roles.
5. The
   [`update_alliance` SQL function](https://github.com/sakomws/goup.vc/blob/f8fb052c8f795dc0fb1e3532f159e357727dbbc7/database/migrations/functions/dashboard-alliance/update_alliance.sql)
   uses the actor ID for the audit log but does not visibly call a permission function before the
   update.

The source also shows the intended stronger pattern. For example,
[`move_event`](https://github.com/sakomws/goup.vc/blob/f8fb052c8f795dc0fb1e3532f159e357727dbbc7/database/migrations/functions/dashboard-group/move_event.sql#L29-L37)
checks `user_has_alliance_permission`, and
[`upsert_custom_domain`](https://github.com/sakomws/goup.vc/blob/f8fb052c8f795dc0fb1e3532f159e357727dbbc7/database/migrations/functions/dashboard-group/upsert_custom_domain.sql#L40-L52)
checks `user_has_group_permission` inside SQL.

### What remains unknown

The audit did not prove that an unauthorized mutation succeeds. It could not inspect external
gateway or web application firewall rules, and it did not run the handler and database path. It
also does not know whether production access to token creation or `/api/v1` mutations is restricted
outside this repository.

### Safe validation plan

Run this only against a disposable local database:

1. Create test user A with no alliance or group role.
2. Create a separate alliance, group, and event owned by test user B.
3. As A, request an `admin:alliance` token through the local token endpoint.
4. Use the token to change only a harmless description field on B's disposable alliance.
5. Repeat with a `write:events` token and B's disposable event.
6. Record whether the API returns success and whether the database row changes, then delete the
   environment.

Do not run this validation against production, shared staging, or real tenant data.

## Lead 2: Database mutations may trust caller-provided actor identity

- **Status:** needs validation
- **Severity:** unassigned
- **Boundary:** MCP bearer token and other non-middleware callers to PL/pgSQL mutation functions

### What the source shows

The regular browser path applies permission middleware before many dashboard handlers. The lead is
about callers that can reach mutation functions without that middleware:

1. The MCP server authenticates requests with
   [one configured bearer token](https://github.com/sakomws/goup.vc/blob/f8fb052c8f795dc0fb1e3532f159e357727dbbc7/mcp/server.mjs#L8-L17)
   and compares it in
   [`isAuthorized`](https://github.com/sakomws/goup.vc/blob/f8fb052c8f795dc0fb1e3532f159e357727dbbc7/mcp/server.mjs#L133-L135).
2. Mutating MCP tools are disabled unless `MCP_ENABLE_MUTATIONS=true`. When enabled,
   [`createLead`](https://github.com/sakomws/goup.vc/blob/f8fb052c8f795dc0fb1e3532f159e357727dbbc7/mcp/server.mjs#L451-L478)
   accepts `actor_user_id` and `alliance_id` from tool arguments and passes both to
   `add_gtm_lead`.
3. [`runPsql`](https://github.com/sakomws/goup.vc/blob/f8fb052c8f795dc0fb1e3532f159e357727dbbc7/mcp/server.mjs#L1355-L1378)
   executes the generated statement with the server's database credentials.
4. [`add_gtm_lead`](https://github.com/sakomws/goup.vc/blob/f8fb052c8f795dc0fb1e3532f159e357727dbbc7/database/migrations/functions/dashboard-gtm/add_gtm_lead.sql)
   validates data relationships but does not visibly verify that `p_actor_user_id` has permission
   for `p_alliance_id`.
5. [`add_group_team_member`](https://github.com/sakomws/goup.vc/blob/f8fb052c8f795dc0fb1e3532f159e357727dbbc7/database/migrations/functions/dashboard-group/add_group_team_member.sql)
   is a representative dashboard mutation that inserts a caller-selected role and uses the actor ID
   for auditing without a visible permission check.

There are important source-level guardrails. MCP mutations default to disabled. The EC2 setup script
also defaults the MCP listener to `127.0.0.1`, even though running `mcp/server.mjs` directly defaults
to `0.0.0.0`. See
[`scripts/setup-mcp-ec2.sh`](https://github.com/sakomws/goup.vc/blob/f8fb052c8f795dc0fb1e3532f159e357727dbbc7/scripts/setup-mcp-ec2.sh#L10-L16).

### What remains unknown

The audit did not determine:

- whether MCP mutations are enabled in any deployment;
- who can obtain the MCP bearer token;
- whether the MCP database role can execute the affected functions;
- whether row-level security or other database policy blocks the writes; or
- whether network policy prevents untrusted clients from reaching the MCP listener.

Without those deployment facts and a sandbox execution, this remains a lead rather than a confirmed
vulnerability.

### Safe validation plan

Run this only in scratch infrastructure with synthetic records:

1. Load the migrations into disposable PostgreSQL.
2. Create a role-less test actor and separate victim-owned alliance and group records.
3. Using the same database role as MCP, call `add_gtm_lead` and `add_group_team_member` with the
   role-less actor and victim-owned resource IDs.
4. Observe whether the rows are inserted.
5. Start the local MCP server with mutations enabled, a dummy bearer token, and the disposable
   database.
6. Call the relevant tool with synthetic IDs and confirm whether the actor identity is bound to an
   authenticated principal.
7. Destroy the environment.

Do not enable mutation tools or probe actor IDs on a live or shared MCP server.

## Other hardening observations

These observations are not findings:

- `search_user` may let an authenticated dashboard user confirm whether an exact email address is
  registered. Consider requiring a stronger role or removing exact-email matching.
- Direct execution of the MCP server defaults to `0.0.0.0`, uses one static bearer token, and allows
  cross-origin requests. The provided EC2 setup script is safer because it defaults to
  `127.0.0.1`.
- The Zoom recording webhook stores a provider-supplied share URL after signature verification.
  A scheme and host allowlist would reduce impact if the webhook secret were compromised.
- Event discovery validates a resolved address before the HTTP client resolves it again. Connecting
  to the already validated address would reduce the DNS-rebinding time-of-check/time-of-use window.
- Custom-domain verification relies on Cloudflare's DNS-over-HTTPS resolver and a positive cache
  that can retain routing for up to one minute.
- Some spotlight and store-item writes rely on HTTP middleware without repeating permission checks
  in SQL.

Each item is a defense-in-depth opportunity. None was demonstrated as an exploitable vulnerability
in this run.

## Controls that worked well in source review

The audit also recorded positive patterns:

- Dashboard routes apply permission middleware by resource bucket and re-resolve selected alliance
  and group context from the session.
- High-impact SQL operations such as event moves, custom-domain changes, and co-hosting repeat
  authorization checks at the database boundary.
- Stripe and Zoom webhooks verify signatures with timestamp windows and constant-time comparison.
- Image uploads enforce size, dimensions, file type, extension, and SVG-content checks.
- Redirects constrain destinations to configured hosts.
- API tokens are stored as hashes and support scopes and revocation.
- OAuth and OpenID Connect login paths enforce state and nonce checks and require verified provider
  email addresses.

## Recommendation and next steps

The recommended invariant is simple: every mutation should enforce resource authorization at the
last trusted decision point. API scopes should limit what a token can attempt, while resource checks
should decide where that token can act. A database function that accepts an actor ID should verify
that actor's permission before changing data.

Recommended sequence:

1. Assign a GOUP maintainer to each lead and set a validation date.
2. Run both validation plans in disposable local infrastructure.
3. If confirmed, restrict token scopes to a user's effective roles and add per-resource checks to
   the API handlers.
4. Add database permission checks to mutation functions, following `move_event` and
   `upsert_custom_domain`.
5. Add regression tests for cross-alliance, cross-group, and caller-selected actor cases.
6. Review deployed ingress, MCP mutation flags, token holders, network policy, and database grants.
7. Run a broader audit after fixes and publish the confirmed result.

- **Owner:** not assigned in the audit artifact
- **Deadline:** not set in the audit artifact
- **Decision:** validation must precede severity assignment or vulnerability claims

## Open questions

- Are `/api/v1` mutation and token-management routes restricted by deployment infrastructure?
- Is `MCP_ENABLE_MUTATIONS` enabled anywhere?
- Which principals hold the MCP bearer token?
- What database role does MCP use, and what function privileges or row-level policies constrain it?
- Who owns validation, and by what date?

Until those questions are answered, the accurate public statement is unchanged: **no
vulnerabilities were confirmed, and two authorization leads require validation**.
