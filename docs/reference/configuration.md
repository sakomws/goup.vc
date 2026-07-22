# Configuration

The server is configured via a YAML file (default: `~/.config/ocg/server.yml`) and environment variables with the `OCG_` prefix. Environment variables override file values; nested keys use `__` as the separator (e.g., `OCG_DB__HOST`).

Config is loaded by `ocg-server/src/config.rs` using Figment.

## Database (`db`)

Standard `deadpool-postgres` connection config.

| Key | Env var | Default | Description |
|-----|---------|---------|-------------|
| `db.host` | `OCG_DB__HOST` | `localhost` | PostgreSQL host |
| `db.port` | `OCG_DB__PORT` | `5432` | PostgreSQL port |
| `db.dbname` | `OCG_DB__DBNAME` | `ocg` | Database name |
| `db.user` | `OCG_DB__USER` | `postgres` | Database user |
| `db.password` | `OCG_DB__PASSWORD` | — | Database password |
| `db.pool.max_size` | `OCG_DB__POOL__MAX_SIZE` | `25` | Max connections in pool |

## Email (`email`)

| Key | Env var | Description |
|-----|---------|-------------|
| `email.from_address` | `OCG_EMAIL__FROM_ADDRESS` | Sender address for all outgoing email |
| `email.from_name` | `OCG_EMAIL__FROM_NAME` | Sender display name |
| `email.smtp.host` | `OCG_EMAIL__SMTP__HOST` | SMTP server hostname |
| `email.smtp.port` | `OCG_EMAIL__SMTP__PORT` | SMTP port (default 587) |
| `email.smtp.username` | `OCG_EMAIL__SMTP__USERNAME` | SMTP auth username |
| `email.smtp.password` | `OCG_EMAIL__SMTP__PASSWORD` | SMTP auth password |
| `email.rcpts_whitelist` | `OCG_EMAIL__RCPTS_WHITELIST` | Optional list of allowed recipient addresses (`null` = allow all) |

## Images (`images`)

| Key | Env var | Description |
|-----|---------|-------------|
| `images.provider` | `OCG_IMAGES__PROVIDER` | `db` (store in PostgreSQL) or `s3` |
| `images.s3.access_key_id` | `OCG_IMAGES__S3__ACCESS_KEY_ID` | AWS access key (S3 only) |
| `images.s3.secret_access_key` | `OCG_IMAGES__S3__SECRET_ACCESS_KEY` | AWS secret key (S3 only) |
| `images.s3.bucket` | `OCG_IMAGES__S3__BUCKET` | S3 bucket name (S3 only) |
| `images.s3.region` | `OCG_IMAGES__S3__REGION` | AWS region (S3 only) |
| `images.s3.endpoint` | `OCG_IMAGES__S3__ENDPOINT` | Optional custom endpoint (S3-compatible APIs) |
| `images.s3.force_path_style` | `OCG_IMAGES__S3__FORCE_PATH_STYLE` | `true` for MinIO-style endpoints |

## Logging (`log`)

| Key | Env var | Default | Description |
|-----|---------|---------|-------------|
| `log.format` | `OCG_LOG__FORMAT` | `json` | Log format: `json` or `pretty` |

## HTTP server (`server`)

| Key | Env var | Default | Description |
|-----|---------|---------|-------------|
| `server.addr` | `OCG_SERVER__ADDR` | `127.0.0.1:9000` | Bind address |
| `server.base_url` | `OCG_SERVER__BASE_URL` | — | Public-facing URL (used in links, OAuth redirects) |
| `server.disable_referer_checks` | `OCG_SERVER__DISABLE_REFERER_CHECKS` | `false` | Disable referer header validation |
| `server.cookie.secure` | `OCG_SERVER__COOKIE__SECURE` | `true` | Set Secure flag on session cookies |
| `server.login.email` | `OCG_SERVER__LOGIN__EMAIL` | `true` | Enable email/password login |
| `server.login.github` | `OCG_SERVER__LOGIN__GITHUB` | `false` | Enable GitHub OAuth |
| `server.login.linkedin` | `OCG_SERVER__LOGIN__LINKEDIN` | `false` | Enable LinkedIn OAuth |

## OAuth2 providers (`server.oauth2`)

Keyed by provider name (`github`, `linkedin`). Each entry has:

| Sub-key | Description |
|---------|-------------|
| `client_id` | OAuth app client ID |
| `client_secret` | OAuth app client secret |
| `auth_url` | Authorization endpoint |
| `token_url` | Token endpoint |
| `redirect_uri` | Callback URL (must match provider settings) |
| `scopes` | Requested scopes |

## OIDC providers (`server.oidc`)

Keyed by provider name (`google`). Each entry has:

| Sub-key | Description |
|---------|-------------|
| `client_id` | Client ID |
| `client_secret` | Client secret |
| `issuer_url` | OIDC issuer URL |
| `redirect_uri` | Callback URL |
| `scopes` | Requested scopes |

## Meetings (`meetings`) — optional

| Key | Description |
|-----|-------------|
| `meetings.google_meet.enabled` | Enable Google Meet integration |
| `meetings.google_meet.calendar_id` | Google Calendar ID (`primary` or specific ID) |
| `meetings.google_meet.client_id` | Google OAuth client ID |
| `meetings.google_meet.client_secret` | Google OAuth client secret |
| `meetings.google_meet.refresh_token` | OAuth refresh token |
| `meetings.google_meet.max_participants` | Max meeting participants (default 100) |
| `meetings.zoom.enabled` | Enable Zoom integration |
| `meetings.zoom.account_id` | Zoom account ID |
| `meetings.zoom.client_id` / `client_secret` | Server-to-Server OAuth credentials |
| `meetings.zoom.host_pool_users` | List of Zoom host email addresses |
| `meetings.zoom.webhook_secret_token` | Zoom webhook validation token |

## Payments (`payments`) — optional

Currently only Stripe is supported.

| Key | Description |
|-----|-------------|
| `payments.stripe.mode` | `test` or `live` |
| `payments.stripe.publishable_key` | Stripe publishable key |
| `payments.stripe.secret_key` | Stripe secret key |
| `payments.stripe.webhook_secret` | Stripe webhook signing secret |

## Recording publishing (`recording_publishing`) — optional

| Key | Description |
|-----|-------------|
| `recording_publishing.youtube.enabled` | Enable YouTube auto-publish |
| `recording_publishing.youtube.client_id` / `client_secret` | Google OAuth credentials |
| `recording_publishing.youtube.refresh_token` | OAuth refresh token |
| `recording_publishing.youtube.drive_folder_id` | Google Drive folder to scan for recordings |
| `recording_publishing.youtube.visibility` | `public`, `unlisted`, or `private` |
| `recording_publishing.youtube.publish_delay_minutes` | Delay before publishing (default 0) |
| `recording_publishing.youtube.retry_delay_minutes` | Retry interval on failure |

## Integrations (`integrations`) — optional

| Key | Description |
|-----|-------------|
| `integrations.you_com.enabled` | Enable You.com event/job discovery |
| `integrations.you_com.api_key` | You.com API key |
| `integrations.you_com.search_url` | You.com search API endpoint |
| `integrations.you_com.schedule_timezone` | Timezone for daily discovery schedule |
| `integrations.you_com.schedule_hour` | Hour of day to run discovery (0–23) |
