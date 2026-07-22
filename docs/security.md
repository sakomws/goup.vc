

# Security

## Authentication model

Session authentication is managed by `axum-login` with `tower-sessions`. Sessions are stored in PostgreSQL (not in-memory), so they survive server restarts.

Session cookies are configured in `ocg-server/src/auth.rs` with:
- `HttpOnly: true` — not accessible to JavaScript
- `SameSite: Lax` — protects against most CSRF vectors
- `Secure: true` in production (configurable via `server.cookie.secure`)
- 7-day inactivity expiry

## OAuth providers

Three OAuth providers are supported: GitHub (OAuth 2.0), Google (OIDC), and LinkedIn (OAuth 2.0). Each is optional and configured independently. OAuth state parameters are validated on callback to prevent CSRF.

Provider config keys live under `server.oauth2` and `server.oidc` in `ocg-server/src/config.rs`.

## Password authentication

Passwords are hashed with Argon2id via the `password-auth` crate. The hash parameters (`m=19456,t=2,p=1`) are embedded in the hash string and verified without separate config.

## Input validation

All user-supplied input is validated with the `garde` crate before reaching the database layer. Validators are defined in `ocg-server/src/validation.rs` and applied to input structs via `#[derive(Validate)]`. Invalid input returns a 400 response before any DB query runs.

## SQL injection prevention

All database queries use `deadpool-postgres` prepared statements. There is no string-concatenated SQL anywhere in `ocg-server/src/db/`. Query parameters are passed as typed Rust values and bound by the PostgreSQL wire protocol.

## Image upload security

Images are validated for MIME type and size before storage. The `DynImageStorage` abstraction in `ocg-server/src/services/images.rs` handles both the DB backend (images stored as bytes) and the S3 backend. Uploaded images are not served from user-controlled URLs; they are proxied through the server.

## API token security

API tokens are issued as random bytes, stored hashed in the database, and compared in constant time. Token creation and revocation are handled in `ocg-server/src/handlers/api/tokens.rs`.

## MCP server

The MCP server (`mcp/server.mjs`) requires a bearer token for all requests:

```
Authorization: Bearer <token>
```

The token is generated during EC2 setup by `scripts/setup-mcp-ec2.sh` and stored in `~/.config/ocg/mcp.env`. Mutation tools (event creation, etc.) are disabled by default and must be explicitly enabled with `MCP_ENABLE_MUTATIONS=true`. The server should be placed behind HTTPS and never exposed directly on port 8787.

## Route protection

Protected routes use `axum-login`'s `login_required!` macro. The router in `ocg-server/src/router.rs` applies this middleware to all dashboard and user routes. Admin-only API endpoints check the user's role via the `AuthnBackend` before executing the handler.

## Secrets management

Secrets (OAuth client secrets, SMTP credentials, Stripe keys, Google API credentials) are passed as environment variables or via the YAML config file. They are never logged. The bootstrap script writes the server config to a file owned by the `ocg` system user with restricted permissions.

## Dependency auditing

`cargo audit` runs in CI on every change to `Cargo.lock` or `Cargo.toml`. The audit config is in `.cargo/audit.toml`.
