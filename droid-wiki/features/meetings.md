# Meetings

The meetings feature provisions video-conference links for group events. It supports two external providers — Google Meet and Zoom — behind a shared trait-object interface.

## Provider abstraction

`ocg-server/src/services/meetings.rs` defines the `MeetingsProvider` trait:

```rust
pub(crate) trait MeetingsProvider {
    async fn create_meeting(&self, meeting: &Meeting) -> Result<MeetingProviderMeeting, MeetingProviderError>;
    async fn delete_meeting(&self, provider_meeting_id: &str) -> Result<(), MeetingProviderError>;
    async fn end_meeting(&self, provider_meeting_id: &str) -> Result<MeetingEndResult, MeetingProviderError>;
    async fn get_meeting(&self, provider_meeting_id: &str) -> Result<MeetingProviderMeeting, MeetingProviderError>;
    async fn update_meeting(&self, provider_meeting_id: &str, meeting: &Meeting) -> Result<(), MeetingProviderError>;
}

pub(crate) type DynMeetingsProvider = Arc<dyn MeetingsProvider + Send + Sync>;
pub(crate) type DynMeetingsProviders = Arc<HashMap<MeetingProvider, DynMeetingsProvider>>;
```

`mockall::automock` is applied in test builds.

### MeetingProviderError variants

| Variant | Retryable |
|---|---|
| `Client(String)` | No |
| `Network(String)` | Yes |
| `NotFound` | No |
| `NoSlotsAvailable` | No |
| `RateLimit { retry_after }` | Yes (with delay) |
| `Server(String)` | Yes |
| `Token(String)` | Yes |

## GoogleMeetMeetingsProvider

`ocg-server/src/services/meetings/google.rs` — wraps a `GoogleCalendarClient` that calls the Google Calendar API to create calendar events with a Google Meet conference link attached. The join URL is extracted from the event's meet URL field. `end_meeting` is a no-op (returns `AlreadyNotRunning`) because Google Meet has no server-side end-call API.

## ZoomMeetingsProvider

`ocg-server/src/services/meetings/zoom.rs` — wraps a Zoom API client. Unlike Google Meet, Zoom supports server-side meeting termination; `end_meeting` signals Zoom to close the meeting.

## MeetingsManager

`MeetingsManager::new` is called at server startup and launches three families of background workers via `tokio_util::task::TaskTracker`:

### MeetingsSyncWorker (2 workers)

Polls the database for meetings whose state is out of sync with their provider (e.g., a meeting that needs to be created, updated, or deleted on the provider side). Each iteration dequeues a pending meeting, calls the appropriate provider method, and records the result. On error the worker pauses for `PAUSE_ON_SYNC_ERROR` (30 s) before retrying; on an empty queue it waits `PAUSE_ON_SYNC_NONE` (30 s).

`Meeting::sync_action` computes the required `SyncAction` (`Create`, `Update`, `Delete`, or `None`) by comparing local state with the presence or absence of a `provider_meeting_id`.

### MeetingsAutoEndWorker (1 worker)

Periodically checks for meetings that have passed their scheduled end time and calls `end_meeting` on the provider. Pauses 1 minute when the queue is empty, 30 s on error.

### MeetingsClaimRecoveryWorker (1 worker)

Detects meetings whose processing was claimed but not completed within `MEETING_PROCESSING_TIMEOUT` (15 minutes) — e.g., due to a crash — and releases the claim so a sync worker can retry. Pauses 1 minute when nothing needs recovery.

## Link provisioning flow

1. A group admin creates or updates an event through the dashboard and selects a provider (Google Meet or Zoom).
2. The database record is written with `provider_meeting_id = NULL` and a `sync_pending` flag.
3. A `MeetingsSyncWorker` picks up the record, calls `create_meeting` on the appropriate provider, and stores the returned `join_url` and `provider_meeting_id`.
4. Subsequent event detail pages display the join URL to eligible attendees.
5. When the event is cancelled or rescheduled, the dashboard writes an update; a sync worker calls `update_meeting` or `delete_meeting` as appropriate.

## Active contributors

Sako Mammadov, Sergio Castaño Arteaga
