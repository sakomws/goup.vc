# Dependencies

Key dependencies for `ocg-server` and `ocg-redirector` from `Cargo.toml`.

## Web framework

| Crate | Version | Purpose |
|-------|---------|---------|
| `axum` | 0.8 | HTTP router and handler framework |
| `axum-login` | 0.17 | Session-based authentication layer |
| `axum-messages` | 0.7 | Flash messages middleware |
| `tower` | 0.5 | Middleware abstractions |
| `tower-http` | 0.6 | HTTP-specific middleware (trace, set-header) |
| `tower-sessions` | 0.14 | Session management |
| `tokio` | 1.52 | Async runtime (macros, process, rt-multi-thread, signal, sync, time) |

## Database

| Crate | Version | Purpose |
|-------|---------|---------|
| `deadpool-postgres` | 0.14 | Async PostgreSQL connection pool |
| `postgres-openssl` | 0.5 | TLS support for PostgreSQL connections |
| `openssl` | 0.10 | OpenSSL bindings for TLS |
| `tokio-postgres` | 0.7 | Async PostgreSQL driver |

## Serialization

| Crate | Version | Purpose |
|-------|---------|---------|
| `serde` | 1.0 | Serialization/deserialization framework |
| `serde_json` | 1.0 | JSON support |
| `serde_qs` | 1.1 | Query string deserialization (with axum feature) |
| `serde_urlencoded` | 0.7 | URL-encoded form data |
| `serde_with` | 3.21 | Extra serde helpers (`skip_serializing_none`, etc.) |

## Templating

| Crate | Version | Purpose |
|-------|---------|---------|
| `minijinja` | 2.10 | Jinja2-style HTML template engine |
| `minijinja-contrib` | 2.10 | Extra MiniJinja filters |
| `rust-embed` | 8.11 | Embed static assets into the binary at compile time |

## Auth and crypto

| Crate | Version | Purpose |
|-------|---------|---------|
| `oauth2` | 4.4 | OAuth 2.0 client |
| `openidconnect` | 3.5 | OIDC client (used for Google login) |
| `password-auth` | 1.0 | Argon2id password hashing and verification |
| `sha2` | 0.11 | SHA-256 for token hashing |
| `subtle` | 2.6 | Constant-time comparison |

## Validation

| Crate | Version | Purpose |
|-------|---------|---------|
| `garde` | 0.22 | Declarative input validation via `#[derive(Validate)]` |

## Configuration

| Crate | Version | Purpose |
|-------|---------|---------|
| `figment` | 0.10 | Configuration layering from YAML and environment |
| `clap` | 4.5 | CLI argument parsing |

## Payments

| Crate | Version | Purpose |
|-------|---------|---------|
| `async-stripe` | 0.40 | Stripe API client |

## Images

| Crate | Version | Purpose |
|-------|---------|---------|
| `aws-sdk-s3` | 1.81 | S3 image storage |
| `image` | 0.25 | Image decoding and processing |

## Utilities

| Crate | Version | Purpose |
|-------|---------|---------|
| `anyhow` | 1.0 | Error handling |
| `thiserror` | 2.0 | Derive macros for typed errors |
| `tracing` | 0.1 | Structured logging and spans |
| `tracing-subscriber` | 0.3 | Log output formatting |
| `uuid` | 1.11 | UUID generation and parsing |
| `chrono` | 0.4 | Date/time types |
| `chrono-tz` | 0.10 | Timezone support |
| `time` | 0.3 | Time types (used by tower-sessions) |
| `reqwest` | 0.12 | HTTP client for external API calls |
| `async-trait` | 0.1 | Trait objects with async methods |
| `strum` | 0.28 | Enum/string conversion macros |
| `rkyv` | 0.7 | Zero-copy deserialization (cache layer) |

## Node.js (MCP server)

The MCP server in `mcp/` uses only Node.js built-in modules (`http`, `url`, `fs`). No npm dependencies beyond what ships with Node.js itself. See `mcp/package.json`.
