# Cleanup opportunities

## TODO / FIXME / HACK annotations

There are **0** `TODO`, `FIXME`, or `HACK` comments in `ocg-server/src/`. The codebase enforces a no-stale-annotation policy and carries none.

## Largest source files

The ten largest `.rs` files by line count:

| Lines | File |
|---|---|
| 5 079 | `ocg-server/src/handlers/auth/tests.rs` |
| 3 356 | `ocg-server/src/handlers/dashboard/group/events/tests.rs` |
| 2 666 | `ocg-server/src/handlers/dashboard/group/attendees/tests.rs` |
| 2 469 | `ocg-server/src/handlers/event/tests.rs` |
| 1 979 | `ocg-server/src/db/dashboard/group.rs` |
| 1 765 | `ocg-server/src/db/contract_tests.rs` |
| 1 715 | `ocg-server/src/handlers/tests.rs` |
| 1 571 | `ocg-server/src/db/mock.rs` |
| 1 538 | `ocg-server/src/services/notifications/tests.rs` |

All of the largest files are test modules. The largest non-test file is `ocg-server/src/db/dashboard/group.rs` at 1 979 lines, which contains all group-dashboard database queries. It may benefit from being split into sub-modules (e.g., one per dashboard section: events, members, settings) if the query count grows significantly.

## Migration file consolidation

`database/migrations/schema/` contains **102** migration files. Files `0002` through approximately `0097` are named `*_baseline_compatibility.sql`; these encode the historical schema shipped to existing deployments before tern was adopted. They cannot be consolidated into the initial migration without breaking the tern checksum chain for installations that have already run them. No action is needed unless a fresh-schema baseline is established and all active deployments are migrated to it first.
