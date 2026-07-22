# Mock interviews

The mock interview feature lets community members request peer practice interviews, get matched with a partner, schedule the session, and record feedback. Option taxonomies are seeded from real community poll results.

## Domain types

`ocg-server/src/types/mock_interviews.rs` defines all input and view types.

### MockInterviewOption

A static struct carrying a `value` slug, a `label`, and a `votes` count from the founding poll. The `demand_percent` method normalises votes to a 8–100% scale relative to the highest-voted option, used to render demand bars in the UI.

### Taxonomy options

All taxonomy options were derived from a community poll:

| Dimension | Top option | Top vote count |
|---|---|---|
| `practice_role` | `interviewee` | 41 |
| `interview_type` | `software_engineering` | 49 |
| `target_company` | `remote_global` | 57 |
| `seniority` | `senior` (3–7 y exp) | 37 |

Additional options include `ai_ml`, `startup_cofounder`, `product_management`, `devops_cloud`, `security`, `behavioral_hr`, and geographic `location` codes.

### Request lifecycle

A mock interview request progresses through: `requested → matched → scheduled → completed` (or `canceled` from `scheduled`).

A match progresses through: `matched → scheduled → completed` (or `canceled` from `scheduled`).

## Database operations

`ocg-server/src/db/mock_interviews.rs` defines the `DBMockInterviews` trait:

| Method | Description |
|---|---|
| `get_mock_interview_dashboard` | Returns the admin queue and all existing matches, filtered by `MockInterviewFilters`. |
| `list_user_mock_interview_matches` | Returns matches where the caller is an assigned participant. |
| `list_user_mock_interview_requests` | Returns requests the caller submitted. |
| `add_mock_interview_request` | Creates a new request; returns the new UUID. |
| `request_group_mock_interviewer` | Directly requests a specific group member as interviewer. |
| `upsert_mock_interview_match` | Creates or updates a match for a given request. |
| `get_mock_interview_match_notification_context` | Fetches participant details needed to send match notification emails. |
| `update_mock_interview_feedback` | Records admin-side feedback and final match status. |
| `update_user_mock_interview_feedback` | Records per-participant feedback for the caller's role. |
| `update_user_mock_interview_schedule` | Records the scheduled time proposed by a participant. |

## Workflow

1. A community member submits a request (practice role, interview type, target company, seniority, location, availability notes).
2. An admin reviews the open queue via `get_mock_interview_dashboard`.
3. The admin pairs two participants via `upsert_mock_interview_match`; request status advances to `matched`.
4. Match notification emails are sent using context from `get_mock_interview_match_notification_context`.
5. Each participant proposes a schedule via `update_user_mock_interview_schedule`; status becomes `scheduled`.
6. After the session, participants submit feedback; the admin marks the match `completed`.
7. Either party may cancel at the `scheduled` stage.

## Active contributors

Sako Mammadov, Sergio Castaño Arteaga
