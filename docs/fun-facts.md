# Fun facts

Data collected on 2026-07-22.

## Zero TODO/FIXME comments in Rust source

A search across all 286 Rust source files (`ocg-server/` and `ocg-redirector/`) finds **zero** `TODO` or `FIXME` comments. The codebase carries no documented deferred work in source code.

## Test files dominate the largest-file list

Eight of the nine largest Rust files are test modules. The ratio of test code to production code is high enough that the largest non-test file — `ocg-server/src/db/dashboard/group.rs` at 1,979 lines — ranks fifth overall.

## A five-month gap in the commit history

After 295 commits in the first five months (August–December 2024), the repository went silent for nearly five months — one commit landed in January 2025, then nothing until June 2025. Development resumed and eventually produced the busiest single month on record: **182 commits in June 2026**.

## 676 SQL migration files

The database migration history in `database/migrations/schema/` contains 676 files totalling 104,760 lines. This is roughly on par with the entire Rust codebase in line count, making SQL the joint second-largest language in the repository by lines.

## One bot-attributed commit across 968 total

Only one commit carries a `Co-authored-by: [bot]` trailer. Almost all code has been authored by human contributors.
