# Group distribution dashboard

The group dashboard's **Distribution** tab is a planning and measurement workspace for event campaigns.
It does not connect social accounts or publish through social APIs.

Organizers with `group.distribution.write` can:

- create campaigns, partner referral codes, and frozen tracked links;
- plan LinkedIn, X, and Instagram content through `idea`, `draft`, `ready`, and `posted_manual`;
- reuse captions, calls to action, hashtags, and event image references;
- copy prepared content, open the public platform compose page, and manually mark it posted;
- opt into idempotent reminders when ready content becomes due; and
- export campaign and link metrics as CSV.

Public `/r/{code}` redirects accept only public HTTP(S) targets. Incoming non-attribution query parameters
are retained, while the stored UTM and referral values always win. Clicks are deduplicated per link and day
using a one-way fingerprint; raw IP addresses and request headers are not stored.

Registration totals use `event_registration_attribution` when that independently migrated table exists.
Without it, the dashboard remains available and reports zero registrations.
