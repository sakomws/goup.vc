# Discovery catalogs

`jobs-sources.csv` and `events-sources.csv` are source catalogs, not job or event
records. Every row uses this schema:

```text
url,label,category,region,verified_at,provenance
```

`verified_at` is the UTC date on which `build_catalogs.py` successfully fetched
the canonical URL. `provenance` identifies the pinned upstream dataset and the
live HTTP status observed by that build; it is deliberately not a claim that an
organizer endorses this catalog.

## Rebuild and validate

```sh
python3 database/data/discovery/build_catalogs.py --build
python3 database/data/discovery/build_catalogs.py --validate
```

The build uses only the Python standard library. It fetches the following pinned
open datasets, resolves redirects, accepts only successful HTTP(S) responses,
canonicalizes their URLs, and de-duplicates before writing CSV:

- Jobs: `Feashliaa/job-board-aggregator` at
  `bc0fa75f98b6151177752f7d1ad54d9bd1b0a355`. The upstream exposes public ATS
  company identifiers. The build accepts public Greenhouse, Lever, Ashby,
  BambooHR, and Paylocity career endpoints and assigns the required
  `enterprise-careers` category. This is ATS/public-career-page data; it does
  not use a Fortune ranking.
- Events: `dmitryvinn/tech-conferences` at
  `dc8cbe0611405a56849e2e7a9c45addde099d6a9`. Its `conferences.json` supplies
  organizer-maintained conference homepages plus topic and region metadata.
- Events: `tech-conferences/conference-data` (the confs.tech dataset) at
  `56efdfcf195b7531ebda8f56a3c514002ddd2973`. The build ingests its 2025–2027
  topic JSON records, which identify the official conference URL, country, and
  topic. Historical files are deliberately excluded: they would weaken the
  recurring-calendar quality of the catalog.

The event catalog combines those independently curated sources, then performs
the same live verification and URL-level de-duplication as jobs. `--validate` is
the acceptance gate: exactly 1,000 rows and unique canonical HTTP(S) URLs are
required in each CSV.
