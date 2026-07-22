# Design decisions

Key architectural choices made during the development of the GOUP Alliance platform.

## Rust and axum over other web frameworks

The platform was built from the start in Rust using the `axum` framework. The primary motivation was performance and safety guarantees: Rust eliminates whole classes of memory and concurrency bugs at compile time, which is valuable for a multi-tenant application handling authentication, payments, and sensitive user data. `axum` was chosen over alternatives such as Actix-web because it is built directly on `tower` middleware, giving a composable and well-typed middleware stack with the same ergonomics as async Rust.

The trade-off is a steeper learning curve and longer compile times compared to frameworks in Go, Python, or Node.js. The team accepted this trade-off in exchange for runtime correctness.

## MiniJinja and HTMX over a JavaScript SPA framework

The UI is rendered server-side with MiniJinja templates (`askama` crate, which compiles Jinja2-compatible templates at build time) and enhanced with HTMX for partial page updates. This approach avoids a separate JavaScript build pipeline, eliminates client/server data-synchronisation overhead, and keeps the codebase in a single language.

HTMX partial swaps (`hx-swap`, `hx-push-url`, `HX-Push-Url` response headers) are used throughout the dashboard to update only the changed region of the page without a full navigation. This delivers SPA-like interactivity at the cost of less granular client-side state management, which the team judged acceptable for a content-and-form-heavy community platform.

`minify-html` is applied to rendered HTML before sending to the client to reduce transfer size without a separate asset pipeline step.

## tern for database migrations

[tern](https://github.com/jackc/tern) (a Go-based schema migration tool) was chosen for managing PostgreSQL schema changes. Migrations live in `database/migrations/schema/` as sequentially numbered `.sql` files. The repo contains 102 migration files, of which the first ~97 carry the suffix `_baseline_compatibility` — these represent the baseline schema shipped to existing deployments before the migration tool was introduced, allowing tern to run against pre-existing databases without re-executing historical DDL.

tern was preferred over Rust-native options (e.g., `refinery`, `sqlx migrate`) because the team was already familiar with it from prior projects and because it operates as a standalone binary independent of application code, making it easy to run in CI or during container start-up.

## Trait objects (DynDB, DynImageStorage, DynMeetingsProvider) for testability

Core service dependencies are typed as `Arc<dyn Trait + Send + Sync>` aliases:

- `DynDB` — database operations
- `DynImageStorage` — image storage
- `DynMeetingsProvider` / `DynMeetingsProviders` — meeting providers

This indirection allows tests to inject `mockall`-generated mock implementations without standing up real infrastructure. The `#[cfg_attr(test, automock)]` attribute on each trait generates a `Mock*` type automatically.

The cost is a small per-call vtable dispatch and slightly more complex type signatures, but the gain in test isolation and the ability to swap backends (e.g., `DbImageStorage` in development, `S3ImageStorage` in production) justifies the overhead.

## rust-embed for static assets

Static assets (CSS, JavaScript, fonts) are embedded directly into the server binary at compile time using the `rust-embed` crate. This produces a single self-contained binary with no dependency on a file system layout at runtime, simplifying deployment to containers and reducing the number of files that must be present in the production image.

In debug builds, rust-embed reads from the file system to support hot-reload; in release builds, the bytes are baked in. The `mime_guess` crate is used to derive correct `Content-Type` headers for embedded assets.
