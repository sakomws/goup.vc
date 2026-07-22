# Lore

A timeline of the GOUP Alliance repository from first commit to today.

## Eras

### Era 1 — Foundation (2024-08-05 to 2024-08-26)

The project started on 5 August 2024 with an "Initial commit." Within four days the first database schema landed, a minimal axum router was wired up, and the codebase was rebranded from an unnamed project to **Open Community Groups** (`2024-08-09 Rebrand to Open Community Groups`). By late August the first explore endpoints were in place.

10 commits total.

### Era 2 — Community platform build (2024-09-01 to 2024-10-31)

The heaviest early-stage push: 76 commits in September and 83 in October. The explore page, community index, event cards, group cards, macros/templates system, mobile layouts, and markdown-to-HTML conversion all landed here. The `figment` configuration library replaced an ad-hoc config struct. The router was refactored.

159 commits total.

### Era 3 — Feature expansion (2024-11-01 to 2024-12-31)

206 commits across November and December 2024 — the densest two-month stretch before the 2026 sprint. Notifications, dashboard improvements, and broader group/event management shipped. December 2024 alone saw 129 commits, the second-highest monthly count in the project's history.

206 commits total.

### Era 4 — Quiet period (2025-01-01 to 2025-05-31)

A single commit in January 2025, then silence until June. No commits in February through May. The project was likely dormant or work was happening outside this repository.

1 commit total.

### Era 5 — Revival (2025-06-01 to 2025-12-31)

Activity resumed in June 2025 and accelerated: 48 commits in August, 93 in September — the third-highest monthly total — tapering through October–December. Core platform stabilisation and dependency upgrades occurred during this period.

218 commits total.

### Era 6 — Alliance platform (2026-01-01 to 2026-07-22)

The project shifted from a community events platform to a full alliance product. Features added in rapid succession: CoffeeMeet member matching, gamification analytics, group join approval, accelerator management, mock interview scheduling, intentional dating curation, book exchange dashboards, global job discovery, and a complete public site redesign. June 2026 was the single busiest month in the project's history with 182 commits (PR #55 through #116). The project was also renamed/rebranded — commit `Fix/public site branding (#113)` and `Redesign landing page and public theme (#91)` mark the shift to the GOUP Alliance identity.

374 commits total (through 2026-07-22).

## Longest-standing features

- **Explore page** — present since 2024-08-15 (`Add route for community explore page`), continuously evolved.
- **Community index** — `get_community_index_data` db function added 2024-09-19, still central.
- **PostgreSQL/tern migration system** — first schema file committed 2024-08-06, now spanning 676 migration files.

## Major rewrites and renames

| Date | Event |
|------|-------|
| 2024-08-09 | Rebranded to **Open Community Groups** |
| 2024-09-18 | Configuration replaced with `figment` |
| 2024-09-26 | HTTP router refactored |
| 2026-07-20 | Public site rebranded to **GOUP Alliance** identity |

## Growth trajectory

```
2024-Q3   86 commits   — bootstrapping
2024-Q4  289 commits   — initial feature build
2025-Q1    1 commit    — dormant
2025-Q2    0 commits   — dormant
2025-Q3  149 commits   — revival
2025-Q4   62 commits   — steady
2026-Q1   80 commits   — alliance features begin
2026-Q2  227 commits   — peak output (Jun 182)
2026-Q3   41 commits   — in progress (Jul only)
```
