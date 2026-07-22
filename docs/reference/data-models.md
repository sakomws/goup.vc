# Data models

Domain types are defined in `ocg-server/src/types/`. Most are serializable with `serde` and use `uuid::Uuid` as primary key types.

## User

Defined in `ocg-server/src/types/user.rs`.

| Field | Type | Description |
|-------|------|-------------|
| `user_id` | `Uuid` | Primary key |
| `username` | `String` | Unique login name |
| `name` | `Option<String>` | Display name |
| `bio` | `Option<String>` | Short biography |
| `photo_url` | `Option<String>` | Profile photo URL |
| `title` | `Option<String>` | Job title |
| `company` | `Option<String>` | Employer |
| `coffee_meet_enabled` | `bool` | Accepts direct CoffeeMeet requests |
| `provider` | `Option<UserProvider>` | OAuth provider metadata |
| social URLs | `Option<String>` | GitHub, LinkedIn, Twitter, Bluesky, Substack, YouTube, Facebook, website |

`PublicUserProfile` is a subset safe for public profile cards.

## Alliance

Defined in `ocg-server/src/types/alliance.rs`. Top-level organization. An alliance has a unique `name` (slug) and `display_name`, an optional logo, and settings for enabled features (dating, wiki, store, etc.).

## Group

Defined in `ocg-server/src/types/group.rs`.

Three variants exist at different detail levels:
- `GroupMinimal` — ID, name, slug (used in dashboard selectors)
- `GroupSummary` — adds location, category, member count
- Full group — adds members list, settings, payment config

Key fields: `group_id`, `name`, `slug`, `slug_pretty` (admin-customized URL slug), `active`, alliance reference, location, category.

## Event

Defined in `ocg-server/src/types/event.rs`.

Two variants:
- `EventSummary` — card-level data (title, dates, location, RSVP status)
- Full event — adds description, agenda, CFS questions, attendee list, meeting links, ticket types

Key fields: `event_id`, `alliance_name`, `group_id`, `title`, `start`, `end`, `timezone`, `capacity`, `waitlist_enabled`, `attendee_approval_required`, `canceled`, `online_meeting_url`, `youtube_video_url`.

## Job

Defined in `ocg-server/src/types/jobs.rs`. Represents a job posting on the board.

Key fields: `job_id`, `title`, `description`, `company`, `location`, `url`, `published`, `expires_at`, discovery source.

## Landscape entry

Defined in `ocg-server/src/types/landscape.rs`. Represents a startup, OSS project, partner community, or similar entity in the ecosystem directory.

## Payment types

Defined in `ocg-server/src/types/payments.rs`.

| Type | Description |
|------|-------------|
| `EventPurchaseStatus` | `pending`, `completed`, `expired`, `refund_pending`, `refund_requested`, `refunded` |
| `EventTicketType` | Ticket tier for an event (name, price, capacity) |
| `EventDiscountCode` | Discount code with amount/percentage and usage limits |
| `EventRefundRequestStatus` | `pending`, `approved`, `rejected` |
| `PaymentMode` | `test` or `live` |
| `PaymentProvider` | Currently only `stripe` |
| `GroupPaymentRecipient` | Stripe account details for a group's payment recipient |

## Permissions

Defined in `ocg-server/src/types/permissions.rs`. Used by `AuthnBackend` to gate routes and actions by role (member, group admin, alliance admin, platform admin).

## Search

Defined in `ocg-server/src/types/search/`. The unified search result type returned by `/api/v1/search`, combining events, groups, jobs, landscape entries, and wiki sources into a single ranked list.

## Pagination

Defined in `ocg-server/src/types/pagination.rs`. Standard cursor-based pagination envelope used by list endpoints.
